//! Authenticated release manifests and the native five-field update cache.
mod download;
pub mod profile;
pub mod guest;
pub mod canvas;
use base64::{engine::general_purpose::STANDARD, Engine};
pub use download::{stage, Source};
use garlic_state::{edit_or_create, StateError};
use ring::{digest, signature};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

// Raw Ed25519 public key, selected by the distributor at build time.
fn publisher_key() -> Option<[u8; 32]> {
    STANDARD.decode(option_env!("GARLIC_PUBLISHER_KEY_B64")?).ok()?.try_into().ok()
}

#[derive(Clone, Copy)]
pub struct Target {
    pub platform: &'static str,
    pub manifest: &'static str,
    pub extension: &'static str,
    pub folder: &'static str,
}
impl Target {
    pub fn for_platform(platform: &str) -> Result<Self, StateError> {
        let (platform, manifest, extension, folder) = match platform {
            "android-aarch64" => ("android-aarch64", "android-aarch64.json", "so", "cores-v1"),
            "android-arm" => ("android-arm", "android-arm.json", "so", "cores-v1"),
            "android-x86_64" => ("android-x86_64", "android-x86_64.json", "so", "cores-v1"),
            "windows-x86_64" => ("windows-x86_64", "windows-x86_64.json", "exe", "cores-v1"),
            "windows-ui" => (
                "windows-ui",
                "b1c1d2a8bf110144bb1a11a5a850e4e0f9f7c21ef56c900dd1a530dfb108ad59",
                "dll",
                "ui-v1",
            ),
            "android-ui" => (
                "android-ui",
                "5080ae6ecda28043063d8c278df34361681d614107691ee38147eade0fde347d",
                "dex",
                "ui-v1",
            ),
            _ => return Err("unsupported update target".into()),
        };
        Ok(Self {
            platform,
            manifest,
            extension,
            folder,
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Envelope {
    pub payload: String,
    pub signature: String,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Release {
    pub abi: u32,
    pub platform: String,
    pub version: String,
    pub sequence: u64,
    pub size: u64,
    pub sha256: String,
    pub expires: u64,
}
fn version(s: &str) -> Option<[u64; 3]> {
    let mut fields = s.split('.');
    let numbers = [
        fields.next()?.parse().ok()?,
        fields.next()?.parse().ok()?,
        fields.next()?.parse().ok()?,
    ];
    fields.next().is_none().then_some(numbers)
}
pub fn verify(env: &Envelope, target: Target) -> Result<Release, StateError> {
    let payload = STANDARD.decode(&env.payload)?;
    let sig = STANDARD.decode(&env.signature)?;
    signature::UnparsedPublicKey::new(&signature::ED25519, publisher_key().ok_or("no publisher key configured")?)
        .verify(&payload, &sig)
        .map_err(|_| "incorrect release signature")?;
    let rel: Release = serde_json::from_slice(&payload)?;
    if rel.platform != target.platform
        || rel.abi != 1
        || rel.sequence == 0
        || rel.size == 0
        || rel.size > 128 * 1024 * 1024
        || rel.sha256.len() != 64
        || !rel
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !version(&rel.version).is_some_and(|v| v >= [0, 9, 2])
    {
        return Err("incompatible release manifest".into());
    }
    Ok(rel)
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn digest_file(path: &Path, rel: &Release) -> Result<(), StateError> {
    let mut f = File::open(path)?;
    if f.metadata()?.len() != rel.size {
        return Err("incorrect release size".into());
    }
    let mut ctx = digest::Context::new(&digest::SHA256);
    let mut chunk = [0u8; 65536];
    loop {
        let n = f.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        ctx.update(&chunk[..n]);
    }
    if hex(ctx.finish().as_ref()) != rel.sha256 {
        return Err("incorrect release hash".into());
    }
    Ok(())
}
#[derive(Default, Serialize, Deserialize)]
struct State {
    highest: u64,
    pending: Option<Envelope>,
    trial: Option<Envelope>,
    active: Option<Envelope>,
    previous: Option<Envelope>,
}
pub struct Cache {
    dir: PathBuf,
    target: Target,
}
impl Cache {
    pub fn open(profile: &Path, target: Target) -> Result<Self, StateError> {
        let dir = profile.join(target.folder);
        std::fs::create_dir_all(&dir)?;
        Ok(Self { dir, target })
    }
    fn edit<T>(
        &self,
        f: impl FnOnce(&mut State) -> Result<T, StateError>,
    ) -> Result<T, StateError> {
        edit_or_create(&self.dir.join("state.json"), || Ok(State::default()), f)
    }
    fn path(&self, rel: &Release) -> PathBuf {
        self.dir
            .join(format!("{}.{}", rel.sha256, self.target.extension))
    }
    fn selected(&self, env: &Envelope, trial: bool) -> Result<Value, StateError> {
        let rel = verify(env, self.target)?;
        let path = self.path(&rel);
        digest_file(&path, &rel)?;
        Ok(
            json!({"path":path,"sha256":rel.sha256,"version":rel.version,"sequence":rel.sequence,"trial":trial}),
        )
    }
    pub fn select(&self) -> Result<Value, StateError> {
        self.edit(|state| {
            // An unconfirmed trial means the previous application launch failed.
            state.trial = None;
            if let Some(pending) = state.pending.take() {
                if let Ok(choice) = self.selected(&pending, true) {
                    state.trial = Some(pending);
                    return Ok(choice);
                }
            }
            for env in [&state.active, &state.previous].into_iter().flatten() {
                if let Ok(choice) = self.selected(env, false) {
                    return Ok(choice);
                }
            }
            Ok(json!({"bundled":true}))
        })
    }
    pub fn confirm(&self, seq: u64) -> Result<Value, StateError> {
        self.edit(|state| {
            if let Some(trial) = state.trial.as_ref() {
                if verify(trial, self.target)?.sequence != seq {
                    return Err("trial has changed".into());
                }
                state.previous = state.active.take();
                state.active = state.trial.take();
            }
            Ok(json!({"ok":true}))
        })
    }
}
