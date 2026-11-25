use crate::{
    api::Api,
    settings::{time, SavedSession, Settings},
};
use garlic_state::{self as files, StateError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, sync::Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
struct Lease {
    device_id: Uuid,
    generation: u64,
    token: String,
    expires_at: u64,
}
struct Entry {
    lease: Lease,
    session: SavedSession,
}
struct State {
    inherited: Lease,
    inherited_origin: Option<String>,
    known: BTreeMap<String, Lease>,
    entries: BTreeMap<String, Entry>,
}
pub struct Devices {
    api: Api,
    path: PathBuf,
    origins: PathBuf,
    state: Mutex<State>,
}
impl Devices {
    pub fn new(api: Api, settings: &Settings) -> Result<Self, StateError> {
        let path = settings.side("device.json");
        let saved = files::edit_or_create(
            &path,
            || {
                Ok(Lease {
                    device_id: Uuid::new_v4(),
                    generation: 0,
                    token: String::new(),
                    expires_at: 0,
                })
            },
            |l| {
                if l.device_id.is_nil() {
                    return Err("invalid stored device identifier".into());
                }
                Ok(l.clone())
            },
        )?;
        let origins = settings.side("device-origins.json");
        let known = files::edit_or_create(
            &origins,
            || Ok(BTreeMap::new()),
            |book: &mut BTreeMap<String, Lease>| {
                book.retain(|_, lease| {
                    lease.device_id == saved.device_id && lease.expires_at > time()
                });
                Ok(book.clone())
            },
        )?;
        // A legacy lease has no issuer field. Only import it for the single
        // authenticated origin when no per-origin lease history exists.
        let inherited_origin = if known.is_empty() && !saved.token.is_empty() {
            let sessions = settings.sessions()?;
            if sessions.entries.len() == 1 {
                sessions.entries.into_keys().next()
            } else {
                None
            }
        } else {
            None
        };
        Ok(Self {
            api,
            path,
            origins,
            state: Mutex::new(State {
                inherited: saved,
                inherited_origin,
                known,
                entries: Default::default(),
            }),
        })
    }
    fn record(&self, url: &str, lease: &Lease, selected: bool) -> Result<(), StateError> {
        files::edit_or_create(
            &self.origins,
            || Ok(BTreeMap::new()),
            |book: &mut BTreeMap<String, Lease>| {
                book.retain(|_, old| old.device_id == lease.device_id && old.expires_at > time());
                let previous = book.get(url);
                if previous.is_none_or(|old| old.generation <= lease.generation) {
                    book.insert(url.to_owned(), lease.clone());
                }
                Ok(())
            },
        )?;
        files::edit_or_create(
            &self.path,
            || Ok(lease.clone()),
            |last: &mut Lease| {
                // Older programs expect the last selected lease in the original file.
                if last.device_id == lease.device_id && (selected || last.token == lease.token) {
                    *last = lease.clone();
                }
                Ok(())
            },
        )
    }
    fn renew(
        &self,
        url: &str,
        session: &SavedSession,
        lease: &mut Lease,
    ) -> Result<(), StateError> {
        self.api.call(
            url,
            Method::POST,
            "/v1/auth/device/renew",
            Some(&session.token),
            None,
            Some(&json!({"token":lease.token})),
        )?;
        lease.expires_at = time() + 60;
        Ok(())
    }
    pub fn claim(
        &self,
        url: &str,
        session: &SavedSession,
        takeover: bool,
    ) -> Result<Option<String>, StateError> {
        let mut s = self.state.lock().unwrap();
        if let Some(e) = s.entries.get_mut(url).filter(|e| {
            !takeover && e.session.token == session.token && e.lease.expires_at > time() + 10
        }) {
            return Ok(Some(e.lease.token.clone()));
        }
        // Never send an unidentified legacy lease to multiple coordinators.
        let known = s.known.get(url);
        let inherited = Some(&s.inherited).filter(|lease| {
            s.inherited_origin.as_deref() == Some(url)
                && known.is_none_or(|old| old.token != lease.token)
        });
        let mut renewed = None;
        if !takeover {
            for candidate in known.into_iter().chain(inherited) {
                if candidate.expires_at <= time() || candidate.token.is_empty() {
                    continue;
                }
                let mut lease = candidate.clone();
                if self.renew(url, session, &mut lease).is_ok() {
                    renewed = Some(lease);
                    break;
                }
            }
        }
        let lease = if let Some(lease) = renewed {
            lease
        } else {
            let value = self.api.call(
                url,
                Method::POST,
                "/v1/auth/device/claim",
                Some(&session.token),
                None,
                Some(&json!({"device_id":s.inherited.device_id,"takeover":takeover})),
            )?;
            serde_json::from_value::<Lease>(value)?
        };
        // Stock coordinators use this tuple for accounts exempt from device leases.
        if lease.device_id == s.inherited.device_id
            && lease.generation == 0
            && lease.expires_at == u64::MAX
            && lease.token.is_empty()
        {
            return Ok(None);
        }
        if lease.device_id != s.inherited.device_id
            || lease.token.is_empty()
            || lease.expires_at <= time()
        {
            return Err("invalid device claim response".into());
        }
        self.record(url, &lease, true)?;
        let token = lease.token.clone();
        s.known.insert(url.to_owned(), lease.clone());
        s.entries.insert(
            url.into(),
            Entry {
                lease,
                session: session.clone(),
            },
        );
        Ok(Some(token))
    }
    pub fn forget_rejected(&self, url: &str, token: &str) {
        let mut s = self.state.lock().unwrap();
        if s.entries.get(url).is_some_and(|e| e.lease.token == token) {
            s.entries.remove(url);
        }
        if s.known.get(url).is_some_and(|l| l.token == token) {
            s.known.remove(url);
        }
    }
    pub fn maintain(&self) {
        let due: Vec<_> = self.state.lock().unwrap().entries.iter()
            .filter(|(_, e)| e.lease.expires_at <= time() + 35)
            .map(|(url, e)| (url.clone(), e.session.clone(), e.lease.clone()))
            .collect();
        for (url, session, mut lease) in due {
            // A slow authority must not stall unrelated connections using cached leases.
            if let Err(error) = self.renew(&url, &session, &mut lease) {
                if error.to_string() == "HTTP 409: device_in_use" {
                    self.forget_rejected(&url, &lease.token);
                }
                continue;
            }
            let mut s = self.state.lock().unwrap();
            let Some(current) = s.entries.get_mut(&url) else { continue };
            if current.session.token != session.token || current.lease.token != lease.token
                || current.lease.expires_at > lease.expires_at { continue; }
            current.lease = lease.clone();
            let _ = self.record(&url, &lease, false);
            s.known.insert(url, lease);
        }
    }
    pub fn release(&self) {
        let mut s = self.state.lock().unwrap();
        for (url, e) in std::mem::take(&mut s.entries) {
            let released = self.api.call(
                &url,
                Method::POST,
                "/v1/auth/device/release",
                Some(&e.session.token),
                None,
                Some(&json!({"token":e.lease.token})),
            );
            if released.is_ok() {
                let _ = files::edit_or_create(
                    &self.origins,
                    || Ok(BTreeMap::new()),
                    |book: &mut BTreeMap<String, Lease>| {
                        if book
                            .get(&url)
                            .is_some_and(|lease| lease.token == e.lease.token)
                        {
                            book.remove(&url);
                        }
                        Ok(())
                    },
                );
                s.known.remove(&url);
            }
        }
    }
}
