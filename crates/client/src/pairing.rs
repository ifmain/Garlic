use crate::settings::{origin, Settings};
use garlic_state::StateError;
use reqwest::{blocking::Client, Url};
use rustls::{pki_types::CertificateDer, RootCertStore};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::Read,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const PATH: &str = "/.well-known/garlic/ca.der";
const LIMIT: usize = 16 * 1024;

#[derive(Clone)]
pub(super) struct Paired {
    pub client: Client,
    pub der: Vec<u8>,
}

#[derive(Default)]
struct Slot {
    paired: Option<Paired>,
    attempted: Option<Instant>,
}

pub(super) struct Pairing {
    settings: Settings,
    slots: Mutex<BTreeMap<String, Arc<Mutex<Slot>>>>,
    authorities: BTreeMap<String, String>,
}

impl Pairing {
    pub fn new(settings: &Settings) -> Self {
        // This directory is stored only after a successful authenticated network request.
        let directory: Option<Value> = garlic_state::read(&settings.side("network.json")).ok();
        let nodes = directory.as_ref().and_then(|d| d["nodes"].as_array());
        let mut authorities = BTreeMap::new();
        // Old saved-key profiles may only have the routing preferences. Use the
        // explicitly configured control node for that profile's own endpoints.
        if let Ok(profile) = settings.load() {
            if let Some(control) = profile
                .control_urls
                .iter()
                .filter_map(|u| Url::parse(u).ok())
                .find(|u| u.scheme() == "https" && u.domain().is_some())
            {
                for address in profile.control_urls.iter().chain(&profile.storage_urls) {
                    if let Ok(target) = origin(address) {
                        authorities.insert(target, control.to_string());
                    }
                }
            }
        }
        if let Some(nodes) = nodes {
            let entry = nodes
                .iter()
                .filter(|n| n["role"] == "ru")
                .filter_map(|n| Url::parse(n["url"].as_str()?).ok())
                .find(|u| u.scheme() == "https" && u.domain().is_some());
            if let Some(entry) = entry {
                for node in nodes {
                    if let Some(target) = node["url"].as_str().and_then(|u| origin(u).ok()) {
                        authorities.insert(target, entry.to_string());
                    }
                }
            }
        }
        Self {
            settings: settings.clone(),
            slots: Default::default(),
            authorities,
        }
    }

    fn issuer(&self, target: &str) -> Result<Url, StateError> {
        let target = origin(target)?;
        let mut url = Url::parse(self.authorities.get(&target).unwrap_or(&target))?;
        if url.scheme() != "https" {
            return Err("pairing requires HTTPS".into());
        }
        url.set_port(None).map_err(|_| "invalid enrollment port")?;
        url.set_path(PATH);
        Ok(url)
    }

    pub fn resolve(
        &self,
        target: &str,
        fetch: bool,
        discovery: &Client,
        build: impl Fn(&[u8]) -> Result<Client, StateError>,
    ) -> Result<Option<Paired>, StateError> {
        if !target.starts_with("https://") {
            return Ok(None);
        }
        let issuer = self.issuer(target)?;
        let slot = self
            .slots
            .lock()
            .unwrap()
            .entry(issuer.to_string())
            .or_default()
            .clone();
        let mut state = slot.lock().unwrap();
        if !fetch {
            if state.paired.is_none() {
                let file = self.settings.side("pairing.json");
                if file.try_exists()? {
                    let book: BTreeMap<String, Vec<u8>> = garlic_state::read(&file)?;
                    if let Some(der) = book.get(issuer.as_str()) {
                        state.paired = Some(Paired {
                            client: build(der)?,
                            der: der.clone(),
                        });
                    }
                }
            }
            return Ok(state.paired.clone());
        }
        if state
            .attempted
            .is_some_and(|t| t.elapsed() < Duration::from_secs(10))
        {
            return state
                .paired
                .clone()
                .map(Some)
                .ok_or_else(|| "network CA enrollment temporarily unavailable".into());
        }
        state.attempted = Some(Instant::now());
        // No Authorization or device credentials. This client uses the normal TLS roots
        // and does not follow redirects or trust CAs downloaded through this endpoint.
        let response = discovery
            .get(issuer.clone())
            .timeout(Duration::from_secs(6))
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("HTTPS network CA enrollment failed: {e}"))?;
        let mut der = Vec::new();
        response.take(LIMIT as u64 + 1).read_to_end(&mut der)?;
        if der.is_empty() || der.len() > LIMIT {
            return Err("invalid network CA length".into());
        }
        RootCertStore::empty().add(CertificateDer::from(der.as_slice()))?;
        let client = build(&der)?;
        garlic_state::edit_or_create(
            &self.settings.side("pairing.json"),
            || Ok(BTreeMap::<String, Vec<u8>>::new()),
            |book| {
                book.insert(issuer.to_string(), der.clone());
                Ok(())
            },
        )?;
        state.paired = Some(Paired { client, der });
        Ok(state.paired.clone())
    }
}

pub(super) fn unknown_issuer(e: &reqwest::Error) -> bool {
    if !e.is_connect() {
        return false;
    }
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(e);
    while let Some(err) = cause {
        if matches!(
            err.downcast_ref::<rustls::Error>(),
            Some(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer
            ))
        ) {
            return true;
        }
        // hyper-rustls wraps the TLS error in io::Error, whose source() skips
        // the contained error itself on some platforms.
        if let Some(inner) = err
            .downcast_ref::<std::io::Error>()
            .and_then(|e| e.get_ref())
        {
            if matches!(
                inner.downcast_ref::<rustls::Error>(),
                Some(rustls::Error::InvalidCertificate(
                    rustls::CertificateError::UnknownIssuer
                ))
            ) {
                return true;
            }
        }
        // Some reqwest TLS backends erase the concrete rustls error type.
        if err
            .to_string()
            .ends_with("invalid peer certificate: UnknownIssuer")
        {
            return true;
        }
        cause = err.source();
    }
    false
}
