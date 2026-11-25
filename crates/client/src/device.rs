use crate::{
    api::Api,
    settings::{time, SavedSession, Settings},
};
use garlic_state::{self as files, StateError};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, sync::{Arc, Mutex}};
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
    claims: BTreeMap<String, Arc<Mutex<()>>>,
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
                claims: BTreeMap::new(),
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
        // Serialize claim/renew for this issuer only. A timeout on one node
        // must not hold the device cache lock needed by every other route.
        let gate = self.state.lock().unwrap().claims.entry(url.to_owned()).or_default().clone();
        let _claim = gate.lock().unwrap();
        let s = self.state.lock().unwrap();
        if let Some(e) = s.entries.get(url).filter(|e| {
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
        let candidates: Vec<_> = known.into_iter().chain(inherited).cloned().collect();
        let device_id = s.inherited.device_id;
        drop(s);
        let mut renewed = None;
        if !takeover {
            for mut lease in candidates {
                if lease.expires_at <= time() || lease.token.is_empty() {
                    continue;
                }
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
                Some(&json!({"device_id":device_id,"takeover":takeover})),
            )?;
            serde_json::from_value::<Lease>(value)?
        };
        // Stock coordinators use this tuple for accounts exempt from device leases.
        if lease.device_id == device_id
            && lease.generation == 0
            && lease.expires_at == u64::MAX
            && lease.token.is_empty()
        {
            return Ok(None);
        }
        if lease.device_id != device_id
            || lease.token.is_empty()
            || lease.expires_at <= time()
        {
            return Err("invalid device claim response".into());
        }
        let mut s = self.state.lock().unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Config;
    use std::{io::{BufRead, BufReader, Write}, net::TcpListener, sync::mpsc, thread, time::Duration};

    #[test]
    fn stalled_claim_does_not_hold_other_issuers_cache() {
        let dir = std::env::temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir(&dir).unwrap();
        let settings = Settings { config: dir.join("client.json") };
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let slow = format!("http://{}", listener.local_addr().unwrap());
        let fast = "http://127.0.0.1:1".to_owned();
        settings.save_key("fixture", vec![slow.clone(), fast.clone()]).unwrap();
        let session = settings.sessions().unwrap().entries[&fast].clone();
        let devices = Devices::new(Api::new(&Config::default()).unwrap(), &settings).unwrap();
        let device_id = devices.state.lock().unwrap().inherited.device_id;
        devices.state.lock().unwrap().entries.insert(fast.clone(), Entry {
            session: session.clone(),
            lease: Lease { device_id, generation: 1, token: "cached".into(), expires_at: time()+300 },
        });
        let (pending_tx,pending_rx) = mpsc::channel();
        let (finish_tx,finish_rx) = mpsc::channel();
        thread::scope(|scope| {
            scope.spawn(move || {
                let (mut socket,_) = listener.accept().unwrap();
                let mut line = String::new();
                BufReader::new(&socket).read_line(&mut line).unwrap();
                assert!(line.starts_with("POST /v1/auth/device/claim "));
                pending_tx.send(()).unwrap();
                finish_rx.recv_timeout(Duration::from_secs(3)).ok();
                let _ = socket.write_all(b"HTTP/1.1 503 Unavailable\r\nContent-Length: 0\r\n\r\n");
            });
            let slow_request = scope.spawn(|| devices.claim(&slow, &session, false));
            pending_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            let (done_tx,done_rx) = mpsc::channel();
            let d = &devices;
            let s = &session;
            scope.spawn(move || done_tx.send(d.claim(&fast,s,false)).unwrap());
            let fast_result = done_rx.recv_timeout(Duration::from_secs(1));
            finish_tx.send(()).unwrap();
            assert_eq!(fast_result.unwrap().unwrap().as_deref(), Some("cached"));
            assert!(slow_request.join().unwrap().is_err());
        });
        std::fs::remove_dir_all(dir).unwrap();
    }
}
