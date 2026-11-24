use crate::files;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use garlic_relay_data::control::{
    Authority, Principal, Session, Subscription, User, UserSnapshot, UserTombstone,
};
use garlic_relay_session::{
    identity::{Challenge, ChallengeRequest, Grant, KeyRing, Proof},
    SessionError,
};
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    io,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;
pub mod admin;
pub mod operator;
mod replication;

#[derive(Deserialize, Serialize)]
struct Node {
    public_key: String,
    label: String,
    enabled: bool,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
struct Accounts {
    version: u32,
    revision: u64,
    users: BTreeMap<Uuid, User>,
    nodes: BTreeMap<String, Node>,
    sessions: BTreeMap<String, Session>,
    tombstones: BTreeMap<String, UserTombstone>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

pub struct AccountsService {
    pub accounts: PathBuf,
    pub keys: PathBuf,
    pub owner: Authority,
    pending: Mutex<HashMap<String, Challenge>>,
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn random_secret(prefix: &str) -> Result<String, SessionError> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes)
        .map_err(|_| io::Error::other("system random generator unavailable"))?;
    Ok(format!("{prefix}{}", B64.encode(bytes)))
}

pub fn token_hash(token: &str) -> String {
    B64.encode(digest(&SHA256, token.as_bytes()))
}
pub fn denied() -> SessionError {
    io::Error::new(io::ErrorKind::PermissionDenied, "authorization denied").into()
}

fn eligible(user: &User, timestamp: u64) -> bool {
    user.enabled
        && !matches!(user.subscription,Subscription::ExpiresAt { expires_at } if expires_at <= timestamp)
}

impl Accounts {
    fn check_version(&self) -> Result<(), SessionError> {
        if self.version != 1 {
            return Err("unsupported accounts file version".into());
        }
        Ok(())
    }

    fn user(&self, owner: &Authority, token: &str) -> Result<(User, Session), SessionError> {
        let s = self.sessions.get(&token_hash(token)).ok_or_else(denied)?;
        let Principal::User {
            user_id,
            username,
            role,
        } = &s.principal
        else {
            return Err(denied());
        };
        let time = now();
        let user = self
            .users
            .get(user_id)
            .filter(|u| {
                &u.username == username
                    && &u.role == role
                    && u.generation == s.generation
                    && eligible(u, time)
            })
            .ok_or_else(denied)?;
        let local =
            s.owner.owner_id == owner.owner_id && s.owner.epoch > 0 && s.owner.epoch <= owner.epoch;
        let portable = s.owner.owner_id.is_nil() && s.owner.epoch == 0;
        if !(local || portable) || s.expires_at <= time || s.issued_at > time {
            return Err(denied());
        }
        Ok((user.clone(), s.clone()))
    }

    fn issue(
        &mut self,
        owner: &Authority,
        principal: Principal,
        generation: u64,
        key_id: Option<String>,
    ) -> Result<Grant, SessionError> {
        self.check_version()?;
        let time = now();
        self.sessions.retain(|_, s| s.expires_at > time);
        let (prefix, lifetime) = match principal {
            Principal::User { .. } => ("gu_", 3600),
            Principal::Node { .. } => ("gn_", 300),
        };
        let token = random_secret(prefix)?;
        let expires_at = time + lifetime;
        self.sessions.insert(
            token_hash(&token),
            Session {
                owner: owner.clone(),
                principal: principal.clone(),
                generation,
                key_id,
                issued_at: time,
                expires_at,
            },
        );
        Ok(Grant {
            token,
            expires_at,
            principal,
            authority: owner.clone(),
        })
    }
}

impl AccountsService {
    pub fn node_label(&self, id: &str) -> Result<String, SessionError> {
        let db: Accounts = files::read(&self.accounts)?;
        db.check_version()?;
        db.nodes
            .get(id)
            .filter(|n| n.enabled)
            .map(|n| n.label.clone())
            .ok_or_else(denied)
    }
    pub fn new(accounts: PathBuf, keys: PathBuf, owner: Authority) -> Self {
        Self {
            accounts,
            keys,
            owner,
            pending: Default::default(),
        }
    }

    pub fn login(&self, username: &str, password: &str) -> Result<Grant, SessionError> {
        self.login_for_network(username, password, false)
    }

    pub(crate) fn login_for_network(&self, username: &str, password: &str, portable: bool) -> Result<Grant, SessionError> {
        let db: Accounts = files::read(&self.accounts)?;
        db.check_version()?;
        let user = db
            .users
            .values()
            .find(|u| u.username == username)
            .ok_or_else(denied)?;
        let hash = PasswordHash::new(&user.password_hash).map_err(|_| denied())?;
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(|_| denied())?;
        if !user.enabled {
            return Err(denied());
        }
        if !eligible(user, now()) {
            return Err(SubscriptionExpired.into());
        }
        files::edit(&self.accounts, |current: &mut Accounts| {
            let row = current
                .users
                .get(&user.id)
                .filter(|u| {
                    u.username == username
                        && u.generation == user.generation
                        && u.password_hash == user.password_hash
                        && eligible(u, now())
                })
                .ok_or_else(denied)?;
            let scope = if portable { Authority {owner_id: Uuid::nil(), epoch: 0} } else { self.owner.clone() };
            let mut grant = current.issue(
                &scope,
                Principal::User {
                    user_id: row.id,
                    username: row.username.clone(),
                    role: row.role,
                },
                row.generation,
                None,
            )?;
            grant.authority = self.owner.clone();
            Ok(grant)
        })
    }

    pub fn challenge(&self, request: ChallengeRequest) -> Result<Challenge, SessionError> {
        let time = now();
        let keys: KeyRing = files::read(&self.keys)?;
        request.verify(&keys, time).map_err(|_| denied())?;
        let db: Accounts = files::read(&self.accounts)?;
        db.check_version()?;
        if db
            .nodes
            .get(&request.node_id)
            .is_some_and(|n| !n.enabled || n.public_key != request.public_key)
        {
            return Err(denied());
        }
        let nonce = random_secret("")?;
        let mut pending = self.pending.lock().unwrap();
        pending.retain(|_, c| c.expires_at > time);
        if pending.len() >= 4096
            || pending.values().any(|c| {
                c.request.node_id == request.node_id
                    && c.request.client_nonce == request.client_nonce
            })
        {
            return Err(denied());
        }
        let challenge = Challenge {
            nonce: nonce.clone(),
            authority: self.owner.clone(),
            expires_at: time + 30,
            request,
        };
        pending.insert(nonce, challenge.clone());
        Ok(challenge)
    }

    pub fn prove(&self, proof: Proof) -> Result<Grant, SessionError> {
        let challenge = self
            .pending
            .lock()
            .unwrap()
            .remove(&proof.nonce)
            .ok_or_else(denied)?;
        let keys: KeyRing = files::read(&self.keys)?;
        proof
            .verify(&keys, &challenge, now())
            .map_err(|_| denied())?;
        let request = challenge.request;
        files::edit(&self.accounts, |db: &mut Accounts| {
            let row = db
                .nodes
                .entry(request.node_id.clone())
                .or_insert_with(|| Node {
                    enabled: true,
                    public_key: request.public_key.clone(),
                    label: request.label.clone(),
                    extra: Default::default(),
                });
            if !row.enabled || row.public_key != request.public_key {
                return Err(denied());
            }
            row.label = request.label;
            db.issue(
                &self.owner,
                Principal::Node {
                    node_id: request.node_id,
                },
                0,
                Some(request.key_id),
            )
        })
    }

    pub fn user(&self, token: &str) -> Result<(User, Session), SessionError> {
        let db: Accounts = files::read(&self.accounts)?;
        db.check_version()?;
        db.user(&self.owner, token)
    }

    pub fn logout(&self, token: &str) -> Result<(), SessionError> {
        files::edit(&self.accounts, |db: &mut Accounts| {
            db.check_version()?;
            db.sessions.remove(&token_hash(token));
            Ok(())
        })
    }

    pub fn snapshot(&self) -> Result<UserSnapshot, SessionError> {
        let db: Accounts = files::read(&self.accounts)?;
        db.check_version()?;
        let time = now();
        Ok(UserSnapshot {
            version: 1,
            revision: db.revision,
            server_unix: time,
            users: db.users.into_values().collect(),
            tombstones: db.tombstones.into_values().collect(),
            portable_sessions: db
                .sessions
                .into_iter()
                .filter(|(_, s)| {
                    matches!(s.principal, Principal::User { .. })
                        && s.owner.owner_id.is_nil()
                        && s.owner.epoch == 0
                        && s.expires_at > time
                })
                .collect(),
        })
    }
}

#[derive(Debug)]
pub struct SubscriptionExpired;
impl std::fmt::Display for SubscriptionExpired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("authentication SubscriptionExpired")
    }
}
impl std::error::Error for SubscriptionExpired {}
