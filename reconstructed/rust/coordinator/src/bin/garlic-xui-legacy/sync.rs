use super::{trojan_grpc::Grant, Result};
use garlic_relay_session::{
    endpoint::NodeEndpoint,
    identity::{self, Identity, IdentityFile, KeyRing},
};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Deserialize)]
pub struct Source {
    origin: String,
    identity_file: PathBuf,
    keys_file: PathBuf,
    ca_file: PathBuf,
}

pub type Cache = Arc<RwLock<Option<Grant>>>;

fn refresh(source: &Source, target: &Path) -> Result<Grant> {
    let endpoint = NodeEndpoint::new(&source.origin)?;
    let ca = fs::read(&source.ca_file)?;
    let identity: IdentityFile = garlic_state::read(&source.identity_file)?;
    let keys: KeyRing = garlic_state::read(&source.keys_file)?;
    let credentials = identity::login_endpoint(
        &endpoint,
        &ca,
        &Identity::load(&identity)?,
        &keys,
        "XUI Legacy W",
        Duration::from_secs(8),
    )?;
    let mut client = Client::builder()
        .https_only(true)
        .tls_built_in_root_certs(false)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(8));
    for root in reqwest::Certificate::from_pem_bundle(&ca)? {
        client = client.add_root_certificate(root);
    }
    let response = endpoint
        .configure(client)
        .build()?
        .get(endpoint.url().join("/v1/infra/legacy/accounts")?)
        .bearer_auth(credentials.token)
        .send()?
        .error_for_status()?;
    let mut bytes = Vec::new();
    response.take(1_048_577).read_to_end(&mut bytes)?;
    if bytes.len() > 1_048_576 {
        return Err("legacy grant is too large".into());
    }
    let grant: Grant = serde_json::from_slice(&bytes)?;
    if u128::from(grant.valid_until_ms) <= SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    {
        return Err("legacy grant has expired".into());
    }
    let directory = target
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(directory)?;
    serde_json::to_writer(&mut file, &grant)?;
    file.flush()?;
    file.as_file().sync_all()?;
    file.persist(target)?;
    Ok(grant)
}

pub async fn run(source: Source, path: PathBuf, cache: Cache) {
    loop {
        let source = source.clone();
        let path = path.clone();
        match tokio::task::spawn_blocking(move || refresh(&source, &path)).await {
            Ok(Ok(grant)) => {
                let Ok(mut cache) = cache.write() else {
                    return;
                };
                *cache = Some(grant);
            }
            _ => eprintln!("Legacy account refresh failed"),
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
