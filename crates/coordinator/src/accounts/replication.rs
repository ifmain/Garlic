use super::{now, Accounts, AccountsService, B64};
use crate::files;
use base64::Engine;
use garlic_relay_data::control::{Authority, Principal, UserSnapshot};
use garlic_relay_session::SessionError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};
use uuid::Uuid;

#[derive(Deserialize, Eq, PartialEq, Serialize)]
struct ReplicaSource {
    url: String,
    owner_id: Uuid,
    epoch: u64,
    revision: u64,
}

impl AccountsService {
    /// Apply a complete snapshot fetched from the configured account authority
    /// over authenticated TLS. Local node registrations and local login sessions
    /// have separate lifetimes from portable account keys.
    pub(crate) fn import(
        &self,
        source: &str,
        owner: &Authority,
        snapshot: UserSnapshot,
    ) -> Result<(), SessionError> {
        if snapshot.version != 1 || owner.owner_id.is_nil() || owner.epoch == 0 {
            return Err("invalid upstream account snapshot".into());
        }
        let mut users = BTreeMap::new();
        let mut names = BTreeSet::new();
        for user in snapshot.users {
            if user.id.is_nil()
                || user.generation == 0
                || !names.insert(user.username.clone())
                || users.insert(user.id, user).is_some()
            {
                return Err("invalid account record in upstream snapshot".into());
            }
        }
        let tombstones = snapshot
            .tombstones
            .into_iter()
            .map(|t| (t.id.to_string(), t))
            .collect::<BTreeMap<_, _>>();
        for t in tombstones.values() {
            if users.get(&t.id).is_some_and(|u| u.revision <= t.revision) {
                users.remove(&t.id);
            }
        }
        let current_time = now();
        let mut portable = BTreeMap::new();
        for (hash, session) in snapshot.portable_sessions {
            let Principal::User {
                user_id,
                username,
                role,
            } = &session.principal
            else {
                continue;
            };
            if !session.owner.owner_id.is_nil()
                || session.owner.epoch != 0
                || session.expires_at <= current_time
                || session.issued_at > current_time
                || session.key_id.is_some()
                || !B64.decode(&hash).is_ok_and(|v| v.len() == 32)
            {
                continue;
            }
            if users.get(user_id).is_some_and(|u| {
                u.enabled
                    && u.username == *username
                    && u.role == *role
                    && u.generation == session.generation
            }) {
                portable.insert(hash, session);
            }
        }
        let replication = self.accounts.with_added_extension("replication.json");
        let incoming = ReplicaSource {
            url: source.to_owned(),
            owner_id: owner.owner_id,
            epoch: owner.epoch,
            revision: snapshot.revision,
        };
        files::edit(&self.accounts, |db: &mut Accounts| {
            db.check_version()?;
            let previous = match files::read::<ReplicaSource>(&replication) {
                Ok(previous) => Some(previous),
                Err(e)
                    if e.downcast_ref::<io::Error>()
                        .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
                {
                    None
                }
                Err(e) => return Err(e),
            };
            let inline: Option<ReplicaSource> = db
                .extra
                .remove("replica_source")
                .map(serde_json::from_value)
                .transpose()?;
            if previous.iter().chain(inline.iter()).any(|previous| {
                previous.url == incoming.url
                    && previous.owner_id == incoming.owner_id
                    && (previous.epoch, previous.revision) > (incoming.epoch, incoming.revision)
            }) {
                return Err("upstream snapshot regressed".into());
            }
            // Commit the watermark under the account lock before changing the database.
            // After an interrupted write the identical snapshot can be replayed.
            if previous.as_ref() != Some(&incoming) {
                files::save(&replication, &incoming)?;
            }
            db.sessions.retain(|_, s| {
                if s.expires_at <= current_time {
                    return false;
                }
                match &s.principal {
                    Principal::Node { .. } => true,
                    Principal::User {
                        user_id,
                        username,
                        role,
                    } => {
                        s.owner.owner_id == self.owner.owner_id
                            && users.get(user_id).is_some_and(|u| {
                                u.enabled
                                    && u.username == *username
                                    && u.role == *role
                                    && u.generation == s.generation
                            })
                    }
                }
            });
            db.sessions.extend(portable);
            db.users = users;
            db.tombstones = tombstones;
            db.revision = snapshot.revision;
            db.extra
                .insert("last_synced_at".into(), json!(current_time));
            Ok(())
        })
    }
}
