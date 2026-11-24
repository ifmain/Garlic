use garlic_relay_session::SessionError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, fmt, time::SystemTime};
use uuid::Uuid;

pub(crate) fn millis() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
#[derive(Debug, Clone, Copy)]
pub enum QueueError {
    Missing,
    Conflict,
    Invalid,
    Unsupported,
    Capacity,
    Forbidden,
}
impl fmt::Display for QueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Missing => "not found",
            Self::Conflict => "state or ownership conflict",
            Self::Invalid => "invalid request",
            Self::Unsupported => "unsupported Infinity protocol version",
            Self::Capacity => "Infinity capacity exhausted",
            Self::Forbidden => "credential scope forbidden",
        })
    }
}
impl std::error::Error for QueueError {}
pub type Result<T> = std::result::Result<T, SessionError>;
pub type Extra = BTreeMap<String, Value>;

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Echo,
    Uppercase,
    Reverse,
    Http,
    Transfer,
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub operation: Operation,
    #[serde(default)]
    pub input: String,
    #[serde(default)]
    pub exchange: Option<Value>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default = "Uuid::new_v4")]
    pub task_id: Uuid,
}
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Leased,
    Completed,
    Failed,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Record {
    pub task: Task,
    #[serde(default)]
    pub owner_id: Option<Uuid>,
    pub status: Status,
    pub result: Option<String>,
    pub exchange: Option<Value>,
    pub error: Option<String>,
    pub attempts: u32,
    #[serde(flatten)]
    pub extra: Extra,
}
impl Record {
    pub fn new(task: Task, owner: Uuid) -> Self {
        Self {
            task,
            owner_id: Some(owner),
            status: Status::Pending,
            result: None,
            exchange: None,
            error: None,
            attempts: 0,
            extra: Extra::new(),
        }
    }
    pub fn active(&self) -> bool {
        matches!(self.status, Status::Pending | Status::Leased)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Lease {
    pub task_id: Uuid,
    pub channel: Uuid,
    pub node_id: String,
    pub worker: String,
    pub id: Uuid,
    pub until_unix_ms: u64,
    pub max_attempts: u32,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Retry {
    pub task_id: Uuid,
    pub retry_at_unix_ms: u64,
    #[serde(flatten)]
    pub extra: Extra,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Disk {
    #[serde(default)]
    pub records: Vec<Record>,
    #[serde(default)]
    pub leases: Vec<Lease>,
    #[serde(default)]
    pub retries: Vec<Retry>,
    #[serde(flatten)]
    pub extra: Extra,
}
impl Disk {
    pub fn record(&self, id: Uuid) -> Result<&Record> {
        self.records
            .iter()
            .find(|r| r.task.task_id == id)
            .ok_or_else(|| QueueError::Missing.into())
    }
    pub fn record_mut(&mut self, id: Uuid) -> Result<&mut Record> {
        self.records
            .iter_mut()
            .find(|r| r.task.task_id == id)
            .ok_or_else(|| QueueError::Missing.into())
    }
    pub fn retry(&mut self, lease: &Lease, now: u64) -> Result<()> {
        let r = self.record_mut(lease.task_id)?;
        if r.status != Status::Leased {
            return Ok(());
        }
        if r.attempts >= lease.max_attempts {
            r.status = Status::Failed;
            r.error = Some("task lease expired".into());
        } else {
            r.status = Status::Pending;
            let delay = 250u64 << r.attempts.saturating_sub(1).min(6);
            self.retries.retain(|old| old.task_id != lease.task_id);
            self.retries.push(Retry {
                task_id: lease.task_id,
                retry_at_unix_ms: now.saturating_add(delay),
                extra: Default::default(),
            });
        }
        Ok(())
    }
}
#[derive(Clone, Deserialize)]
pub struct Registration {
    pub protocol_version: u32,
    pub worker_id: String,
    pub node_id: String,
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(default)]
    pub reserve: bool,
    pub operations: Vec<Operation>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub capacity: u32,
}
#[derive(Copy, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    Warmup,
    Active,
    Draining,
}
pub struct Channel {
    pub reg: Registration,
    pub phase: Phase,
    pub seen: u64,
}
impl Channel {
    pub fn supports(&self, task: &Task) -> bool {
        self.reg.operations.contains(&task.operation)
            && (task.operation != Operation::Http
                || task
                    .exchange
                    .as_ref()
                    .and_then(|v| v["service"].as_str())
                    .is_some_and(|name| self.reg.services.iter().any(|s| s == name)))
    }
}
#[derive(Deserialize)]
pub struct WorkerRequest {
    pub worker_id: String,
}
#[derive(Clone, Deserialize)]
pub struct LeaseRequest {
    pub worker_id: String,
    pub task_id: Uuid,
    pub lease_id: Uuid,
}
#[derive(Deserialize)]
pub struct Completion {
    #[serde(flatten)]
    pub ids: LeaseRequest,
    pub result: Record,
}
#[derive(Deserialize)]
pub struct Reserve {
    #[serde(default = "one")]
    pub requested_slots: u32,
    #[serde(default)]
    pub requested_bytes: u64,
    #[serde(default = "default_ttl")]
    pub ttl_ms: u64,
}
fn one() -> u32 {
    1
}
fn default_ttl() -> u64 {
    15_000
}
pub struct Reservation {
    pub user: Uuid,
    pub slots: u32,
    pub bytes: u64,
    pub until: u64,
}
