use crate::settings::{origin, time, Config, SavedSession, Settings};
use garlic_relay_data::control::{Authority, Principal};
use garlic_state::StateError;
use reqwest::{blocking::Client, Method};
use serde_json::{json, Value};
use std::{
    env,
    io::Read,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};
use uuid::Uuid;

#[cfg(windows)]
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
    sync::Arc,
};

#[cfg(all(test, windows))]
#[test]
fn vpn_bootstrap_keeps_usable_routes_and_pins_exchange_socket() {
    let mut config = Config {
        control_urls: vec![
            "https://[2001:db8::1]:8443".into(),
            "https://198.51.100.2:8443".into(),
        ],
        storage_urls: vec!["https://127.0.0.1:18443".into()],
        ..Config::default()
    };
    let (api, bypass) = Api::for_windows_vpn(&mut config).unwrap();
    assert_eq!(config.control_urls, ["https://198.51.100.2:8443"]);
    assert_eq!(config.storage_urls, ["https://127.0.0.1:18443"]);
    assert_eq!(bypass, BTreeSet::from([Ipv4Addr::new(198, 51, 100, 2)]));
    assert_eq!(
        api.exchange_peers("198.51.100.2", 8443).unwrap(),
        ["198.51.100.2:8443".parse::<SocketAddr>().unwrap()]
    );
    assert!(api.exchange_peers("2001:db8::1", 8443).is_err());
}

#[derive(Clone)]
pub struct Api {
    client: Client,
    #[cfg(windows)]
    bootstrap: Option<Arc<BTreeMap<String, Vec<SocketAddr>>>>,
}
impl Api {
    pub fn new(cfg: &Config) -> Result<Self, StateError> {
        Ok(Self {
            client: Self::builder(cfg)?.build()?,
            #[cfg(windows)]
            bootstrap: None,
        })
    }

    fn builder(cfg: &Config) -> Result<reqwest::blocking::ClientBuilder, StateError> {
        let mut builder = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_millis(
                cfg.request_timeout_ms.clamp(100, 120_000),
            ));
        if let Some(path) = env::var_os("GARLIC_CA_CERT") {
            let roots = reqwest::Certificate::from_pem_bundle(&std::fs::read(path)?)?;
            if roots.is_empty() {
                return Err("empty GARLIC_CA_CERT".into());
            }
            for root in roots {
                builder = builder.add_root_certificate(root);
            }
        }
        Ok(builder)
    }

    pub(crate) fn exchange_peers(
        &self,
        host: &str,
        port: u16,
    ) -> Result<Vec<SocketAddr>, StateError> {
        #[cfg(windows)]
        if let Some(bootstrap) = &self.bootstrap {
            let peers = bootstrap
                .get(host)
                .ok_or("exchange host is not in the VPN configuration")?;
            return Ok(peers
                .iter()
                .map(|peer| SocketAddr::new(peer.ip(), port))
                .collect());
        }
        Ok((host, port).to_socket_addrs()?.collect())
    }

    #[cfg(windows)]
    pub(crate) fn for_windows_vpn(
        cfg: &mut Config,
    ) -> Result<(Self, BTreeSet<Ipv4Addr>), StateError> {
        use std::net::IpAddr;

        let mut bootstrap = BTreeMap::new();
        let mut bypass = BTreeSet::new();
        let mut builder = Self::builder(cfg)?;
        for uri in cfg.storage_urls.iter().chain(&cfg.control_urls) {
            let url = reqwest::Url::parse(&origin(uri)?)?;
            let host = url
                .host_str()
                .ok_or("missing gateway address")?
                .trim_matches(['[', ']']);
            if bootstrap.contains_key(host) {
                continue;
            }
            let Ok(resolved) = (host, 0).to_socket_addrs() else {
                continue;
            };
            let peers: Vec<_> = resolved
                .filter(|peer| peer.is_ipv4() || peer.ip().is_loopback())
                .collect();
            if peers.is_empty() {
                continue;
            }
            bypass.extend(peers.iter().filter_map(|peer| match peer.ip() {
                IpAddr::V4(ip) if !ip.is_loopback() => Some(ip),
                _ => None,
            }));
            builder = builder.resolve_to_addrs(host, &peers);
            bootstrap.insert(host.to_owned(), peers);
        }
        for urls in [&mut cfg.control_urls, &mut cfg.storage_urls] {
            urls.retain(|uri| {
                reqwest::Url::parse(uri).is_ok_and(|url| {
                    url.host_str()
                        .is_some_and(|host| bootstrap.contains_key(host.trim_matches(['[', ']'])))
                })
            });
        }
        if bootstrap.is_empty() {
            return Err("no usable bootstrap address for the configured VPN routes".into());
        }
        Ok((
            Self {
                client: builder.build()?,
                bootstrap: Some(Arc::new(bootstrap)),
            },
            bypass,
        ))
    }
    pub(crate) fn route_probe(&self, endpoint: &str, token: &str) -> Result<u64, StateError> {
        let response = self.client.get(format!("{}/v1/auth/info", origin(endpoint)?))
            .bearer_auth(token).timeout(Duration::from_secs(2)).send()?.error_for_status()?;
        let mut data = Vec::new();
        response.take(16385).read_to_end(&mut data)?;
        if data.len() > 16384 { return Err("route probe exceeded size limit".into()); }
        let info: Value = serde_json::from_slice(&data)?;
        let _: Authority = serde_json::from_value(info["authority"].clone())?;
        Ok(info["load_percent"].as_u64().unwrap_or(0).min(100))
    }

    pub fn call(
        &self,
        endpoint: &str,
        method: Method,
        path: &str,
        bearer: Option<&str>,
        device: Option<&str>,
        body: Option<&Value>,
    ) -> Result<Value, StateError> {
        let mut req = self
            .client
            .request(method, format!("{}{path}", origin(endpoint)?));
        if let Some(token) = bearer {
            req = req.bearer_auth(token);
        }
        if let Some(token) = device {
            req = req.header("x-garlic-device-token", token);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        let response = req.send().map_err(|e| {
            // The nested error distinguishes a TCP failure from a TLS trust
            // failure. Request URLs and authorization headers stay private.
            let e = e.without_url();
            let mut reason = e.to_string();
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                let detail = cause.to_string();
                if !detail.is_empty() && !reason.ends_with(&detail) {
                    reason.push_str(": ");
                    reason.push_str(&detail);
                }
                source = cause.source();
            }
            reason
        })?;
        let status = response.status();
        let relay = response.headers().get("x-garlic-relay-transport")
            .and_then(|h| h.to_str().ok()).filter(|v| matches!(*v, "relay-shaped-v2" | "relay-tls-v1" | "direct-egress")).map(str::to_owned);
        let mut bytes = vec![];
        response.take(4_194_305).read_to_end(&mut bytes)?;
        if bytes.len() > 4_194_304 {
            return Err("Garlic API response exceeded size limit".into());
        }
        if !status.is_success() {
            let value = serde_json::from_slice::<Value>(&bytes).unwrap_or_default();
            let detail = value["error"]
                .as_str()
                .or_else(|| value["error"]["code"].as_str())
                .unwrap_or("request declined");
            return Err(format!(
                "HTTP {}: {}",
                status.as_u16(),
                detail.chars().take(150).collect::<String>()
            )
            .into());
        }
        if bytes.is_empty() {
            return Ok(Value::Null);
        }
        let mut value: Value = serde_json::from_slice(&bytes)?;
        if path == "/v1/infinity/streams" && value.is_object() {
            value["relay_protocol"] = json!(relay);
        }
        Ok(value)
    }
    pub fn login(
        &self,
        settings: &Settings,
        url: &str,
        name: &str,
        password: &str,
    ) -> Result<Value, StateError> {
        let endpoint = origin(url)?;
        let result = self.call(
            &endpoint,
            Method::POST,
            "/v1/auth/login",
            None,
            None,
            Some(&json!({"username":name,"password":password})),
        )?;
        let grant: SavedSession = serde_json::from_value(result.clone())?;
        if !matches!(grant.principal, Principal::User { .. })
            || grant.token.is_empty()
            || grant.expires_at <= time()
        {
            return Err("invalid session in login response".into());
        }
        let confirmation = json!({"logged_in":endpoint,"principal":grant.principal});
        settings.remember(vec![(endpoint, grant)])?;
        Ok(confirmation)
    }
    pub fn connect(&self, settings: &Settings, url: &str, key: &str) -> Result<Value, StateError> {
        let endpoint = origin(url)?;
        let account = self.call(&endpoint, Method::GET, "/v1/auth/me", Some(key), None, None)?;
        let principal: Principal = serde_json::from_value(account["principal"].clone())?;
        if !matches!(principal, Principal::User { .. }) {
            return Err("server did not return a user account".into());
        }
        let directory = self.call(&endpoint, Method::GET, "/v1/network", Some(key), None, None)?;
        if directory["version"] != 1 {
            return Err("unsupported Garlic network directory".into());
        }
        let mut controls = vec![];
        let mut exits = vec![];
        for node in directory["nodes"]
            .as_array()
            .ok_or("missing network nodes")?
        {
            let dest = match node["role"].as_str() {
                Some("ru") => &mut controls,
                Some("exit") => &mut exits,
                _ => continue,
            };
            let address = origin(node["url"].as_str().ok_or("missing node address")?)?;
            if !dest.contains(&address) {
                dest.push(address);
            }
        }
        if controls.is_empty() {
            controls.push(endpoint.clone());
        }
        let grant = SavedSession {
            token: key.into(),
            expires_at: u64::MAX,
            principal,
            authority: Authority {
                owner_id: Uuid::nil(),
                epoch: 0,
            },
        };
        let entries = std::iter::once(&endpoint)
            .chain(&controls)
            .chain(&exits)
            .map(|u| (u.clone(), grant.clone()))
            .collect();
        settings.remember(entries)?;
        settings.cache("account-cache.json", &account)?;
        settings.cache("network.json", &directory)?;
        let cfg = settings.edit(|cfg| {
            cfg.storage_urls = exits.clone();
            cfg.control_urls = controls.clone();
            Ok(())
        })?;
        garlic_state::save(&settings.side("preferences.json"), &cfg)?;
        Ok(
            json!({"ok":true,"origin":endpoint,"account":account,"control_urls":controls,"direct_urls":exits,"network_id":directory["network_id"]}),
        )
    }
    pub fn admin(
        &self,
        settings: &Settings,
        cfg: &Config,
        op: &str,
        body: Option<&Value>,
    ) -> Result<Value, StateError> {
        if op == "network-set" {
            if let Some(legacy) = body.and_then(|value| value.get("legacy")) {
                return self.legacy_admin(settings, cfg, legacy);
            }
        }
        let store = settings.sessions()?;
        let (method, path) = match op {
            "account" => (Method::GET, "/v1/auth/me"),
            "list" => (Method::GET, "/v1/auth/users"),
            "manage" => (Method::POST, "/v1/auth/users"),
            "network" => (Method::GET, "/v1/network"),
            "network-set" => (Method::POST, "/v1/admin/network/policy"),
            _ => return Err("unknown administration operation".into()),
        };
        let mut last: StateError = "no saved session".into();
        let urls = cfg.control_urls.iter().chain(&cfg.storage_urls);
        for endpoint in urls {
            let Some(session) = store
                .entries
                .get(endpoint)
                .filter(|s| s.expires_at > time())
            else {
                continue;
            };
            match self.call(
                endpoint,
                method.clone(),
                path,
                Some(&session.token),
                None,
                body,
            ) {
                Ok(v) => {
                    return Ok(match op {
                        "account" => {
                            settings.cache("account-cache.json", &v)?;
                            json!({"account":v,"origin":endpoint})
                        }
                        "list" => json!({"users":v}),
                        "network" => json!({"network":v}),
                        _ => v,
                    })
                }
                Err(e) => {
                    // A submitted mutation may already have taken effect when the response is lost.
                    if method != Method::GET
                        || !garlic_client_logic::retry_allowed(Some(&e.to_string()))
                    {
                        return Err(e);
                    }
                    last = e;
                }
            }
        }
        Err(last)
    }

    fn legacy_admin(
        &self,
        settings: &Settings,
        cfg: &Config,
        request: &Value,
    ) -> Result<Value, StateError> {
        let request = request
            .as_object()
            .ok_or("invalid legacy subscription request")?;
        let selected = request
            .get("origin")
            .map(|value| origin(value.as_str().ok_or("invalid subscription origin")?))
            .transpose()?;
        let operation = request.get("operation");
        if operation.is_some_and(|value| !value.is_object())
            || (operation.is_some() && selected.is_none())
        {
            return Err("subscription changes require a selected origin and an operation".into());
        }
        let method = if operation.is_some() {
            Method::POST
        } else {
            Method::GET
        };
        let store = settings.sessions()?;
        let mut last: StateError = "no saved session for a subscription service".into();
        for endpoint in cfg.control_urls.iter().chain(&cfg.storage_urls) {
            if selected.as_ref().is_some_and(|url| url != endpoint) {
                continue;
            }
            let Some(session) = store
                .entries
                .get(endpoint)
                .filter(|session| session.expires_at > time())
            else {
                continue;
            };
            match self.call(
                endpoint,
                method.clone(),
                "/v1/admin/legacy",
                Some(&session.token),
                None,
                operation,
            ) {
                Ok(value) if method == Method::GET => {
                    return Ok(json!({"ok":true,"origin":endpoint,"legacy":value}));
                }
                Ok(value) => return Ok(value),
                Err(error)
                    if method == Method::GET
                        && selected.is_none()
                        && garlic_client_logic::retry_allowed(Some(&error.to_string())) =>
                {
                    last = error;
                }
                Err(error) => return Err(error),
            }
        }
        Err(last)
    }

    pub fn proxy_status(&self, address: std::net::SocketAddr) -> Result<Value, StateError> {
        if !address.ip().is_loopback() {
            return Err("expected loopback proxy address".into());
        }
        self.call(
            &format!("http://{address}"),
            Method::GET,
            "/status",
            None,
            None,
            None,
        )
    }

    pub fn takeover(&self, settings: &Settings, cfg: &Config) -> Result<Value, StateError> {
        let store = settings.sessions()?;
        let devices = crate::device::Devices::new(self.clone(), settings)?;
        let mut last: StateError = "no session".into();
        for endpoint in cfg.control_urls.iter().chain(&cfg.storage_urls) {
            let Some(session) = store
                .entries
                .get(endpoint)
                .filter(|s| s.expires_at > time())
            else {
                continue;
            };
            match devices.claim(endpoint, session, true) {
                Ok(_) => return Ok(json!({"ok":true})),
                Err(e) if garlic_client_logic::retry_allowed(Some(&e.to_string())) => last = e,
                Err(e) => return Err(e),
            }
        }
        Err(last)
    }
}
