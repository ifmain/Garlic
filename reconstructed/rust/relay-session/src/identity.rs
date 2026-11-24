//! Infrastructure authentication for an already provisioned Garlic node.
use crate::{endpoint::NodeEndpoint, SessionError};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};
use garlic_relay_data::control::{Authority, Principal};
use reqwest::blocking::Client;
use ring::{
    digest, hmac,
    signature::{Ed25519KeyPair, KeyPair},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{self, Read},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize, Serialize)]
pub struct IdentityFile {
    version: u32,
    secret: String,
}
#[derive(Deserialize, Serialize)]
pub struct KeyRing {
    version: u32,
    active: String,
    keys: BTreeMap<String, String>,
}

#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ChallengeRequest {
    pub node_id: String,
    pub public_key: String,
    pub label: String,
    pub key_id: String,
    pub client_nonce: String,
    pub issued_at: u64,
    pub membership: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Challenge {
    pub nonce: String,
    pub authority: Authority,
    pub expires_at: u64,
    pub request: ChallengeRequest,
}

#[derive(Deserialize, Serialize)]
pub struct Proof {
    pub nonce: String,
    signature: String,
    mac: String,
}

#[derive(Deserialize, Serialize)]
pub struct Grant {
    pub token: String,
    pub expires_at: u64,
    pub principal: Principal,
    pub authority: Authority,
}

pub struct Identity {
    signing: Ed25519KeyPair,
    pub node_id: String,
}

fn fixed(value: &str) -> Result<[u8; 32], SessionError> {
    B64.decode(value)?
        .try_into()
        .map_err(|_| "invalid node key or nonce length".into())
}

fn new_secret() -> Result<String, SessionError> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes)
        .map_err(|_| io::Error::other("system random generator unavailable"))?;
    Ok(B64.encode(bytes))
}
impl IdentityFile {
    pub fn generate() -> Result<Self, SessionError> {
        Ok(Self {
            version: 1,
            secret: new_secret()?,
        })
    }
}

impl Identity {
    pub fn load(file: &IdentityFile) -> Result<Self, SessionError> {
        if file.version != 1 {
            return Err("unsupported node identity version".into());
        }
        let signing = Ed25519KeyPair::from_seed_unchecked(&fixed(&file.secret)?)
            .map_err(|_| "invalid Ed25519 node identity")?;
        let node_id = B64.encode(digest::digest(
            &digest::SHA256,
            signing.public_key().as_ref(),
        ));
        Ok(Self { signing, node_id })
    }

    pub fn request(
        &self,
        ring: &KeyRing,
        label: &str,
        now: u64,
    ) -> Result<ChallengeRequest, SessionError> {
        if ring.version != 1 {
            return Err("unsupported infrastructure key ring".into());
        }
        let key = ring.key(&ring.active)?;
        let mut nonce = [0u8; 32];
        getrandom::fill(&mut nonce)
            .map_err(|_| io::Error::other("could not generate node nonce"))?;
        let client_nonce = B64.encode(nonce);
        let public_key = B64.encode(self.signing.public_key().as_ref());
        let mut request = ChallengeRequest {
            node_id: self.node_id.clone(),
            public_key,
            label: label.into(),
            key_id: ring.active.clone(),
            client_nonce,
            issued_at: now,
            membership: String::new(),
        };
        request.membership = B64.encode(hmac::sign(&key, &request.membership_data()?));
        Ok(request)
    }

    pub fn prove(
        &self,
        ring: &KeyRing,
        sent: &ChallengeRequest,
        response: &Challenge,
        now: u64,
    ) -> Result<Proof, SessionError> {
        if &response.request != sent
            || sent.node_id != self.node_id
            || response.expires_at <= now
            || response.authority.owner_id.is_nil()
            || fixed(&response.nonce).is_err()
        {
            return Err("challenge does not match the outstanding node request".into());
        }
        let key = ring.key(&sent.key_id)?;
        let message = serde_json::to_vec(&("garlic-infra-v1", response))?;
        Ok(Proof {
            nonce: response.nonce.clone(),
            signature: B64.encode(self.signing.sign(&message)),
            mac: B64.encode(hmac::sign(&key, &message)),
        })
    }
}

impl KeyRing {
    pub fn create(id: &str) -> Result<Self, SessionError> {
        let mut ring = Self {
            version: 1,
            active: id.into(),
            keys: Default::default(),
        };
        ring.add(id)?;
        Ok(ring)
    }
    pub fn add(&mut self, id: &str) -> Result<(), SessionError> {
        if self.version != 1
            || id.trim().is_empty()
            || id.len() > 64
            || id.chars().any(char::is_control)
            || self.keys.contains_key(id)
        {
            return Err("authentication Invalid".into());
        }
        self.keys.insert(id.into(), new_secret()?);
        Ok(())
    }
    pub fn activate(&mut self, id: &str) -> Result<(), SessionError> {
        self.key(id)?;
        self.active = id.into();
        Ok(())
    }
    pub fn retire(&mut self, id: &str) -> Result<(), SessionError> {
        if id == self.active || self.keys.remove(id).is_none() {
            return Err("authentication Invalid".into());
        }
        Ok(())
    }
    pub fn status(&self) -> serde_json::Value {
        serde_json::json!({"accepted":self.keys.keys().collect::<Vec<_>>(),"active":self.active})
    }
    fn key(&self, id: &str) -> Result<hmac::Key, SessionError> {
        let secret = self
            .keys
            .get(id)
            .ok_or("infrastructure membership key unavailable")?;
        Ok(hmac::Key::new(hmac::HMAC_SHA256, &fixed(secret)?))
    }
}

impl ChallengeRequest {
    fn membership_data(&self) -> Result<Vec<u8>, SessionError> {
        Ok(serde_json::to_vec(&(
            "garlic-infra-admission-v1",
            &self.node_id,
            &self.public_key,
            &self.label,
            &self.key_id,
            &self.client_nonce,
            self.issued_at,
        ))?)
    }

    /// Verify a node's public identity and its membership in the provisioned
    /// key ring before retaining a short-lived challenge.
    pub fn verify(&self, ring: &KeyRing, now: u64) -> Result<(), SessionError> {
        let public = fixed(&self.public_key)?;
        if ring.version != 1
            || self.issued_at.abs_diff(now) > 30
            || self.node_id != B64.encode(digest::digest(&digest::SHA256, &public))
            || self.label.is_empty()
            || self.label.len() > 255
            || self.label.chars().any(char::is_control)
        {
            return Err("invalid node admission request".into());
        }
        fixed(&self.client_nonce)?;
        hmac::verify(
            &ring.key(&self.key_id)?,
            &self.membership_data()?,
            &fixed(&self.membership)?,
        )
        .map_err(|_| "invalid node membership".into())
    }
}

impl Proof {
    pub fn verify(
        &self,
        ring: &KeyRing,
        challenge: &Challenge,
        now: u64,
    ) -> Result<(), SessionError> {
        if self.nonce != challenge.nonce || challenge.expires_at <= now {
            return Err("node proof has no active challenge".into());
        }
        let message = serde_json::to_vec(&("garlic-infra-v1", challenge))?;
        let key = ring.key(&challenge.request.key_id)?;
        hmac::verify(&key, &message, &fixed(&self.mac)?)
            .map_err(|_| "invalid node membership proof")?;
        let public = fixed(&challenge.request.public_key)?;
        ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public)
            .verify(&message, &B64.decode(&self.signature)?)
            .map_err(|_| "invalid node signature".into())
    }
}

fn now() -> Result<u64, SessionError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

/// Authenticate with a configured HTTPS coordinator using the operator's CA.
pub fn login(
    origin: &str,
    ca: &[u8],
    identity: &Identity,
    keys: &KeyRing,
    label: &str,
    timeout: Duration,
) -> Result<Grant, SessionError> {
    login_endpoint(
        &NodeEndpoint::new(origin)?,
        ca,
        identity,
        keys,
        label,
        timeout,
    )
}

pub fn login_endpoint(
    endpoint: &NodeEndpoint,
    ca: &[u8],
    identity: &Identity,
    keys: &KeyRing,
    label: &str,
    timeout: Duration,
) -> Result<Grant, SessionError> {
    let mut url = endpoint.url().clone();
    let mut builder = Client::builder()
        .use_rustls_tls()
        .tls_built_in_root_certs(false)
        .https_only(true)
        .no_proxy()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none());
    let roots = reqwest::Certificate::from_pem_bundle(ca)?;
    if roots.is_empty() {
        return Err("empty coordinator trust store".into());
    }
    for root in roots {
        builder = builder.add_root_certificate(root);
    }
    let client = endpoint.configure(builder).build()?;
    let request = identity.request(keys, label, now()?)?;
    url.set_path("/v1/infra/challenge");
    let response: Challenge = post(&client, &url, &request)?;
    let proof = identity.prove(keys, &request, &response, now()?)?;
    url.set_path("/v1/infra/prove");
    let grant: Grant = post(&client, &url, &proof)?;
    if !matches!(&grant.principal, Principal::Node{node_id} if *node_id==identity.node_id)
        || grant.authority != response.authority
        || grant.token.is_empty()
        || grant.expires_at <= now()?
    {
        return Err("coordinator grant does not match the authenticated node".into());
    }
    Ok(grant)
}

fn post<T: Serialize, R: serde::de::DeserializeOwned>(
    client: &Client,
    url: &reqwest::Url,
    body: &T,
) -> Result<R, SessionError> {
    let response = client
        .post(url.clone())
        .json(body)
        .send()?
        .error_for_status()?;
    let mut bytes = Vec::new();
    response.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err("oversized authentication response".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}
