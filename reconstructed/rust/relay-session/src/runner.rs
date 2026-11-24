//! Authenticated Relay v2 worker using the provisioned Garlic service environment.
use crate::{
    connect_at, destination,
    endpoint::NodeEndpoint,
    exit::{FlowPool, Worker},
    identity, SessionError,
};
use garlic_relay_data::control::{Authority, Capability, Hello};
use std::{
    env, fs,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

#[derive(Clone)]
pub struct WorkerSettings {
    pub endpoint: NodeEndpoint,
    ca: String,
    identity_file: String,
    keys: String,
    label: String,
    capacity: u64,
    connections: usize,
    loopback_fixture: bool,
}

struct PoolContext {
    owner: Authority,
    node: String,
    flows: FlowPool,
}

impl WorkerSettings {
    pub fn load() -> Result<Self, SessionError> {
        let origin =
            env::var("GARLIC_DNS_URL").map_err(|_| "missing service setting GARLIC_DNS_URL")?;
        Self::for_endpoint(NodeEndpoint::new(&origin)?)
    }

    pub fn for_endpoint(endpoint: NodeEndpoint) -> Result<Self, SessionError> {
        let get =
            |name: &str| env::var(name).map_err(|_| format!("missing service setting {name}"));
        let capacity = env::var("GARLIC_RELAY_MAX_STREAMS")
            .unwrap_or_else(|_| "512".into())
            .parse()?;
        if !(1..=1024).contains(&capacity) {
            return Err("GARLIC_RELAY_MAX_STREAMS must be between 1 and 1024".into());
        }
        let connections = env::var("GARLIC_RELAY_CONNECTIONS")
            .unwrap_or_else(|_| "1".into())
            .parse()?;
        if !(1..=8).contains(&connections) {
            return Err("GARLIC_RELAY_CONNECTIONS must be in 1..=8".into());
        }
        Ok(Self {
            endpoint,
            ca: get("GARLIC_CA_CERT")?,
            identity_file: get("GARLIC_NODE_IDENTITY_FILE")?,
            keys: get("GARLIC_INFRA_KEYS_FILE")?,
            label: get("GARLIC_NODE_LABEL")?,
            capacity,
            connections,
            loopback_fixture: env::var("GARLIC_DEV_MODE").as_deref() == Ok("1")
                && env::var("GARLIC_DEV_ALLOW_PRIVATE_PROXY_TARGETS").as_deref() == Ok("1"),
        })
    }

    fn session(
        &self,
        stop: &AtomicBool,
        shared: &Mutex<Option<PoolContext>>,
    ) -> Result<(), SessionError> {
        let ca = fs::read(&self.ca)?;
        let node =
            identity::Identity::load(&serde_json::from_slice(&fs::read(&self.identity_file)?)?)?;
        let keys = serde_json::from_slice(&fs::read(&self.keys)?)?;
        let timeout = Duration::from_secs(15);
        let grant =
            identity::login_endpoint(&self.endpoint, &ca, &node, &keys, &self.label, timeout)?;
        let hello = Hello {
            protocol_version: 1,
            worker_id: uuid::Uuid::new_v4(),
            node_id: node.node_id,
            capabilities: Capability::ALL.to_vec(),
            max_streams: self.capacity,
        };
        let mut channel = connect_at(
            &self.endpoint.url().as_str().parse()?,
            self.endpoint.address,
            &ca,
            &grant.token,
            &grant.authority,
            &hello,
            timeout,
        )?;
        let pool = {
            let mut pool = shared.lock().unwrap();
            if pool
                .as_ref()
                .is_none_or(|p| p.owner != grant.authority || p.node != hello.node_id)
            {
                *pool = Some(PoolContext {
                    owner: grant.authority.clone(),
                    node: hello.node_id.clone(),
                    flows: FlowPool::new(
                        grant.authority.clone(),
                        self.capacity as usize * self.connections,
                        Duration::from_secs(60),
                    ),
                });
            }
            pool.as_ref().unwrap().flows.clone()
        };
        println!("Authenticated worker ready for {} streams", self.capacity);
        let mut worker = Worker::with_pool(pool, hello.worker_id, self.capacity as usize, |open| {
            destination::select(&open.connect_authority, self.loopback_fixture)
        });
        worker.run_until(&mut channel, || stop.load(Ordering::Relaxed))?;
        Ok(())
    }
}

pub fn run(settings: WorkerSettings, stop: Arc<AtomicBool>) -> Result<(), SessionError> {
    let pool: Mutex<Option<PoolContext>> = Mutex::new(None);
    thread::scope(|threads| {
        for _ in 0..settings.connections {
            threads.spawn(|| {
                while !stop.load(Ordering::Relaxed) {
                    if let Some(context) = pool.lock().unwrap().as_ref() {
                        context.flows.expire();
                    }
                    if let Err(err) = settings.session(&stop, &pool) {
                        eprintln!("Relay session ended: {err}");
                    }
                    if !stop.load(Ordering::Relaxed) {
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            });
        }
    });
    Ok(())
}
