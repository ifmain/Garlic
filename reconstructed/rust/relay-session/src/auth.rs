//! Read-only verification of node credentials issued by the Garlic account authority.
use crate::{coordinator::NodeSession, SessionError};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use garlic_relay_data::control::{Authority, Principal, Session};
use ring::digest::{digest, SHA256};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

#[derive(Deserialize)]
struct Store {
    version: u32,
    nodes: BTreeMap<String, Node>,
    sessions: BTreeMap<String, Session>,
}

#[derive(Deserialize)]
struct Node {
    enabled: bool,
}

#[derive(Deserialize)]
struct Keys {
    version: u32,
    keys: BTreeMap<String, serde::de::IgnoredAny>,
}

/// The authority is loaded from the coordinator's trusted owner/epoch file.
/// The key ring is consulted on every upgrade so retirement applies immediately.
pub fn verify_node(
    accounts_path: &Path,
    keys_path: &Path,
    authority: &Authority,
    token: &str,
    now: u64,
) -> Result<NodeSession, SessionError> {
    let denied = || io::Error::new(io::ErrorKind::PermissionDenied, "node session denied");
    if token.is_empty() || token.len() > 1024 {
        return Err(denied().into());
    }
    let store: Store = serde_json::from_reader(BufReader::new(File::open(accounts_path)?))?;
    let keys: Keys = serde_json::from_reader(BufReader::new(File::open(keys_path)?))?;
    let hash = URL_SAFE_NO_PAD.encode(digest(&SHA256, token.as_bytes()));
    let session = store.sessions.get(&hash).ok_or_else(denied)?;
    let Principal::Node { node_id } = &session.principal else {
        return Err(denied().into());
    };
    if store.version != 1
        || keys.version != 1
        || session.owner != *authority
        || session.generation != 0
        || session.issued_at > now
        || session.expires_at <= now
        || !store.nodes.get(node_id).is_some_and(|node| node.enabled)
        || !session
            .key_id
            .as_ref()
            .is_some_and(|id| keys.keys.contains_key(id))
    {
        return Err(denied().into());
    }
    Ok(NodeSession {
        node_id: node_id.clone(),
        authority: authority.clone(),
    })
}
