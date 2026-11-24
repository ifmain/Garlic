//! JSON messages carried in text frames between the coordinator and exit worker.

mod snapshot;
mod validation;
pub use snapshot::{Principal, Role, Session, Subscription, User, UserSnapshot, UserTombstone};
pub use validation::valid_connect_authority;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Message {
    Hello(Hello),
    Ready {
        protocol_version: u32,
        worker_id: Uuid,
        authority: Authority,
    },
    Open {
        request_id: Uuid,
        open: StreamOpen,
        admin: bool,
    },
    Opened {
        request_id: Uuid,
        error: Option<ProxyError>,
    },
    AdoptFlow {
        flow_id: Uuid,
    },
    Drain,
    Drained,
    CloseFlow {
        flow_id: Uuid,
    },
    UserSync {
        snapshot: UserSnapshot,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    pub protocol_version: u32,
    pub worker_id: Uuid,
    pub node_id: String,
    pub capabilities: Vec<Capability>,
    pub max_streams: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Authority {
    pub owner_id: Uuid,
    pub epoch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Duplex,
    HalfClose,
    Window,
    Reset,
    Resume,
    StructuredErrors,
}

impl Capability {
    pub const ALL: [Self; 6] = [
        Self::Duplex,
        Self::HalfClose,
        Self::Window,
        Self::Reset,
        Self::Resume,
        Self::StructuredErrors,
    ];
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StreamOpen {
    pub control_version: u32,
    pub data_version: u8,
    pub node_id: String,
    pub channel_id: Uuid,
    pub user_id: Uuid,
    pub flow_id: Uuid,
    pub stream_id: Uuid,
    pub connect_authority: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_exit: Option<String>,
    pub route: Route,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    pub window_size: u32,
    pub deadline_unix_ms: u64,
    pub authority: Authority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Direct,
    ThroughRu,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProxyError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    IncompatibleControlVersion,
    IncompatibleDataVersion,
    UnsupportedCapability,
    InvalidAuthority,
    InvalidDestination,
    InvalidIdentifier,
    InvalidWindow,
    StaleAuthority,
    RouteUnavailable,
    Unauthorized,
    Forbidden,
    SubscriptionExpired,
    CapacityExhausted,
    DeadlineExpired,
    InvalidRequest,
}
