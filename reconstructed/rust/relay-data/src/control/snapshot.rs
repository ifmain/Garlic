use super::Authority;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserSnapshot {
    pub version: u32,
    pub revision: u64,
    pub server_unix: u64,
    pub users: Vec<User>,
    #[serde(default)]
    pub tombstones: Vec<UserTombstone>,
    #[serde(default)]
    pub portable_sessions: BTreeMap<String, Session>,
}

// Leave credentials and session identifiers out of routine diagnostic output.
impl std::fmt::Debug for UserSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserSnapshot")
            .field("version", &self.version)
            .field("revision", &self.revision)
            .field("users", &self.users.len())
            .field("tombstones", &self.tombstones.len())
            .field("portable_sessions", &self.portable_sessions.len())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct User {
    pub id: Uuid,
    #[serde(default)]
    pub dns_id: String,
    pub username: String,
    pub password_hash: String,
    pub enabled: bool,
    #[serde(default)]
    pub role: Role,
    pub generation: u64,
    #[serde(default)]
    pub subscription: Subscription,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub updated_at: u64,
    #[serde(default)]
    pub revision: u64,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Client,
    Admin,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Subscription {
    #[default]
    Unlimited,
    ExpiresAt {
        expires_at: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserTombstone {
    pub id: Uuid,
    pub username: String,
    pub generation: u64,
    pub deleted_at: u64,
    pub revision: u64,
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Session {
    pub principal: Principal,
    pub owner: Authority,
    pub key_id: Option<String>,
    pub generation: u64,
    pub issued_at: u64,
    pub expires_at: u64,
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Principal {
    User {
        user_id: Uuid,
        username: String,
        #[serde(default)]
        role: Role,
    },
    Node {
        node_id: String,
    },
}
