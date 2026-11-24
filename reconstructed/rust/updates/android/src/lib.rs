use garlic_state::StateError;
use jni_sys::{jclass, jint, jstring, JNIEnv};
use release_cache::{Cache, Source, Target};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::Path,
    ptr,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Deserialize)]
struct Session {
    token: String,
    expires_at: u64,
}
#[derive(Deserialize)]
struct Sessions {
    version: u32,
    entries: BTreeMap<String, Session>,
}

fn update(dir: &Path, action: &str) -> Result<Value, StateError> {
    let (platform, command) = if let Some(command) = action.strip_prefix("ui-") {
        ("android-ui", command)
    } else {
        let platform = match std::env::consts::ARCH {
            "aarch64" => "android-aarch64",
            "arm" => "android-arm",
            "x86_64" => "android-x86_64",
            _ => return Err("unsupported Android architecture".into()),
        };
        (platform, action)
    };
    let cache = Cache::open(dir, Target::for_platform(platform)?)?;
    match command {
        "select" => cache.select(),
        "stage" => {
            let sessions = match std::fs::File::open(dir.join("client.sessions.json")) {
                Ok(file) => serde_json::from_reader::<_, Sessions>(file)?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(json!({"staged":false}))
                }
                Err(e) => return Err(e.into()),
            };
            if sessions.version != 1 {
                return Err("unsupported session store".into());
            }
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let sources = sessions
                .entries
                .into_iter()
                .filter(|(_, s)| s.expires_at > now)
                .map(|(origin, s)| Source {
                    origin,
                    token: s.token,
                })
                .collect::<Vec<_>>();
            release_cache::stage(&cache, &sources)
        }
        other => cache.confirm(other.parse()?),
    }
}
unsafe fn text(env: *mut JNIEnv, s: jstring) -> Result<String, StateError> {
    if s.is_null() {
        return Err("null update argument".into());
    }
    let j = &(**env).v1_2;
    let len = (j.GetStringLength)(env, s);
    if !(0..=65536).contains(&len) {
        return Err("invalid update argument length".into());
    }
    let chars = (j.GetStringChars)(env, s, ptr::null_mut());
    if chars.is_null() {
        return Err("could not read update argument".into());
    }
    let value = String::from_utf16(std::slice::from_raw_parts(chars, len as usize));
    (j.ReleaseStringChars)(env, s, chars);
    Ok(value?)
}

/// # Safety
/// ART supplies a valid per-thread JNI environment and local string references.
#[no_mangle]
pub unsafe extern "system" fn Java_org_garlic_client_CoreLoader_update(
    env: *mut JNIEnv,
    _class: jclass,
    dir: jstring,
    command: jstring,
) -> jstring {
    let result = (|| update(Path::new(&text(env, dir)?), &text(env, command)?))()
        .unwrap_or_else(|e| json!({"ok":false,"error":e.to_string()}));
    let chars: Vec<u16> = result.to_string().encode_utf16().collect();
    ((**env).v1_2.NewString)(env, chars.as_ptr(), chars.len() as jint)
}
