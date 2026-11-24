use super::model::{Operation, Status, Task};
use super::transfers::Work;
use base64::{engine::general_purpose::STANDARD, Engine};
use garlic_relay_session::{
    endpoint::NodeEndpoint,
    identity::{self, Grant, Identity, KeyRing},
    SessionError,
};
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    Method, Url,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env, fs,
    io::Read,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use uuid::Uuid;

const MAX_BYTES: usize = 1 << 20;
fn bytes(response: Response, limit: usize) -> Result<Vec<u8>, SessionError> {
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err("response too large".into());
    }
    let mut b = Vec::new();
    response.take(limit as u64 + 1).read_to_end(&mut b)?;
    if b.len() > limit {
        return Err("response too large".into());
    }
    Ok(b)
}
#[derive(Deserialize)]
struct Exchange {
    service: String,
    method: String,
    path: String,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    #[serde(default)]
    body_base64: String,
}
/// Service origins are configured on this node. Tasks supply only a logical
/// service name and a relative request path, as in the recovered 0.3 sources.
fn services() -> Result<BTreeMap<String, Url>, SessionError> {
    let mut out = BTreeMap::new();
    for item in env::var("GARLIC_SERVICES")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
    {
        let (name, address) = item.split_once('=').ok_or("invalid configured service")?;
        let url = Url::parse(address)?;
        let loopback = url.host_str().is_some_and(|h| {
            h == "localhost"
                || h.trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
        if name.is_empty()
            || url.host().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
        {
            return Err("invalid configured service origin".into());
        }
        out.insert(name.to_owned(), url);
    }
    Ok(out)
}
struct Pool {
    endpoint: NodeEndpoint,
    ca: Vec<u8>,
    identity: Identity,
    keys: KeyRing,
    label: String,
    name: String,
    capacity: usize,
    services: BTreeMap<String, Url>,
    control: Client,
    http: Client,
    grant: Mutex<Option<Grant>>,
}
impl Pool {
    fn new(endpoint: NodeEndpoint) -> Result<Self, SessionError> {
        let ca = fs::read(env::var("GARLIC_CA_CERT")?)?;
        let control = crate::upstream::node_client(&ca, Duration::from_secs(12), &endpoint)?;
        let http = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(12))
            .build()?;
        let identity = Identity::load(&crate::files::read(&std::path::PathBuf::from(env::var(
            "GARLIC_NODE_IDENTITY_FILE",
        )?))?)?;
        let keys = crate::files::read(&std::path::PathBuf::from(env::var(
            "GARLIC_INFRA_KEYS_FILE",
        )?))?;
        let label = env::var("GARLIC_NODE_LABEL")?;
        let capacity = env::var("GARLIC_WORKERS")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(40usize)
            .clamp(1, 40);
        let name = env::var("GARLIC_STORAGE_NAME").unwrap_or_else(|_| "storage-a".into());
        Ok(Self {
            endpoint,
            ca,
            identity,
            keys,
            label,
            name,
            capacity,
            services: services()?,
            control,
            http,
            grant: Default::default(),
        })
    }
    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, SessionError> {
        let mut held = self
            .grant
            .lock()
            .map_err(|_| "worker credential state unavailable")?;
        if held
            .as_ref()
            .is_none_or(|g| g.expires_at <= crate::accounts::now() + 15)
        {
            *held = Some(identity::login_endpoint(
                &self.endpoint,
                &self.ca,
                &self.identity,
                &self.keys,
                &self.label,
                Duration::from_secs(8),
            )?);
        }
        let grant = held.as_ref().unwrap();
        Ok(self
            .control
            .request(method, self.endpoint.url().join(path)?)
            .bearer_auth(&grant.token)
            .header("x-garlic-owner-id", grant.authority.owner_id.to_string())
            .header("x-garlic-owner-epoch", grant.authority.epoch))
    }
    fn post(&self, path: &str, body: &Value) -> Result<Value, SessionError> {
        let response = self.request(Method::POST, path)?.json(body).send()?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            *self
                .grant
                .lock()
                .map_err(|_| "worker credentials unavailable")? = None;
        }
        let raw = bytes(response.error_for_status()?, MAX_BYTES * 2)?;
        Ok(if raw.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&raw)?
        })
    }
    fn exchange(&self, input: Option<&Value>) -> Result<Value, SessionError> {
        let x: Exchange =
            serde_json::from_value(input.cloned().ok_or("missing exchange specification")?)?;
        let base = self
            .services
            .get(&x.service)
            .ok_or("service not configured on this node")?;
        if !x.path.starts_with('/')
            || x.path.starts_with("//")
            || x.path.contains(['#', '\\'])
            || x.path.chars().any(char::is_control)
        {
            return Err("invalid relative path".into());
        }
        let url = Url::parse(&format!(
            "{}{}",
            base.as_str().trim_end_matches('/'),
            x.path
        ))?;
        if url.origin() != base.origin() {
            return Err("invalid relative path".into());
        }
        let method = Method::from_bytes(x.method.as_bytes())?;
        if !matches!(
            method,
            Method::GET
                | Method::POST
                | Method::PUT
                | Method::PATCH
                | Method::DELETE
                | Method::HEAD
        ) {
            return Err("method not allowed".into());
        }
        let data = STANDARD.decode(x.body_base64)?;
        if data.len() > MAX_BYTES {
            return Err("request too large".into());
        }
        let mut req = self.http.request(method, url).body(data);
        for (key, val) in x.headers {
            if matches!(
                key.to_ascii_lowercase().as_str(),
                "content-type"
                    | "accept"
                    | "accept-language"
                    | "if-none-match"
                    | "if-modified-since"
            ) {
                req = req.header(key, val);
            }
        }
        let res = req.send().map_err(|_| "upstream request failed")?;
        let status = res.status().as_u16();
        let mut headers = BTreeMap::new();
        for k in ["content-type", "etag", "last-modified"] {
            if let Some(v) = res.headers().get(k).and_then(|h| h.to_str().ok()) {
                headers.insert(k, v.to_owned());
            }
        }
        Ok(
            json!({"status":status,"headers":headers,"body_base64":STANDARD.encode(bytes(res,MAX_BYTES)?)}),
        )
    }
    fn execute(
        &self,
        task: &Task,
        channel: Uuid,
        worker: &str,
        lease: Uuid,
    ) -> Result<(Option<String>, Option<Value>), SessionError> {
        let text = match task.operation {
            Operation::Echo => task.input.clone(),
            Operation::Uppercase => task.input.to_uppercase(),
            Operation::Reverse => task.input.chars().rev().collect(),
            Operation::Http => return Ok((None, Some(self.exchange(task.exchange.as_ref())?))),
            Operation::Transfer => {
                let work: Work = serde_json::from_str(&task.input)?;
                let base = format!("/v1/infinity/transfers/{}", work.session_id);
                let authorization = json!({"task_id":task.task_id,"worker_id":worker,"channel_id":channel,
                    "lease_id":lease,"offset":work.offset});
                let res = self
                    .request(Method::POST, &format!("{base}/read"))?
                    .json(&authorization)
                    .send()?
                    .error_for_status()?;
                let part = bytes(res, 262144)?;
                self.request(Method::PUT, &format!("{base}/received/{}", work.offset))?
                    .header("x-task-id", task.task_id.to_string())
                    .header("x-channel-id", channel.to_string())
                    .header("x-worker-id", worker)
                    .header("x-lease-id", lease.to_string())
                    .header("content-type", "application/octet-stream")
                    .body(part)
                    .send()?
                    .error_for_status()?;
                "part_received".into()
            }
        };
        Ok((Some(text), None))
    }
    fn channel(
        &self,
        index: usize,
        target: &AtomicUsize,
        stop: &AtomicBool,
    ) -> Result<(), SessionError> {
        let name = format!("{}-{}-{index}", self.name, self.label);
        let open = json!({"protocol_version":1,"worker_id":name,"node_id":self.identity.node_id,
            "services":self.services.keys().collect::<Vec<_>>(),"reserve":false,
            "operations":["echo","uppercase","reverse","http","transfer"],"capabilities":garlic_relay_data::control::Capability::ALL,
            "capacity":self.capacity});
        let reply = self.post("/v1/infinity/channels", &open)?;
        let channel: Uuid = serde_json::from_value(reply["channel_id"].clone())?;
        let root = format!("/v1/infinity/channels/{channel}");
        let owner = json!({"worker_id":name});
        self.post(&format!("{root}/ready"), &owner)?;
        while !stop.load(Ordering::Acquire) && index < target.load(Ordering::Acquire) {
            let reply = self.post(&format!("{root}/claim"), &owner)?;
            let Some(task) = reply.get("task").filter(|t| !t.is_null()) else {
                thread::sleep(Duration::from_millis(300));
                continue;
            };
            let task: Task = serde_json::from_value(task.clone())?;
            let lease: Uuid = serde_json::from_value(reply["lease_id"].clone())?;
            let (status, result, exchange, error) = match self.execute(&task, channel, &name, lease)
            {
                Ok((result, exchange)) => (Status::Completed, result, exchange, None),
                Err(e) => (Status::Failed, None, None, Some(e.to_string())),
            };
            let complete = json!({"worker_id":name,"task_id":task.task_id,"lease_id":lease,"result":{
                "task":task,"attempts":1,"status":status,"result":result,"exchange":exchange,"error":error}});
            let path = format!("{root}/complete");
            // Retry only the acknowledgement. Repeating a logical HTTP task can change remote state.
            let mut attempts = 0;
            loop {
                match self.post(&path, &complete) {
                    Ok(_) => break,
                    Err(e) if attempts >= 2 => return Err(e),
                    Err(_) => {
                        attempts += 1;
                        thread::sleep(Duration::from_millis(250));
                    }
                }
            }
        }
        let _ = self.post(&format!("{root}/drain"), &owner);
        Ok(())
    }
}
pub fn run(endpoint: NodeEndpoint, stop: Arc<AtomicBool>) -> Result<(), SessionError> {
    let pool = Pool::new(endpoint)?;
    let target = AtomicUsize::new(pool.capacity.min(4));
    thread::scope(|group| {
        let mut workers: Vec<Option<thread::ScopedJoinHandle<'_, ()>>> =
            (0..pool.capacity).map(|_| None).collect();
        while !stop.load(Ordering::Acquire) {
            let limits = (|| -> Result<Value, SessionError> {
                let response = pool
                    .request(Method::GET, "/v1/infinity/stats")?
                    .send()?
                    .error_for_status()?;
                Ok(serde_json::from_slice(&bytes(response, 65536)?)?)
            })();
            if let Ok(limits) = limits {
                if let Some(n) = limits["target_active"].as_u64() {
                    target.store((n.min(pool.capacity as u64)) as usize, Ordering::Release);
                }
            }
            for (index, slot) in workers.iter_mut().enumerate() {
                if slot.as_ref().is_some_and(|t| t.is_finished()) {
                    if let Some(t) = slot.take() {
                        let _ = t.join();
                    }
                }
                if index < target.load(Ordering::Acquire) && slot.is_none() {
                    let (pool, target, stop) = (&pool, &target, &stop);
                    *slot = Some(group.spawn(move || {
                        if let Err(e) = pool.channel(index, target, stop) {
                            eprintln!("Task worker disconnected: {e}");
                        }
                    }));
                }
            }
            for _ in 0..10 {
                if stop.load(Ordering::Acquire) {
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
        }
    });
    Ok(())
}
