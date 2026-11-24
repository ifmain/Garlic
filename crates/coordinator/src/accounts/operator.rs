use super::{now, Accounts};
use crate::{files, http::profile};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use garlic_relay_data::control::{Principal, Role, Subscription, User, UserTombstone};
use garlic_relay_session::SessionError;
use serde_json::{json, Value};
use std::{io, path::Path};
use uuid::Uuid;

const DNS_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
pub(super) fn invalid() -> SessionError {
    io::Error::new(io::ErrorKind::InvalidInput, "authentication Invalid").into()
}
fn entropy() -> Result<[u8; 16], SessionError> {
    let mut bytes = [0; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| io::Error::other("system random generator unavailable"))?;
    Ok(bytes)
}
pub(super) fn hash_password(password: &str) -> Result<String, SessionError> {
    if !(12..=1024).contains(&password.len()) {
        return Err(invalid());
    }
    let salt = SaltString::encode_b64(&entropy()?).map_err(|_| invalid())?;
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| invalid())?
        .to_string())
}
pub fn initialize(path: &Path) -> Result<(), SessionError> {
    let db = Accounts {
        version: 1,
        revision: 1,
        users: Default::default(),
        nodes: Default::default(),
        sessions: Default::default(),
        tombstones: Default::default(),
        extra: [("last_synced_at".into(), json!(now()))].into(),
    };
    files::create(path, &db)
}
pub fn add(path: &Path, name: &str, password: &str) -> Result<User, SessionError> {
    validate_name(name)?;
    let hash = hash_password(password)?;
    files::edit(path, |db: &mut Accounts| {
        insert(db, name, hash, Role::Client, Subscription::Unlimited)
    })
}
pub(super) fn validate_name(name: &str) -> Result<(), SessionError> {
    if name.trim().is_empty()
        || name.len() > 64
        || name
            .chars()
            .any(|c| c.is_control() || matches!(c, '"' | '\'' | '/' | ':' | '\\' | '|'))
    {
        return Err(invalid());
    }
    Ok(())
}
pub(super) fn insert(
    db: &mut Accounts,
    name: &str,
    hash: String,
    role: Role,
    subscription: Subscription,
) -> Result<User, SessionError> {
    validate_name(name)?;
    db.check_version()?;
    if db.users.values().any(|u| u.username == name) {
        return Err(invalid());
    }
    let dns_id = loop {
        let id: String = entropy()?
            .iter()
            .map(|v| char::from(DNS_ALPHABET[*v as usize % DNS_ALPHABET.len()]))
            .collect();
        if !db.users.values().any(|u| u.dns_id == id) {
            break id;
        }
    };
    db.revision = db.revision.checked_add(1).ok_or_else(invalid)?;
    let time = now();
    let user = User {
        id: Uuid::new_v4(),
        dns_id,
        username: name.to_owned(),
        password_hash: hash,
        enabled: true,
        role,
        generation: 1,
        subscription,
        created_at: time,
        updated_at: time,
        revision: db.revision,
    };
    db.users.insert(user.id, user.clone());
    Ok(user)
}

pub enum Change {
    Enabled(bool),
    Role(Role),
    Subscription(Subscription),
    Password(String),
    Rename(String),
    Remove,
}
pub fn change(path: &Path, name: &str, mut change: Change) -> Result<(), SessionError> {
    if let Change::Password(ref mut value) = change {
        *value = hash_password(value)?;
    }
    files::edit(path, |db: &mut Accounts| apply(db, name, change))
}
pub(super) fn apply(db: &mut Accounts, name: &str, change: Change) -> Result<(), SessionError> {
    if let Change::Rename(ref new_name) = change {
        validate_name(new_name)?;
        if new_name != name && db.users.values().any(|u| &u.username == new_name) {
            return Err(invalid());
        }
    }
    db.check_version()?;
    let row = db
        .users
        .values_mut()
        .find(|u| u.username == name)
        .ok_or_else(invalid)?;
    db.revision = db.revision.checked_add(1).ok_or_else(invalid)?;
    row.revision = db.revision;
    row.updated_at = now();
    if !matches!(change, Change::Subscription(_) | Change::Rename(_)) {
        row.generation = row.generation.checked_add(1).ok_or_else(invalid)?;
        db.sessions
            .retain(|_, s| !matches!(s.principal,Principal::User{user_id,..} if user_id==row.id));
    }
    match change {
        Change::Enabled(value) => row.enabled = value,
        Change::Role(value) => row.role = value,
        Change::Subscription(value) => row.subscription = value,
        Change::Password(value) => row.password_hash = value,
        Change::Rename(new_name) => {
            row.username = new_name.clone();
            for session in db.sessions.values_mut() {
                if let Principal::User {
                    user_id, username, ..
                } = &mut session.principal
                {
                    if *user_id == row.id {
                        *username = new_name.clone();
                    }
                }
            }
        }
        Change::Remove => {
            let id = row.id;
            let tomb = UserTombstone {
                id,
                username: row.username.clone(),
                generation: row.generation,
                deleted_at: row.updated_at,
                revision: row.revision,
            };
            db.tombstones.insert(id.to_string(), tomb);
            db.users.remove(&id);
        }
    }
    Ok(())
}

pub fn list(path: &Path) -> Result<Vec<Value>, SessionError> {
    let db: Accounts = files::read(path)?;
    db.check_version()?;
    let mut users: Vec<_> = db.users.into_values().collect();
    users.sort_by(|a, b| a.username.cmp(&b.username));
    Ok(users
        .iter()
        .map(|u| {
            let mut entry = profile(u);
            entry["dns_id"] = json!(u.dns_id);
            if let Ok(domain) = std::env::var("GARLIC_DNS_DOMAIN") {
                entry["doh_url"] = json!(format!("https://{}.{domain}/dns-query", u.dns_id));
            }
            entry
        })
        .collect())
}
pub fn nodes(path: &Path) -> Result<Vec<Value>, SessionError> {
    let db: Accounts = files::read(path)?;
    db.check_version()?;
    Ok(db.nodes.into_iter().map(|(id,n)|json!({"node_id":id,"public_key":n.public_key,"label":n.label,"enabled":n.enabled})).collect())
}
pub fn revoke(path: &Path, id: &str) -> Result<(), SessionError> {
    files::edit(path, |db: &mut Accounts| {
        db.check_version()?;
        db.nodes.get_mut(id).ok_or_else(invalid)?.enabled = false;
        db.sessions
            .retain(|_, s| !matches!(&s.principal,Principal::Node{node_id} if node_id==id));
        Ok(())
    })
}
