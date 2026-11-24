use super::{
    denied, eligible, now, operator, random_secret, token_hash, Accounts, AccountsService,
    SubscriptionExpired, B64,
};
use crate::{files, http::profile};
use base64::Engine;
use garlic_relay_data::control::{Authority, Principal, Role, Session, Subscription, User};
use garlic_relay_session::SessionError;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io;

#[derive(Deserialize, serde::Serialize)]
pub struct Manage {
    pub action: String,
    pub username: String,
    pub new_username: Option<String>,
    pub subscription: Option<String>,
    pub role: Option<Role>,
}

#[derive(Debug)]
pub struct AdminRequired;
impl std::fmt::Display for AdminRequired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("authentication Forbidden")
    }
}
impl std::error::Error for AdminRequired {}

pub fn parse_subscription(value: &str) -> Result<Subscription, SessionError> {
    if value == "unlimited" {
        return Ok(Subscription::Unlimited);
    }
    let expires_at = match value.parse::<u64>() {
        Ok(seconds) => seconds,
        Err(_) => chrono::DateTime::parse_from_rfc3339(value)
            .ok()
            .and_then(|dt| u64::try_from(dt.timestamp()).ok())
            .ok_or_else(operator::invalid)?,
    };
    Ok(Subscription::ExpiresAt { expires_at })
}

fn check(db: &Accounts, owner: &Authority, token: &str) -> Result<(), SessionError> {
    db.check_version()?;
    if db.user(owner, token)?.0.role != Role::Admin {
        return Err(AdminRequired.into());
    }
    Ok(())
}
fn rows(db: &Accounts) -> Vec<Value> {
    let mut users: Vec<_> = db.users.values().collect();
    users.sort_by(|a, b| a.username.cmp(&b.username));
    users
        .into_iter()
        .map(|u| {
            let mut row = profile(u);
            row["client_usage"] = Value::Null;
            row
        })
        .collect()
}

fn access_key(db: &mut Accounts, user: &User, action: &str) -> Result<Value, SessionError> {
    let time = now();
    if !user.enabled {
        return Err(denied());
    }
    if !eligible(user, time) {
        return Err(SubscriptionExpired.into());
    }
    let mut bytes = [0u8; 24];
    getrandom::fill(&mut bytes)
        .map_err(|_| io::Error::other("system random generator unavailable"))?;
    let token = format!("gu_{}_{}", user.id.simple(), B64.encode(bytes));
    let expires_at = time + 31_536_000;
    db.sessions.retain(|_, s| s.expires_at > time);
    db.sessions.insert(
        token_hash(&token),
        Session {
            owner: Authority {
                owner_id: uuid::Uuid::nil(),
                epoch: 0,
            },
            principal: Principal::User {
                user_id: user.id,
                username: user.username.clone(),
                role: user.role,
            },
            generation: user.generation,
            key_id: None,
            issued_at: time,
            expires_at,
        },
    );
    Ok(
        json!({"ok":true,"action":action,"username":user.username,"user":profile(user),"token":token,"expires_at":expires_at}),
    )
}

impl AccountsService {
    pub fn require_admin(&self, token: &str) -> Result<(), SessionError> {
        check(&files::read(&self.accounts)?, &self.owner, token)
    }
    pub fn admin_users(&self, token: &str) -> Result<Vec<Value>, SessionError> {
        let db = files::read(&self.accounts)?;
        check(&db, &self.owner, token)?;
        Ok(rows(&db))
    }
    pub fn admin_manage(&self, token: &str, request: Manage) -> Result<Value, SessionError> {
        let db = files::read(&self.accounts)?;
        check(&db, &self.owner, token)?;
        // Do password hashing outside the writer lock, then recheck the administrator inside it.
        let new_hash = if request.action == "create" {
            operator::validate_name(&request.username)?;
            Some(operator::hash_password(&random_secret("")?)?)
        } else {
            None
        };
        files::edit(&self.accounts, |db: &mut Accounts| {
            check(db, &self.owner, token)?;
            let name = &request.username;
            let action = request.action.as_str();
            let mut out = json!({"ok":true,"action":action,"username":name});
            if let Some(hash) = new_hash {
                let sub =
                    parse_subscription(request.subscription.as_deref().unwrap_or("unlimited"))?;
                let user =
                    operator::insert(db, name, hash, request.role.unwrap_or(Role::Client), sub)?;
                // An expired subscription still creates the account in the original service.
                return Ok(access_key(db, &user, action));
            }
            if action == "issue_key" {
                let user = db
                    .users
                    .values()
                    .find(|u| &u.username == name)
                    .cloned()
                    .ok_or_else(denied)?;
                return Ok(access_key(db, &user, action));
            }
            let change = match action {
                "rename" => {
                    let new_name = request.new_username.ok_or_else(operator::invalid)?;
                    out["username"] = json!(new_name);
                    operator::Change::Rename(new_name)
                }
                "role" => {
                    let role = request.role.ok_or_else(operator::invalid)?;
                    out["role"] = json!(role);
                    operator::Change::Role(role)
                }
                "subscription" => operator::Change::Subscription(parse_subscription(
                    request
                        .subscription
                        .as_deref()
                        .ok_or_else(operator::invalid)?,
                )?),
                "disable" => operator::Change::Enabled(false),
                "enable" => operator::Change::Enabled(true),
                "remove" => {
                    let user = db
                        .users
                        .values()
                        .find(|u| &u.username == name)
                        .ok_or_else(operator::invalid)?;
                    out["removed_user_id"] = json!(user.id);
                    operator::Change::Remove
                }
                _ => return Err(operator::invalid()),
            };
            operator::apply(db, name, change)?;
            out["users"] = json!(rows(db));
            Ok(Ok(out))
        })?
    }
}
