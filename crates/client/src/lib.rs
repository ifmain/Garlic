#[cfg(any(target_os = "android", feature = "jni-test"))]
mod android;
mod api;
mod device;
mod jobs;
mod metrics;
mod journal;
pub use jobs::run_job_cli;
mod proxy;
mod settings;
#[cfg(any(target_os = "android", windows))]
mod tunnel;
mod wire;

use api::Api;
use garlic_state::StateError;
use serde_json::{json, Value};
use settings::Settings;
use std::{
    io::{self, Read},
    net::SocketAddr,
};

fn input() -> Result<String, StateError> {
    let mut value = String::new();
    io::stdin().lock().take(65_537).read_to_string(&mut value)?;
    if value.len() > 65_536 {
        return Err("input exceeds size limit".into());
    }
    Ok(value.trim_end_matches(['\r', '\n']).to_owned())
}

fn default_proxy_bind() -> Result<SocketAddr, StateError> {
    let bind = std::env::var("GARLIC_PROXY_BIND").unwrap_or_else(|_| "127.0.0.1:1080".into());
    Ok(bind.parse()?)
}

pub fn run(args: &[&str]) -> Result<Value, StateError> {
    if args == ["version"] {
        return Ok(
            json!({"version":"0.9.2","platform":std::env::consts::OS,"architecture":std::env::consts::ARCH,"core_abi":1,"alpha_profiles":cfg!(feature="alpha")}),
        );
    }
    let settings = Settings::locate()?;
    if let ["core-update", rest @ ..] = args {
        return run_update(&settings, false, rest);
    }
    let cfg = settings.load_effective()?;
    match args {
        ["config", "show"] => return Ok(serde_json::to_value(cfg)?),
        ["stop"] => {
            settings.stop_proxies()?;
            return Ok(json!({"ok":true}));
        }
        ["signout"] => {
            settings.signout()?;
            return Ok(json!({"ok":true}));
        }
        ["config", "set", name, value] => {
            return Ok(serde_json::to_value(
                settings.edit(|cfg| cfg.set(name, value))?,
            )?)
        }
        ["plugin-key", url, "--key-stdin"] => {
            if !settings.side("installed-gpf").is_file() { return Err("plugin is not installed".into()) }
            let store=proxy::plugin_store(&settings);
            std::fs::create_dir_all(store.config.parent().ok_or("invalid plugin directory")?)?;
            return store.save_key(&input()?, vec![settings::origin(url)?]);
        }
        ["set-key", value, urls @ ..] if urls.len() <= 2 => {
            let direct = settings::origin(
                urls.first()
                    .copied()
                    .ok_or("set-key requires your server URL")?,
            )?;
            let control = settings::origin(
                urls.get(1)
                    .copied()
                    .unwrap_or(direct.as_str()),
            )?;
            let key = if *value == "--key-stdin" {
                input()?
            } else {
                (*value).to_owned()
            };
            let origins = if direct == control {
                vec![direct.clone()]
            } else {
                vec![direct.clone(), control.clone()]
            };
            let result = settings.save_key(&key, origins)?;
            settings.edit(|config| {
                config.mode = "auto".into();
                config.storage_urls = vec![direct];
                config.control_urls = vec![control];
                Ok(())
            })?;
            return Ok(result);
        }
        _ => {}
    }
    #[cfg(windows)]
    if args == ["vpn"] {
        match std::fs::remove_file(settings.side("vpn-error.json")) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
        journal::write(&settings.config, "vpn_start", "");
        let status = tunnel::run_vpn(settings.clone(), cfg);
        if let Err(error) = &status {
            journal::write(&settings.config, "vpn_error", journal::reason(&error.to_string()));
            let _ = settings.cache("vpn-error.json", &json!({"error":error.to_string()}));
        }
        status?;
        return Ok(json!({"ok":true}));
    }
    let api = Api::new(&cfg)?;
    match args {
        ["proxy-status", address] => api.proxy_status(address.parse()?),
        ["proxy-status"] => api.proxy_status(default_proxy_bind()?),
        ["takeover"] => api.takeover(&settings, &cfg),
        ["proxy"] => {
            proxy::run(&settings, &cfg, api, default_proxy_bind()?)?;
            Ok(json!({"ok":true}))
        }
        ["proxy", "--bind", address] => {
            proxy::run(&settings, &cfg, api, address.parse()?)?;
            Ok(json!({"ok":true}))
        }
        ["connect", endpoint, "--key-stdin"] => api.connect(&settings, endpoint, &input()?),
        ["login", endpoint, name, "--password-stdin"] => {
            api.login(&settings, endpoint, name, &input()?)
        }
        [op @ ("me" | "users" | "logout"), endpoint] => {
            let url = settings::origin(endpoint)?;
            let store = settings.sessions()?;
            let session = store.entries.get(&url).ok_or("not logged in for this URL")?;
            if *op == "logout" {
                api.call(&url, reqwest::Method::POST, "/v1/auth/logout", Some(&session.token), None, None)?;
                settings.forget(&url, &session.token)?;
                Ok(json!({"logged_out":url}))
            } else {
                let path = if *op == "me" {"/v1/auth/me"} else {"/v1/auth/users"};
                api.call(&url, reqwest::Method::GET, path, Some(&session.token), None, None)
            }
        }
        ["account"] => api.admin(&settings, &cfg, "account", None),
        ["admin", op @ ("list" | "network")] => api.admin(&settings, &cfg, op, None),
        ["admin", op @ ("manage" | "network-set")] => {
            api.admin(&settings, &cfg, op, Some(&serde_json::from_str(&input()?)?))
        }
        ["admin", action @ ("create" | "key" | "enable" | "disable" | "remove"), username, options @ ..] => {
            let mut body = json!({"action":action, "username":username,"new_username":null,"password":null,
                "role":null,"subscription":null,"ttl_secs":null,"custom_token":null});
            match *action {
                "create" if options.len() <= 2 => {
                    body["subscription"] = json!(options.first().copied().unwrap_or("unlimited"));
                    body["role"] = json!(options.get(1).copied().unwrap_or("client"));
                }
                "key" if options.is_empty() => {
                    body["action"] = json!("issue_key");
                    body["ttl_secs"] = json!(31_536_000);
                }
                "enable" | "disable" | "remove" if options.is_empty() => (),
                _ => return Err("invalid user management arguments".into()),
            }
            api.admin(&settings, &cfg, "manage", Some(&body))
        }
        _ => Err(
            "usage: garlic-internal-network version | config | connect | set-key | login | logout | me | users | account | admin | proxy | proxy-status | vpn | stop | signout | takeover | core-update | echo | uppercase | reverse | http | status"
                .into(),
        ),
    }
}

pub fn run_ui_update(args: &[&str]) -> Result<Value, StateError> {
    run_update(&Settings::locate()?, true, args)
}

fn run_update(settings: &Settings, ui: bool, args: &[&str]) -> Result<Value, StateError> {
    if cfg!(feature = "alpha") {
        return Err("Garlic Alpha accepts signed declarative profiles only".into());
    }
    use release_cache::{Cache, Source, Target};
    let target = Target::for_platform(if ui { "windows-ui" } else { "windows-x86_64" })?;
    let home = settings
        .config
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let cache = Cache::open(home, target)?;
    match args {
        ["select"] => cache.select(),
        ["confirm", sequence] => cache.confirm(sequence.parse()?),
        ["stage"] => {
            let sources = settings
                .sessions()?
                .entries
                .into_iter()
                .filter(|(_, session)| session.expires_at > settings::time())
                .map(|(origin, session)| Source {
                    origin,
                    token: session.token,
                })
                .collect::<Vec<_>>();
            release_cache::stage(&cache, &sources)
        }
        _ => Err("expected select, stage, or confirm with a sequence".into()),
    }
}
