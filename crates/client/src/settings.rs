use garlic_relay_data::control::{Authority, Principal};
use garlic_state::{self as files, StateError};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    env, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

#[cfg(windows)]
pub(crate) mod windows;

pub fn time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn origin(value: &str) -> Result<String, StateError> {
    let mut u = Url::parse(value)?;
    let local = match u.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        None => false,
    };
    if (u.scheme() != "https" && !(local && u.scheme() == "http"))
        || u.host().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.query().is_some()
        || u.fragment().is_some()
        || u.path() != "/"
    {
        return Err("expected a Garlic HTTPS server origin".into());
    }
    u.set_path("");
    Ok(u.as_str().trim_end_matches('/').to_owned())
}
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub version: u32,
    pub mode: String,
    pub storage_urls: Vec<String>,
    pub control_urls: Vec<String>,
    pub request_timeout_ms: u64,
    pub stall_timeout_ms: u64,
    pub total_timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub cooldown_ms: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            mode: "auto".into(),
            storage_urls: vec!["http://127.0.0.1:8090".into()],
            control_urls: vec!["http://127.0.0.1:8080".into()],
            request_timeout_ms: 5000,
            stall_timeout_ms: 5000,
            total_timeout_ms: 45000,
            poll_interval_ms: 100,
            cooldown_ms: 30000,
        }
    }
}
impl Config {
    pub fn set(&mut self, name: &str, value: &str) -> Result<(), StateError> {
        match name {
            "mode" => {
                if !matches!(value, "auto" | "ru" | "direct") {
                    return Err("mode must be auto, direct or ru".into());
                }
                self.mode = value.to_owned();
            }
            "control_urls" | "storage_urls" => {
                let mut urls = vec![];
                for raw in value.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    let endpoint = origin(raw)?;
                    if urls.contains(&endpoint) {
                        return Err("duplicate server origin".into());
                    }
                    urls.push(endpoint);
                }
                if name == "control_urls" {
                    self.control_urls = urls;
                } else {
                    self.storage_urls = urls;
                }
            }
            _ => {
                let (slot, range) = match name {
                    "request_timeout_ms" => (&mut self.request_timeout_ms, 0..=u64::MAX),
                    "stall_timeout_ms" => (&mut self.stall_timeout_ms, 100..=300_000),
                    "total_timeout_ms" => (&mut self.total_timeout_ms, 1..=600_000),
                    "poll_interval_ms" => (&mut self.poll_interval_ms, 1..=5_000),
                    "cooldown_ms" => (&mut self.cooldown_ms, 1..=600_000),
                    _ => return Err("unknown client setting".into()),
                };
                let value = value.parse()?;
                if !range.contains(&value) {
                    return Err("invalid client preferences".into());
                }
                *slot = value;
            }
        }
        Ok(())
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub token: String,
    pub expires_at: u64,
    pub principal: Principal,
    pub authority: Authority,
}
#[derive(Serialize, Deserialize)]
pub struct Sessions {
    pub version: u32,
    pub entries: BTreeMap<String, SavedSession>,
}
impl Default for Sessions {
    fn default() -> Self {
        Self {
            version: 1,
            entries: BTreeMap::new(),
        }
    }
}
#[derive(Clone)]
pub struct Settings {
    pub config: PathBuf,
}
impl Settings {
    pub fn locate() -> Result<Self, StateError> {
        let config = if let Some(path) = env::var_os("GARLIC_CLIENT_CONFIG") {
            PathBuf::from(path)
        } else if cfg!(windows) {
            PathBuf::from(env::var_os("APPDATA").ok_or("APPDATA is unavailable")?)
                .join(if cfg!(feature = "alpha") { "Garlic Alpha/client.json" } else { "Garlic/client.json" })
        } else {
            let base = env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| env::var_os("HOME").map(|s| PathBuf::from(s).join(".config")))
                .ok_or("configuration directory unavailable")?;
            base.join("garlic/client.json")
        };
        if let Some(parent) = config.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        Ok(Self { config })
    }
    pub fn side(&self, ext: &str) -> PathBuf {
        self.config.with_extension(ext)
    }
    pub fn load(&self) -> Result<Config, StateError> {
        let c: Config = read_default(&self.config)?;
        if c.version != 1 {
            return Err("unsupported client configuration".into());
        }
        Ok(c)
    }
    pub fn load_effective(&self) -> Result<Config, StateError> {
        let mut config = self.load()?;
        for (variable, field) in [
            ("GARLIC_STORAGE_URLS", "storage_urls"),
            ("GARLIC_CONTROL_URLS", "control_urls"),
            ("GARLIC_ROUTE_MODE", "mode"),
            ("GARLIC_CLIENT_COOLDOWN_MS", "cooldown_ms"),
            ("GARLIC_CLIENT_POLL_INTERVAL_MS", "poll_interval_ms"),
            ("GARLIC_CLIENT_STALL_TIMEOUT_MS", "stall_timeout_ms"),
            ("GARLIC_CLIENT_TOTAL_TIMEOUT_MS", "total_timeout_ms"),
        ] {
            if let Ok(value) = env::var(variable) {
                config.set(field, &value)?;
            }
        }
        Ok(config)
    }
    pub fn edit(
        &self,
        op: impl FnOnce(&mut Config) -> Result<(), StateError>,
    ) -> Result<Config, StateError> {
        files::edit_or_create(
            &self.config,
            || Ok(Config::default()),
            |cfg| {
                if cfg.version != 1 {
                    return Err("unsupported client configuration".into());
                }
                op(cfg)?;
                Ok(cfg.clone())
            },
        )
    }
    pub fn sessions(&self) -> Result<Sessions, StateError> {
        let saved: Sessions = read_default(&self.side("sessions.json"))?;
        if saved.version != 1 {
            return Err("unsupported session store".into());
        }
        #[cfg(windows)]
        let saved = {
            let mut store = saved;
            for session in store.entries.values_mut() {
                session.token = windows::decode(&session.token)?;
            }
            store
        };
        Ok(saved)
    }
    pub fn remember(&self, entries: Vec<(String, SavedSession)>) -> Result<(), StateError> {
        #[cfg(windows)]
        let entries = entries
            .into_iter()
            .map(|(url, mut session)| {
                session.token = windows::encode(&session.token)?;
                Ok((url, session))
            })
            .collect::<Result<Vec<_>, StateError>>()?;
        files::edit_or_create(
            &self.side("sessions.json"),
            || Ok(Sessions::default()),
            |store| {
                if store.version != 1 {
                    return Err("unsupported session store".into());
                }
                store.entries.extend(entries);
                Ok(())
            },
        )
    }
    pub fn save_key(&self, key: &str, origins: Vec<String>) -> Result<Value, StateError> {
        let key = key.trim();
        if !(1..=128).contains(&key.len()) {
            return Err("expected an application key between 1 and 128 bytes".into());
        }
        let user_id = key
            .strip_prefix("gu_")
            .and_then(|value| value.split_once('_'))
            .filter(|(user, _)| user.len() == 32)
            .and_then(|(user, _)| Uuid::parse_str(user).ok())
            .unwrap_or_default();
        let expires_at = time().saturating_add(31_536_000);
        // These fields allow older clients to read the local profile. The
        // coordinator still validates the key, account and lease on requests.
        let grant = SavedSession {
            token: key.to_owned(),
            expires_at,
            principal: Principal::User {
                user_id,
                username: "key-user".into(),
                role: garlic_relay_data::control::Role::Client,
            },
            authority: Authority {
                owner_id: Uuid::nil(),
                epoch: 0,
            },
        };
        let entries = origins
            .iter()
            .map(|url| (url.clone(), grant.clone()))
            .collect();
        self.remember(entries)?;
        Ok(
            serde_json::json!({"ok":true,"origins":origins,"expires_at":expires_at,"user_id":user_id}),
        )
    }
    pub fn cache(&self, name: &str, value: &Value) -> Result<(), StateError> {
        files::save(&self.side(name), value)
    }
    pub fn forget(&self, url: &str, token: &str) -> Result<(), StateError> {
        files::edit_or_create(
            &self.side("sessions.json"),
            || Ok(Sessions::default()),
            |sessions| {
                if sessions.version != 1 {
                    return Err("unsupported session store".into());
                }
                if let Some(session) = sessions.entries.get(url) {
                    #[cfg(windows)]
                    let saved = windows::decode(&session.token)?;
                    #[cfg(not(windows))]
                    let saved = &session.token;
                    if saved == token {
                        sessions.entries.remove(url);
                    }
                }
                Ok(())
            },
        )
    }

    pub fn stop_generation(&self) -> Result<Uuid, StateError> {
        read_default(&self.side("proxy-generation.json"))
    }

    /// All proxy instances in this profile observe the generation. This also
    /// works from a non-elevated UI without terminating a process by PID.
    pub fn stop_proxies(&self) -> Result<(), StateError> {
        files::edit_or_create(
            &self.side("proxy-generation.json"),
            || Ok(Uuid::nil()),
            |value| {
                *value = Uuid::new_v4();
                Ok(())
            },
        )
    }

    pub fn signout(&self) -> Result<(), StateError> {
        self.stop_proxies()?;
        files::edit_or_create(
            &self.side("sessions.json"),
            || Ok(Sessions::default()),
            |s| {
                if s.version != 1 {
                    return Err("unsupported session store".into());
                }
                s.entries.clear();
                Ok(())
            },
        )?;
        for name in [
            "account-cache.json",
            "network.json",
            "device.json",
            "device-origins.json",
            "preferences.json",
        ] {
            match std::fs::remove_file(self.side(name)) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                other => other?,
            }
        }
        Ok(())
    }
}
fn read_default<T: serde::de::DeserializeOwned + Default>(path: &Path) -> Result<T, StateError> {
    match files::read(path) {
        Err(e)
            if e.downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::NotFound) =>
        {
            Ok(T::default())
        }
        other => other,
    }
}
