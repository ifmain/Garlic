use crate::{accounts::now, http::Shared, network::Observation};
use garlic_relay_data::control::UserSnapshot;
use garlic_relay_session::{
    endpoint::NodeEndpoint,
    identity::{self, Grant, Identity, IdentityFile, KeyRing},
    SessionError,
};
use reqwest::{blocking::Client, Method, Url};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    env, fs,
    io::Read,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

pub struct Upstream {
    origin: Url,
    ca: Vec<u8>,
    client: Client,
    identity: PathBuf,
    keys: PathBuf,
    label: String,
    accounts: bool,
    network: bool,
}
fn trusted_roots(ca: &[u8]) -> Result<Vec<reqwest::Certificate>, SessionError> {
    let roots = reqwest::Certificate::from_pem_bundle(ca)?;
    if roots.is_empty() {
        return Err("missing upstream trust anchor".into());
    }
    Ok(roots)
}

pub struct AccountAuthority {
    origin: Url,
    client: reqwest::Client,
}
pub(crate) fn node_client(
    ca: &[u8],
    timeout: Duration,
    endpoint: &NodeEndpoint,
) -> Result<Client, SessionError> {
    let mut builder = Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .tls_built_in_root_certs(false)
        .timeout(timeout);
    for root in trusted_roots(ca)? {
        builder = builder.add_root_certificate(root);
    }
    Ok(endpoint.configure(builder).build()?)
}
impl AccountAuthority {
    pub fn new(url: &str, ca: &[u8]) -> Result<Self, SessionError> {
        let mut client = reqwest::Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .tls_built_in_root_certs(false)
            .timeout(Duration::from_secs(8));
        for root in trusted_roots(ca)? {
            client = client.add_root_certificate(root);
        }
        Ok(Self {
            origin: NodeEndpoint::new(url)?.url().clone(),
            client: client.build()?,
        })
    }
    /// Only fixed account API paths can receive a user's bearer token.
    pub(crate) async fn request(
        &self,
        path: &'static str,
        bearer: &str,
        body: Option<Value>,
    ) -> Result<Value, crate::http::Failure> {
        use crate::http::Failure;
        use axum::http::StatusCode;
        let unavailable = || {
            Failure(
                StatusCode::SERVICE_UNAVAILABLE,
                "account_authority_unavailable".into(),
            )
        };
        if !matches!(
            path,
            "v1/auth/users"
                | "v1/auth/login"
                | "v1/auth/logout"
                | "v1/auth/device/check"
                | "v1/auth/device/claim"
                | "v1/auth/device/renew"
                | "v1/auth/device/release"
        ) {
            return Err(unavailable());
        }
        let url = self.origin.join(path).map_err(|_| unavailable())?;
        let mut request = if body.is_some() {
            self.client.post(url)
        } else {
            self.client.get(url)
        };
        if let Some(v) = body {
            request = request.json(&v);
        }
        if !bearer.is_empty() {
            request = request.bearer_auth(bearer);
        }
        let mut response = request
            .send()
            .await
            .map_err(|_| unavailable())?;
        let status = response.status();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| unavailable())? {
            if bytes.len() + chunk.len() > 4_194_304 {
                return Err(unavailable());
            }
            bytes.extend_from_slice(&chunk);
        }
        if !status.is_success() {
            let v: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
            return Err(Failure(
                if status.is_client_error() {
                    status
                } else {
                    StatusCode::SERVICE_UNAVAILABLE
                },
                v["error"]
                    .as_str()
                    .unwrap_or("account_authority_declined")
                    .chars()
                    .take(160)
                    .collect(),
            ));
        }
        serde_json::from_slice(&bytes).map_err(|_| unavailable())
    }
}
impl Upstream {
    pub fn new(
        url: &str,
        ca: &[u8],
        identity: PathBuf,
        keys: PathBuf,
        label: String,
        accounts: bool,
        network: bool,
    ) -> Result<Self, SessionError> {
        let endpoint = NodeEndpoint::new(url)?;
        Ok(Self {
            origin: endpoint.url().clone(),
            client: node_client(ca, Duration::from_secs(8), &endpoint)?,
            ca: ca.to_vec(),
            identity,
            keys,
            label,
            accounts,
            network,
        })
    }
    fn fetch<T: DeserializeOwned>(
        &self,
        path: &str,
        token: &str,
        body: Option<Value>,
    ) -> Result<T, SessionError> {
        let mut request = self
            .client
            .request(
                if body.is_some() {
                    Method::POST
                } else {
                    Method::GET
                },
                self.origin.join(path)?,
            )
            .bearer_auth(token);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send()?.error_for_status()?;
        let mut bytes = vec![];
        response.take(8_388_609).read_to_end(&mut bytes)?;
        if bytes.len() > 8_388_608 {
            return Err("upstream response exceeded snapshot limit".into());
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
    fn refresh(&self, app: &Shared, grant: &mut Option<Grant>) -> Result<(), SessionError> {
        if grant.as_ref().is_none_or(|g| g.expires_at <= now() + 30) {
            let id: IdentityFile = serde_json::from_slice(&fs::read(&self.identity)?)?;
            let ring: KeyRing = serde_json::from_slice(&fs::read(&self.keys)?)?;
            *grant = Some(identity::login(
                self.origin.as_str(),
                &self.ca,
                &Identity::load(&id)?,
                &ring,
                &self.label,
                Duration::from_secs(8),
            )?);
        }
        let grant = grant.as_ref().unwrap();
        if self.accounts {
            let snapshot: UserSnapshot = self.fetch("v1/infra/users/sync", &grant.token, None)?;
            app.accounts
                .import(self.origin.as_str(), &grant.authority, snapshot)?;
        }
        if self.network {
            let directory = self.fetch(
                "v1/infra/network/register",
                &grant.token,
                Some(serde_json::to_value(app.network.local_node())?),
            )?;
            app.network.import(directory)?;
            let observation = Observation {
                last_seen: now(),
                active_streams: app.hub.active_streams() as u64,
                timeout_count: 0,
                execution_capacity: app.network.capacity()? as u64,
            };
            self.client
                .post(self.origin.join("v1/infra/network/observation")?)
                .bearer_auth(&grant.token)
                .json(&observation)
                .send()?
                .error_for_status()?;
        }
        Ok(())
    }
    pub fn run(self, app: Shared, stop: Arc<AtomicBool>) {
        let interval = env::var("GARLIC_USER_SYNC_INTERVAL_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(5000);
        let mut grant = None;
        while !stop.load(Ordering::Acquire) {
            if let Err(err) = self.refresh(&app, &mut grant) {
                eprintln!("Upstream synchronization failed: {err}");
                if err
                    .downcast_ref::<reqwest::Error>()
                    .is_some_and(|e| e.status() == Some(reqwest::StatusCode::UNAUTHORIZED))
                {
                    grant = None;
                }
            }
            let mut remaining = interval;
            while remaining > 0 && !stop.load(Ordering::Acquire) {
                let step = remaining.min(500);
                thread::sleep(Duration::from_millis(step));
                remaining -= step;
            }
        }
    }
}
