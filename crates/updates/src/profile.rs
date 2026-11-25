//! Declarative client slot. Its sole capability is to limit the size of an outgoing data record.
use super::{publisher_key, Envelope};
use base64::{engine::general_purpose::STANDARD, Engine};
use garlic_state::{edit_or_create, StateError};
use ring::signature;
use serde::{Deserialize, Serialize};
use std::path::Path;

const DOMAIN: &[u8] = b"Garlic Alpha record profile v1\0";

// The two instructions compose the existing GEX packet and envelope codecs.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Body { Base, SelectiveAck }
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Frame { Fragments, Tlv }
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Module { pub version: u8, pub body: Body, pub frame: Frame }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Installed { pub frame_bytes: usize, pub module: Option<Module> }
impl Default for Installed {
    fn default() -> Self { Self { frame_bytes: 16384, module: None } }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    wasm: Option<String>,
    network: String,
    abi: u32,
    module: Option<Module>,
    sequence: u64,
    expires: u64,
    frame_bytes: usize,
}

fn verify(env: &Envelope, key: &[u8], network: &str) -> Result<Profile, StateError> {
    if env.payload.len() > 16384 || env.signature.len() != 88 {
        return Err("oversize or invalid profile envelope".into());
    }
    let raw = STANDARD.decode(&env.payload)?;
    let sig = STANDARD.decode(&env.signature)?;
    signature::UnparsedPublicKey::new(&signature::ED25519, key)
        .verify(&[DOMAIN, raw.as_slice()].concat(), &sig)
        .map_err(|_| "incorrect profile signature")?;
    let mut profile: Profile = serde_json::from_slice(&raw)?;
    if profile.network != network || !matches!((profile.abi, profile.module, profile.wasm.is_some()), (1, None, false) | (2, Some(Module { version: 128..=255, .. }), false) | (3, _, true)) || profile.sequence == 0
        || !(1024..=16384).contains(&profile.frame_bytes)
    {
        return Err("incompatible record profile".into());
    }
    if let Some(code) = &profile.wasm {
        let guest = crate::guest::Guest::load(code)?;
        let bytes = guest.call("record_limit", (profile.frame_bytes as i32, 1024, 16384))?;
        if !(1024..=16384).contains(&bytes) || profile.module.is_some_and(|m| m.version < 128) {
            return Err("invalid guest record profile".into());
        }
        profile.frame_bytes = bytes as usize;
    }
    Ok(profile)
}

#[derive(Default, Deserialize, Serialize)]
struct Slot {
    current: Option<Envelope>,
}

fn select(path: &Path, key: &[u8], network: &str, offer: Option<&Envelope>, now: u64)
    -> Result<Option<Installed>, StateError>
{
    let candidate = offer.and_then(|e| verify(e, key, network).ok()).filter(|p| p.expires > now);
    let prior = if path.exists() {
        serde_json::from_reader::<_, Slot>(std::fs::File::open(path)?)?.current
    } else { None };
    let previous = prior.as_ref().map(|e| verify(e, key, network)).transpose()?;
    if let Some(new) = candidate.filter(|p| previous.as_ref().is_none_or(|old| p.sequence > old.sequence)) {
        return edit_or_create(path, || Ok(Slot::default()), |slot| {
            let current = slot.current.as_ref().map(|e| verify(e, key, network)).transpose()?;
            if current.as_ref().is_none_or(|old| new.sequence > old.sequence) {
                slot.current = offer.cloned();
                Ok(Some(Installed {frame_bytes: new.frame_bytes, module: new.module}))
            } else {
                Ok(current.filter(|p| p.expires > now).map(|p| Installed {frame_bytes: p.frame_bytes, module: p.module}))
            }
        });
    }
    Ok(previous.filter(|p| p.expires > now).map(|p| Installed {frame_bytes: p.frame_bytes, module: p.module}))
}

pub fn install(path: &Path, network: &str, offer: Option<&Envelope>, now: u64) -> Installed {
    use std::io::Read;
    let mut key = [0; 32];
    match std::fs::File::open(path.with_extension("pub")) {
        Ok(mut local) => {
            let mut extra = [0; 1];
            if local.read_exact(&mut key).is_err() || local.read(&mut extra).ok() != Some(0) {
                return Installed::default();
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let Some(pinned) = publisher_key() else { return Installed::default(); };
            key = pinned;
        }
        Err(_) => return Installed::default(),
    }
    select(path, &key, network, offer, now).ok().flatten().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::{rand::SystemRandom, signature::{Ed25519KeyPair, KeyPair}};
    #[test]
    fn signed_profiles_are_scoped_bounded_and_monotonic() {
        let signer = Ed25519KeyPair::from_pkcs8(Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap().as_ref()).unwrap();
        let sign = |seq, size, extra: bool| {
            let mut p = serde_json::json!({"network":"fixture","abi":1,"sequence":seq,"expires":100,"frame_bytes":size});
            if extra { p["command"] = "test".into(); }
            let raw = serde_json::to_vec(&p).unwrap();
            Envelope { signature: STANDARD.encode(signer.sign(&[DOMAIN, raw.as_slice()].concat()).as_ref()), payload: STANDARD.encode(raw) }
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("slot");
        let choose = |offer: Option<&Envelope>, now| select(&path, signer.public_key().as_ref(), "fixture", offer, now).unwrap().map(|p| p.frame_bytes);
        assert_eq!(choose(Some(&sign(2, 2048, false)), 50), Some(2048));
        for invalid in [sign(1, 4096, false), sign(3, 0, false), sign(3, 4096, true)] {
            assert_eq!(choose(Some(&invalid), 50), Some(2048));
        }
        assert!(verify(&sign(3,4096,false), &[0;32], "fixture").is_err());
        assert!(verify(&sign(3,4096,false), signer.public_key().as_ref(), "other").is_err());
        let signed_guest = |seq, wasm| {
            let raw = serde_json::to_vec(&serde_json::json!({"abi":3,"network":"fixture","sequence":seq,
                "expires":100,"frame_bytes":4096,"wasm":wasm})).unwrap();
            Envelope {signature:STANDARD.encode(signer.sign(&[DOMAIN,&raw].concat()).as_ref()),payload:STANDARD.encode(raw)}
        };
        let valid = signed_guest(3, "AGFzbQEAAAABCAFgA39/fwF/AwIBAAcQAQxyZWNvcmRfbGltaXQAAAoGAQQAIAEL");
        assert_eq!(choose(Some(&valid), 50), Some(1024));
        // A trapping but correctly signed replacement cannot displace the working guest.
        assert_eq!(choose(Some(&signed_guest(4,"AGFzbQEAAAABCAFgA39/fwF/AwIBAAcQAQxyZWNvcmRfbGltaXQAAAoFAQMAAAs=")),50),Some(1024));
        assert_eq!(choose(None, 101), None);
        assert_eq!(choose(Some(&sign(4, 8192, false)), 50), Some(8192));
    }
}
