use super::{digest_file, now, verify, Cache, Envelope, ROOT_CA_DER};
use garlic_state::StateError;
use reqwest::{
    blocking::{Client, Response},
    Url,
};
use serde_json::{json, Value};
use std::{
    env,
    io::{self, Read},
    time::Duration,
};

pub struct Source {
    pub origin: String,
    pub token: String,
}

impl Cache {
    /// A slower concurrent fetch may finish after a newer release has won.
    pub fn ingest(&self, envelope: Envelope, input: impl Read) -> Result<bool, StateError> {
        let rel = verify(&envelope, self.target)?;
        if rel.expires <= now() {
            return Err("release has expired".into());
        }
        if self.edit(|s| Ok(s.highest >= rel.sequence))? {
            return Ok(false);
        }
        let mut file = tempfile::NamedTempFile::new_in(&self.dir)?;
        let count = io::copy(&mut input.take(rel.size + 1), file.as_file_mut())?;
        if count != rel.size {
            return Err("incorrect release size".into());
        }
        file.as_file().sync_all()?;
        digest_file(file.path(), &rel)?;
        self.edit(|state| {
            if state.highest >= rel.sequence {
                return Ok(false);
            }
            file.persist(self.path(&rel))?;
            #[cfg(unix)]
            std::fs::File::open(&self.dir)?.sync_all()?;
            state.highest = rel.sequence;
            state.pending = Some(envelope);
            Ok(true)
        })
    }
}

fn request(client: &Client, source: &Source, name: &str) -> Result<Response, StateError> {
    let mut url = Url::parse(&source.origin)?;
    let local = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        None => false,
    };
    if (url.scheme() != "https" && !(local && url.scheme() == "http"))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err("invalid release origin".into());
    }
    url.set_path(&format!("/v1/auth/updates/{name}"));
    Ok(client
        .get(url)
        .bearer_auth(&source.token)
        .send()?
        .error_for_status()?)
}

pub fn stage(cache: &Cache, sources: &[Source]) -> Result<Value, StateError> {
    let mut builder = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(45))
        .add_root_certificate(reqwest::Certificate::from_der(ROOT_CA_DER)?);
    if let Some(path) = env::var_os("GARLIC_CA_CERT") {
        for cert in reqwest::Certificate::from_pem_bundle(&std::fs::read(path)?)? {
            builder = builder.add_root_certificate(cert);
        }
    }
    let client = builder.build()?;
    for source in sources {
        let attempt = || -> Result<bool, StateError> {
            let mut bytes = vec![];
            request(&client, source, cache.target.manifest)?
                .take(65537)
                .read_to_end(&mut bytes)?;
            if bytes.len() > 65536 {
                return Err("release manifest is too large".into());
            }
            let envelope: Envelope = serde_json::from_slice(&bytes)?;
            let rel = verify(&envelope, cache.target)?;
            if rel.expires <= now() || cache.edit(|s| Ok(s.highest >= rel.sequence))? {
                return Ok(false);
            }
            cache.ingest(envelope, request(&client, source, &rel.sha256)?)
        };
        if let Ok(true) = attempt() {
            return Ok(json!({"staged":true}));
        }
    }
    Ok(json!({"staged":false}))
}
