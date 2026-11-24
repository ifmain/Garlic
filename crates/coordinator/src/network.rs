use crate::{
    accounts::now,
    files,
    http::{bearer, blocking, Failure, Shared},
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use garlic_relay_session::{
    identity::{Identity, IdentityFile},
    SessionError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    env, io,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    node_id: String,
    label: String,
    role: String,
    url: String,
    weight: u32,
}
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Policy {
    pub stream_limit: u32,
    pub worker_limit: u32,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            stream_limit: 2048,
            worker_limit: 40,
        }
    }
}
#[derive(Deserialize, Serialize)]
struct Directory {
    version: u32,
    network_id: String,
    nodes: Vec<Node>,
    #[serde(default)]
    policies: BTreeMap<String, Policy>,
    #[serde(default)]
    observations: BTreeMap<String, Value>,
    #[serde(flatten)]
    extra: BTreeMap<String, Value>,
}
pub struct Network {
    path: PathBuf,
    node: Node,
    id: String,
}
impl Network {
    pub fn capacity(&self) -> Result<u32, SessionError> {
        let dir: Directory = files::read(&self.path)?;
        Ok(dir
            .policies
            .get(&self.node.node_id)
            .copied()
            .unwrap_or_default()
            .worker_limit)
    }
    pub fn import(&self, data: Value) -> Result<(), SessionError> {
        let upstream: Directory = serde_json::from_value(data)?;
        if upstream.version != 1
            || upstream.network_id != self.id
            || !upstream
                .nodes
                .iter()
                .any(|n| n.node_id == self.node.node_id)
        {
            return Err("upstream directory does not include the registered node".into());
        }
        files::edit(&self.path, |dir: &mut Directory| {
            dir.nodes = upstream.nodes;
            dir.policies = upstream.policies;
            dir.observations = upstream.observations;
            Ok(())
        })
    }
    pub fn local_node(&self) -> &Node {
        &self.node
    }
    pub fn from_env(state: &Path) -> Result<Self, SessionError> {
        let identity = env::var_os("GARLIC_NODE_IDENTITY_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| state.with_extension("identity.json"));
        let node_id = files::edit_or_create(&identity, IdentityFile::generate, |file| {
            Ok(Identity::load(file)?.node_id)
        })?;
        let network = Self {
            path: env::var_os("GARLIC_NETWORK_FILE")
                .map(PathBuf::from)
                .unwrap_or_else(|| state.with_extension("network.json")),
            id: env::var("GARLIC_NETWORK_ID").unwrap_or_else(|_| "garlic".into()),
            node: Node {
                node_id,
                label: env::var("GARLIC_NODE_LABEL").unwrap_or_else(|_| "coordinator".into()),
                role: env::var("GARLIC_NODE_ROLE").unwrap_or_else(|_| "exit".into()),
                url: env::var("GARLIC_PUBLIC_URL").or_else(|_| env::var("GARLIC_DNS_URL"))?,
                weight: env::var("GARLIC_NODE_WEIGHT")
                    .ok()
                    .map(|n| n.parse::<u32>())
                    .transpose()?
                    .unwrap_or(1),
            },
        };
        files::edit_or_create(
            &network.path,
            || {
                Ok(Directory {
                    version: 1,
                    network_id: network.id.clone(),
                    nodes: vec![],
                    policies: Default::default(),
                    observations: Default::default(),
                    extra: Default::default(),
                })
            },
            |dir| {
                if dir.version != 1 || dir.network_id != network.id {
                    return Err(
                        "network directory belongs to a different network or version".into(),
                    );
                }
                if let Some(node) = dir
                    .nodes
                    .iter_mut()
                    .find(|n| n.node_id == network.node.node_id)
                {
                    *node = network.node.clone();
                } else {
                    dir.nodes.push(network.node.clone());
                }
                dir.policies
                    .entry(network.node.node_id.clone())
                    .or_default();
                Ok(())
            },
        )?;
        Ok(network)
    }
    pub(crate) fn view(&self, active: usize) -> Result<Value, SessionError> {
        let mut dir: Directory = files::read(&self.path)?;
        let capacity = dir
            .policies
            .get(&self.node.node_id)
            .copied()
            .unwrap_or_default()
            .worker_limit;
        dir.observations.insert(self.node.node_id.clone(),json!({"last_seen":now(),"active_streams":active,"timeout_count":0,"execution_capacity":capacity}));
        Ok(serde_json::to_value(dir)?)
    }
}
pub(crate) fn identity(app: &Shared, token: &str) -> Result<String, SessionError> {
    Ok(garlic_relay_session::auth::verify_node(
        &app.accounts.accounts,
        &app.accounts.keys,
        &app.accounts.owner,
        token,
        now(),
    )?
    .node_id)
}
pub async fn register(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(node): Json<Node>,
) -> Result<Json<Value>, Failure> {
    let token = bearer(&headers)?.to_owned();
    Ok(Json(
        blocking(move || {
            let caller = identity(&app, &token)?;
            if caller != node.node_id {
                return Err(crate::accounts::denied());
            }
            let url = reqwest::Url::parse(&node.url).ok();
            if !matches!(node.role.as_str(), "ru" | "exit")
                || node.weight == 0
                || node.label.is_empty()
                || node.label.len() > 255
                || node.label.chars().any(char::is_control)
                || !url.is_some_and(|u| {
                    u.scheme() == "https"
                        && u.host().is_some()
                        && u.username().is_empty()
                        && u.password().is_none()
                        && u.query().is_none()
                        && u.fragment().is_none()
                        && u.path() == "/"
                })
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid network registration",
                )
                .into());
            }
            files::edit(&app.network.path, |dir: &mut Directory| {
                dir.policies.entry(caller.clone()).or_default();
                if let Some(existing) = dir.nodes.iter_mut().find(|n| n.node_id == caller) {
                    *existing = node;
                } else {
                    dir.nodes.push(node);
                }
                Ok(())
            })?;
            app.network.view(app.hub.active_streams())
        })
        .await?,
    ))
}
#[derive(Deserialize, Serialize)]
pub struct Observation {
    pub last_seen: u64,
    pub active_streams: u64,
    pub timeout_count: u64,
    pub execution_capacity: u64,
}
pub async fn observation(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(mut obs): Json<Observation>,
) -> Result<StatusCode, Failure> {
    let token = bearer(&headers)?.to_owned();
    blocking(move || {
        let id = identity(&app, &token)?;
        files::edit(&app.network.path, |dir: &mut Directory| {
            if !dir.nodes.iter().any(|n| n.node_id == id) {
                return Err(crate::accounts::denied());
            }
            obs.last_seen = now();
            dir.observations.insert(id, serde_json::to_value(obs)?);
            Ok(())
        })
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
pub async fn get(State(app): State<Shared>, headers: HeaderMap) -> Result<Json<Value>, Failure> {
    let token = bearer(&headers)?.to_owned();
    Ok(Json(
        blocking(move || {
            app.accounts.user(&token)?;
            app.network.view(app.hub.active_streams())
        })
        .await?,
    ))
}
#[derive(Deserialize)]
pub struct Update {
    node_id: String,
    #[serde(flatten)]
    policy: Policy,
}
pub async fn set_policy(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(body): Json<Update>,
) -> Result<Json<Value>, Failure> {
    let token = bearer(&headers)?.to_owned();
    let changed = blocking(move || {
        app.accounts.require_admin(&token)?;
        let policy = body.policy;
        if !(1..=4096).contains(&policy.stream_limit) || !(1..=4096).contains(&policy.worker_limit)
        {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "invalid network policy").into(),
            );
        }
        files::edit(&app.network.path, |dir: &mut Directory| {
            if !dir.nodes.iter().any(|n| n.node_id == body.node_id) {
                return Ok(false);
            }
            dir.policies.insert(body.node_id, policy);
            Ok(true)
        })
    })
    .await?;
    if !changed {
        return Err(Failure(
            StatusCode::NOT_FOUND,
            "unknown network node".into(),
        ));
    }
    Ok(Json(json!({"ok":true})))
}
