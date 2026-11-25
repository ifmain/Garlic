use crate::{
    api::Api,
    device::Devices,
    metrics::{Metrics, Traffic},
    settings::{time, Config, SavedSession, Settings},
    wire::Exchange,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use garlic_relay_data::control::{
    valid_connect_authority, Authority, Capability, Principal, Route, StreamOpen,
};
use garlic_state::StateError;
use reqwest::Method;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{self, Read, Write},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

mod health;
mod connector;

struct Endpoint {
    slot: Option<String>,
    url: String,
    route: Route,
    session: SavedSession,
}
struct Proxy {
    settings: Settings,
    api: Api,
    devices: Devices,
    endpoints: Vec<Endpoint>,
    peers: Vec<connector::Peer>,
    required_exit: Option<String>,
    health: health::RouteHealth,
    hooks: Vec<(String,release_cache::guest::Guest)>,
    failed: Mutex<BTreeMap<String, Instant>>,
    lifetime: Duration,
    timeout: Duration,
    cooldown: Duration,
    stop: Arc<AtomicBool>,
    tunnel_ready: Option<Arc<AtomicBool>>,
    active: AtomicUsize,
    address: SocketAddr,
    metrics: Metrics,
    traffic: Arc<Traffic>,
}
enum Request {
    Connect(String, bool),
    Status,
    Pac,
}

fn receive_request(s: &mut TcpStream) -> Result<Request, StateError> {
    let mut first = [0];
    s.read_exact(&mut first)?;
    if first[0] == 5 {
        let mut n = [0];
        s.read_exact(&mut n)?;
        let mut methods = vec![0; n[0] as usize];
        s.read_exact(&mut methods)?;
        if !methods.contains(&0) {
            s.write_all(&[5, 255])?;
            return Err("SOCKS authentication method unsupported".into());
        }
        s.write_all(&[5, 0])?;
        let mut header = [0; 4];
        s.read_exact(&mut header)?;
        if header[..3] != [5, 1, 0] {
            s.write_all(&[5, 7, 0, 1, 0, 0, 0, 0, 0, 0])?;
            return Err("SOCKS command unsupported".into());
        }
        let host = match header[3] {
            1 => {
                let mut ip = [0; 4];
                s.read_exact(&mut ip)?;
                Ipv4Addr::from(ip).to_string()
            }
            4 => {
                let mut ip = [0; 16];
                s.read_exact(&mut ip)?;
                format!("[{}]", Ipv6Addr::from(ip))
            }
            3 => {
                s.read_exact(&mut n)?;
                let mut name = vec![0; n[0] as usize];
                s.read_exact(&mut name)?;
                String::from_utf8(name)?
            }
            _ => return Err("unknown SOCKS address type".into()),
        };
        let mut port = [0; 2];
        s.read_exact(&mut port)?;
        return Ok(Request::Connect(
            format!("{host}:{}", u16::from_be_bytes(port)),
            true,
        ));
    }
    let mut head = vec![first[0]];
    // Peek and consume only headers. A client may already have sent tunneled bytes.
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() >= 8192 {
            return Err("proxy headers too large".into());
        }
        let mut byte = [0];
        s.read_exact(&mut byte)?;
        head.push(byte[0]);
    }
    let line = std::str::from_utf8(&head)?.lines().next().unwrap_or("");
    let mut words = line.split_ascii_whitespace();
    match (words.next(), words.next(), words.next(), words.next()) {
        (Some("CONNECT"), Some(authority), Some("HTTP/1.0" | "HTTP/1.1"), None) => {
            Ok(Request::Connect(authority.to_owned(), false))
        }
        (Some("GET"), Some("/status"), Some("HTTP/1.0" | "HTTP/1.1"), None) => Ok(Request::Status),
        (Some("GET"), Some("/proxy.pac"), Some("HTTP/1.0" | "HTTP/1.1"), None) => Ok(Request::Pac),
        _ => Err("expected HTTP CONNECT or SOCKS5".into()),
    }
}
impl Proxy {
    fn log_transport_error(&self, event: &'static str, endpoint: &Endpoint, error: &str) {
        let route = if endpoint.route == Route::Direct { "direct" } else { "ingress" };
        crate::journal::write(&self.settings.config, event,
            &format!("via={} cause={}", route, crate::journal::reason(error)));
    }

    fn status(&self) -> Value {
        let mut value = self.metrics.status(
            &self.traffic,
            self.address,
            self.endpoints.iter().map(|e| (e.url.as_str(), e.route)).chain(self.peers.iter().map(|p|(p.address.as_str(),Route::Direct))),
            &self.failed.lock().unwrap(),
            self.cooldown,
        );
        if let Some(message) = self.api.clock_warning() {
            value["clock_warning"] = Value::from(message);
        }
        if let Some(ready) = &self.tunnel_ready {
            let ready = ready.load(Ordering::Acquire);
            value["tunnel"] = Value::Bool(ready);
            value["vpn_initializing"] = Value::Bool(!ready);
            if !ready {
                value["network_ready"] = Value::Bool(false);
            }
        }
        value
    }
    fn connect(
        &self,
        endpoint: &Endpoint,
        dest: &str,
        local: &mut TcpStream,
    ) -> Result<(Exchange, Uuid, Option<String>), StateError> {
        let session = &endpoint.session;
        let Principal::User { user_id, .. } = session.principal else {
            return Err("user session required".into());
        };
        let probe = Instant::now();
        let info = self.api.call(
            &endpoint.url,
            Method::GET,
            "/v1/auth/info",
            Some(&session.token),
            None,
            None,
        )?;
        let ping = probe.elapsed().as_millis() as u64;
        let owner: Authority = serde_json::from_value(info["authority"].clone())?;
        let user_id = if user_id.is_nil() {
            let account = self.api.call(
                &endpoint.url,
                Method::GET,
                "/v1/auth/me",
                Some(&session.token),
                None,
                None,
            )?;
            match serde_json::from_value(account["principal"].clone())? {
                Principal::User { user_id, .. } if !user_id.is_nil() => user_id,
                _ => return Err("user session required".into()),
            }
        } else {
            user_id
        };
        let installed = release_cache::profile::Installed::default();
        #[cfg(feature = "alpha")]
        let installed = if std::env::var("GARLIC_ALPHA_SLOT").as_deref() != Ok("0")
            && !self.settings.side("profile-disabled").exists() {
            let network = owner.owner_id.to_string();
            let path = self.settings.side(&format!("profile-{network}.json"));
            let offer = info.get("record_profile").and_then(|v| serde_json::from_value(v.clone()).ok());
            release_cache::profile::install(&path, &network, offer.as_ref(), time())
        } else { installed };
        let protocol = match installed.module {
            Some(module) if info["exchange_module_versions"].as_array()
                .is_some_and(|versions| versions.iter().any(|v| v == module.version)) =>
                    crate::wire::ClientProtocol::Module(module),
            _ => crate::wire::ClientProtocol::negotiate(&info)?,
        };
        let mut device = if info["device_leases"] == true {
            self.devices.claim(&endpoint.url, session, false)?
        } else {
            None
        };
        let flow = Uuid::new_v4();
        let open = StreamOpen {
            control_version: 1,
            data_version: 1,
            node_id: format!("gin-{user_id}"),
            channel_id: Uuid::new_v4(),
            user_id,
            flow_id: flow,
            stream_id: Uuid::new_v4(),
            connect_authority: dest.into(),
            preferred_exit: self.required_exit.clone(),
            route: endpoint.route,
            capabilities: Capability::ALL.to_vec(),
            window_size: 1 << 20,
            deadline_unix_ms: time()
                .saturating_add(self.lifetime.as_secs())
                .saturating_mul(1000),
            authority: owner,
        };
        let body = serde_json::to_value(&open)?;
        let open_with = |proof: Option<&str>| self.api.call(
            &endpoint.url,
            Method::POST,
            "/v1/infinity/streams",
            Some(&session.token),
            proof,
            Some(&body),
        );
        let opened = match open_with(device.as_deref()) {
            Err(error) if device.is_some() && error.to_string() == "HTTP 409: device_in_use" => {
                // The authority refused the proof before allocating a stream.
                // Claim normally so another device still requires an explicit takeover.
                self.devices.forget_rejected(&endpoint.url, device.as_deref().unwrap());
                device = self.devices.claim(&endpoint.url, session, false)?;
                open_with(device.as_deref())?
            }
            result => result?,
        };
        let setup = (|| {
            if opened["flow_id"] != flow.to_string()
                || opened["stream_id"] != open.stream_id.to_string()
                || opened["authority"] != serde_json::to_value(&open.authority)?
            {
                return Err("coordinator returned mismatched stream identifiers".into());
            }
            let mut exchange = Exchange::new(
                &self.api,
                protocol,
                &endpoint.url,
                &session.token,
                device.as_deref(),
                flow,
                self.timeout,
                self.traffic.clone(),
            )?;
            exchange.open(local, self.timeout)?;
            exchange.record_limit = installed.frame_bytes;
            Ok(exchange)
        })();
        match setup {
            Ok(exchange) => {
                let route_changed = self.metrics.opened(
                    &endpoint.url,
                    endpoint.route,
                    &info["server_version"],
                    ping,
                    protocol.name(),
                    exchange.record_limit,
                    &opened["relay_protocol"],
                );
                if route_changed {
                    let via = if endpoint.route == Route::Direct { "direct" } else { "ingress" };
                    crate::journal::write(&self.settings.config, "route_selected",
                        &format!("via={} protocol={} ping_ms={}", via, protocol.name(), ping));
                }
                Ok((exchange, open.stream_id, device))
            }
            Err(e) => {
                self.close(endpoint, open.stream_id, device.as_deref());
                Err(e)
            }
        }
    }
    fn close(&self, endpoint: &Endpoint, id: Uuid, device: Option<&str>) {
        let _ = self.api.call(
            &endpoint.url,
            Method::DELETE,
            &format!("/v1/infinity/streams/{id}"),
            Some(&endpoint.session.token),
            device,
            None,
        );
    }
    fn handle(&self, mut local: TcpStream) -> Result<(), StateError> {
        // Winsock accepts the connection in the listener's nonblocking mode.
        #[cfg(windows)]
        local.set_nonblocking(false)?;
        local.set_nodelay(true)?;
        local.set_read_timeout(Some(self.timeout))?;
        local.set_write_timeout(Some(self.timeout))?;
        let request = receive_request(&mut local)?;
        let (dest, socks) = match request {
            Request::Connect(d, s) => (d, s),
            Request::Status | Request::Pac => {
                let (ty, body) = if matches!(request, Request::Status) {
                    ("application/json", self.status().to_string())
                } else {
                    (
                        "application/x-ns-proxy-autoconfig",
                        garlic_client_logic::generate_pac(&self.address),
                    )
                };
                return Ok(write!(local,"HTTP/1.1 200 OK\r\nContent-Type: {ty}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len())?);
            }
        };
        if !valid_connect_authority(&dest) {
            return Err("invalid CONNECT destination".into());
        }
        let _active = self.metrics.begin();
        let mut order: Vec<_> = self.endpoints.iter().collect();
        self.health.order(&mut order, &self.failed, self.cooldown);
        if !self.peers.is_empty() {
            let failed=self.failed.lock().unwrap();
            if self.peers.iter().any(|p|failed.get(&p.address).is_none_or(|t|t.elapsed()>=self.cooldown)){
                order.retain(|e|failed.get(&e.url).is_none_or(|t|t.elapsed()>=self.cooldown));
            }
        }
        let mut last_error: StateError = "no authenticated route available".into();
        let mut attempt=0;
        while !order.is_empty() {
            let mut index=0;
            for (id,hook) in &self.hooks {
                let normal=order.iter().position(|e| e.slot.is_none());
                let extra=order.iter().position(|e| e.slot.as_ref()==Some(id));
                match hook.call("route_next",(normal.is_some() as i32,extra.is_some() as i32,attempt)) {
                    Ok(1) if normal.is_some()=>index=normal.unwrap(),
                    Ok(2) if extra.is_some()=>index=extra.unwrap(),
                    _=>{}
                }
            }
            let endpoint=order.remove(index);
            attempt+=1;
            match self.connect(endpoint, &dest, &mut local) {
                Ok((mut exchange, id, device)) => {
                    self.failed.lock().unwrap().remove(&endpoint.url);
                    let result = (|| {
                        if socks {
                            local.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0])?;
                        } else {
                            local.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
                        }
                        exchange.relay(&mut local, &self.stop)
                    })();
                    if let Err(e) = &result {
                        if e.downcast_ref::<tungstenite::Error>().is_some() {
                            self.failed
                                .lock()
                                .unwrap()
                                .insert(endpoint.url.clone(), Instant::now());
                            let _ = local.shutdown(std::net::Shutdown::Both);
                        }
                        self.metrics.failed();
                        self.log_transport_error("relay_error", endpoint, &e.to_string());
                        self.metrics
                            .route_error(&endpoint.url, endpoint.route, &e.to_string());
                    }
                    self.close(endpoint, id, device.as_deref());
                    return result;
                }
                Err(e) => {
                    self.log_transport_error("route_error", endpoint, &e.to_string());
                    self.metrics
                        .route_error(&endpoint.url, endpoint.route, &e.to_string());
                    let retry = garlic_client_logic::retry_allowed(Some(&e.to_string()));
                    last_error = e;
                    if !retry {
                        break;
                    }
                    self.failed
                        .lock()
                        .unwrap()
                        .insert(endpoint.url.clone(), Instant::now());
                }
            }
        }
        // A stream is retried only before acknowledging the local CONNECT request.
        // Authentication/authorization failures are terminal, including on the last Garlic route.
        if self.required_exit.is_none() && garlic_client_logic::retry_allowed(Some(&last_error.to_string())) {
            let mut peers=self.peers.iter().collect::<Vec<_>>();
            {
                let failed=self.failed.lock().unwrap();
                peers.sort_by_key(|p|failed.get(&p.address).is_some_and(|t|t.elapsed()<self.cooldown));
            }
            for peer in peers {
                let started=Instant::now();
                match peer.open(&dest) {
                    Ok(stream)=>{
                        self.failed.lock().unwrap().remove(&peer.address);
                        self.metrics.opened(&peer.address,Route::Direct,&Value::Null,started.elapsed().as_millis() as u64,
                            match peer.kind.as_str(){"http"=>"HTTP","https"=>"HTTPS","socks4"=>"SOCKS4","socks5"=>"SOCKS5",_=>"SSH"},0,&Value::Null);
                        if socks { local.write_all(&[5,0,0,1,0,0,0,0,0,0])?; }
                        else {local.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;}
                        return stream.relay(&local,self);
                    }
                    Err(e)=>{
                        self.failed.lock().unwrap().insert(peer.address.clone(),Instant::now());
                        self.metrics.route_error(&peer.address,Route::Direct,&e.to_string());
                        last_error=e;
                    }
                }
            }
        }
        self.metrics.failed();
        if socks {
            local.write_all(&[5, 1, 0, 1, 0, 0, 0, 0, 0, 0])?;
        } else {
            local.write_all(
                b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )?;
        }
        Err(last_error)
    }
}
pub(crate) fn plugin_store(settings: &Settings) -> Settings {
    Settings { config: settings.config.with_file_name("plugin").join("client.json") }
}
fn route_sessions(settings: &Settings) -> Result<crate::settings::Sessions, StateError> {
    let mut sessions=settings.sessions()?;
    if health::read(&settings.side("installed-gpf")).is_some_and(|p| !p.connections.is_empty()) {
        sessions.entries.extend(plugin_store(settings).sessions()?.entries);
    }
    Ok(sessions)
}
pub(crate) fn slot_path(settings:&Settings,id:&str)->Result<Settings,StateError>{
    if id.is_empty() || id.len()>48 || !id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b==b'-') {
        return Err("invalid plugin id".into());
    }
    Ok(Settings{config:settings.config.with_file_name("plugins-v3").join(format!("s-{id}")).join("client.json")})
}
struct NetworkSlot {
    id:String, guest:release_cache::guest::Guest, endpoints:Vec<Endpoint>, peers:Vec<connector::Peer>
}
fn network_slots(settings:&Settings)->Vec<NetworkSlot>{
    if !cfg!(feature="alpha"){return Vec::new()}
    let Ok(entries)=std::fs::read_dir(settings.config.with_file_name("plugins-v3")) else{return Vec::new()};
    let mut ids=entries.flatten().filter_map(|e|e.file_name().into_string().ok()).collect::<Vec<_>>();
    ids.sort();
    ids.into_iter().filter_map(|id|{
        let id=id.strip_prefix("s-")?.to_owned();
        let slot=slot_path(settings,&id).ok()?;
        let read=|ext:&str|->Option<Value>{
            let mut bytes=Vec::new();std::fs::File::open(slot.side(ext)).ok()?.take(16385).read_to_end(&mut bytes).ok()?;
            if bytes.len()>16384{return None} serde_json::from_slice(&bytes).ok()
        };
        let m=read("gpf-module")?;let d=read("installed-gpf")?;
        if m["id"]!=id || m["format"]!="gpf/3" || m["capability"]!="network" || d["id"]!=id || d["enabled"]!=true{return None}
        let guest=release_cache::guest::Guest::load(m["wasm"].as_str()?).ok()?;
        guest.call("route_next",(1,0,0)).ok()?;
        let keys=plugin_store(&slot).sessions().unwrap_or_default();
        let endpoints=d["connections"].as_array()?.iter().take(24).filter_map(|row|{
            let url=crate::settings::origin(row["address"].as_str()?).ok()?;
            let session=keys.entries.get(&url)?.clone();
            if session.expires_at<=time(){return None}
            let route=match row["kind"].as_str()?{"direct"=>Route::Direct,"ingress"=>Route::ThroughRu,_=>return None};
            Some(Endpoint{url,route,session,slot:Some(id.clone())})
        }).collect();
        let secrets=read("connector-login").unwrap_or(Value::Null);
        let peers=d["connections"].as_array()?.iter().take(24).filter_map(|row|{
            let kind=row["kind"].as_str()?;let address=row["address"].as_str()?;
            let code=connector::code(kind)?;
            if !m["transports"].as_array()?.iter().any(|v|v.as_str()==Some(kind)){return None}
            if guest.call("route_transport",(code,0,0)).ok()!=Some(code){return None}
            let login:connector::Login=serde_json::from_value(secrets[address].clone()).unwrap_or_default();
            #[cfg(windows)]
            let login=connector::Login{password:crate::settings::windows::decode(&login.password).ok()?,..login};
            connector::Peer::new(&id,kind,address,login).ok()
        }).collect();
        Some(NetworkSlot{id,guest,endpoints,peers})
    }).take(16).collect()
}
pub(crate) fn test_peer(settings:&Settings,id:&str,kind:&str,address:&str)->Result<(),StateError>{
    let slot=network_slots(settings).into_iter().find(|s|s.id==id).ok_or("enable the network plugin first")?;
    slot.peers.into_iter().find(|p|p.kind==kind && p.address==address).ok_or("save the connection first")?.test()
}
#[cfg(windows)]
pub(crate) fn peer_bypass(settings:&Settings)->(bool,std::collections::BTreeSet<std::net::Ipv4Addr>){
    use std::net::ToSocketAddrs;
    let peers:Vec<_>=network_slots(settings).into_iter().flat_map(|s|s.peers).collect();
    (!peers.is_empty(),peers.into_iter().filter_map(|p|(p.host.as_str(),p.port).to_socket_addrs().ok())
        .flatten().filter_map(|a|match a.ip(){std::net::IpAddr::V4(ip) if !ip.is_loopback()=>Some(ip),_=>None}).collect())
}
pub(crate) fn apply_connections(settings: &Settings, cfg: &mut Config) -> Result<(), StateError> {
    apply_legacy_connections(settings,cfg)?;
    for slot in network_slots(settings) {
        for ep in slot.endpoints {
            let urls=if ep.route==Route::Direct {&mut cfg.storage_urls} else {&mut cfg.control_urls};
            if !urls.contains(&ep.url){urls.push(ep.url);}
        }
    }
    Ok(())
}
fn apply_legacy_connections(settings:&Settings,cfg:&mut Config)->Result<(),StateError>{
    let Some(data) = health::read(&settings.side("installed-gpf")) else { return Ok(()) };
    if data.connections.is_empty() { return Ok(()) }
    let sessions = route_sessions(settings)?;
    let mut direct = Vec::new();
    let mut ingress = Vec::new();
    for row in data.connections {
        let url = crate::settings::origin(&row.address)?;
        let routes = match row.kind.as_str() {
            "direct" => &mut direct, "ingress" => &mut ingress,
            _ => return Err("unsupported plugin connection type".into()),
        };
        if sessions.entries.get(&url).is_some_and(|s| s.expires_at > time()) && !routes.contains(&url) {
            routes.push(url);
        }
    }
    if direct.is_empty() && ingress.is_empty() {
        return Err("Plugin: sign in to a configured Garlic server first".into());
    }
    cfg.storage_urls = direct;
    cfg.control_urls = ingress;
    Ok(())
}
pub fn run(
    settings: &Settings,
    cfg: &Config,
    api: Api,
    address: SocketAddr,
) -> Result<(), StateError> {
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Release))?;
    let mut cfg = cfg.clone();
    apply_connections(settings, &mut cfg)?;
    serve(settings, &cfg, api, address, stop, None, None)
}

pub fn serve(
    settings: &Settings,
    cfg: &Config,
    api: Api,
    address: SocketAddr,
    stop: Arc<AtomicBool>,
    ready: Option<std::sync::mpsc::SyncSender<SocketAddr>>,
    tunnel_ready: Option<Arc<AtomicBool>>,
) -> Result<(), StateError> {
    crate::journal::write(&settings.config, "proxy_start", match cfg.mode.as_str() { "auto" => "auto", "direct" => "direct", "ru" => "ingress", _ => "unknown" });
    let result = serve_inner(settings, cfg, api, address, stop, ready, tunnel_ready);
    match &result {
        Err(e) => crate::journal::write(&settings.config, "proxy_error", crate::journal::reason(&e.to_string())),
        Ok(()) => crate::journal::write(&settings.config, "proxy_stop", ""),
    }
    result
}
fn serve_inner(
    settings: &Settings,
    cfg: &Config,
    api: Api,
    address: SocketAddr,
    stop: Arc<AtomicBool>,
    ready: Option<std::sync::mpsc::SyncSender<SocketAddr>>,
    tunnel_ready: Option<Arc<AtomicBool>>,
) -> Result<(), StateError> {
    if !address.ip().is_loopback() {
        return Err("this client proxy accepts a loopback bind address".into());
    }
    let required_exit = std::env::var("GARLIC_REQUIRED_RELAY_EXIT")
        .ok()
        .filter(|value| !value.is_empty());
    if required_exit.as_ref().is_some_and(|value| {
        !URL_SAFE_NO_PAD
            .decode(value)
            .is_ok_and(|decoded| decoded.len() == 32)
    }) {
        return Err("invalid relay exit ID".into());
    }
    let sessions = route_sessions(settings)?;
    let mut endpoints = vec![];
    for (kind, urls) in [
        (Route::Direct, &cfg.storage_urls),
        (Route::ThroughRu, &cfg.control_urls),
    ] {
        if (cfg.mode == "ru" && kind == Route::Direct)
            || (cfg.mode == "direct" && kind == Route::ThroughRu)
        {
            continue;
        }
        for url in urls {
            if let Some(session) = sessions.entries.get(url).filter(|s| s.expires_at > time()) {
                endpoints.push(Endpoint {
                    slot: None,
                    url: url.clone(),
                    route: kind,
                    session: session.clone(),
                });
            }
        }
    }
    let mut peers=Vec::new();
    let mut hooks=Vec::new();
    for slot in network_slots(settings) {
        if required_exit.is_none(){peers.extend(slot.peers);}
        hooks.push((slot.id,slot.guest));
        endpoints.extend(slot.endpoints.into_iter().filter(|e|{
            let allowed=if e.route==Route::Direct {cfg.mode!="ru" && cfg.storage_urls.contains(&e.url)}
                else {cfg.mode!="direct" && cfg.control_urls.contains(&e.url)};
            allowed
        }));
    }
    if endpoints.is_empty() && peers.is_empty() {
        return Err("no valid user session for the configured routes".into());
    }
    let listener = TcpListener::bind(address)?;
    listener.set_nonblocking(true)?;
    let devices = Devices::new(api.clone(), settings)?;
    let proxy = Arc::new(Proxy {
        settings: settings.clone(),
        api,
        devices,
        health: health::RouteHealth::new(cfg.mode == "auto" && endpoints.len() > 1,
            &settings.side("installed-gpf")),
        endpoints,
        peers,
        hooks,
        required_exit,
        failed: Default::default(),
        timeout: Duration::from_millis(cfg.request_timeout_ms.clamp(1000, 120000)),
        lifetime: Duration::from_secs(24 * 3600),
        cooldown: Duration::from_millis(cfg.cooldown_ms),
        stop,
        tunnel_ready,
        active: 0.into(),
        address: listener.local_addr()?,
        metrics: Metrics::default(),
        traffic: Arc::default(),
    });
    let generation = settings.stop_generation()?;
    crate::journal::write(&settings.config, "proxy_ready", "");
    health::start(&proxy);
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let renew = proxy.clone();
    let manager = thread::spawn(move || {
        while rx.recv_timeout(Duration::from_secs(10)).is_err()
            && !renew.stop.load(Ordering::Acquire)
        {
            crate::journal::write(&renew.settings.config, "traffic", &format!("rx_bytes={} tx_bytes={} active={}",
                renew.traffic.rx.load(Ordering::Relaxed), renew.traffic.tx.load(Ordering::Relaxed), renew.active.load(Ordering::Relaxed)));
            renew.devices.maintain();
        }
    });
    println!("Garlic CONNECT/SOCKS5 proxy listening on {}", proxy.address);
    if let Some(ready) = ready {
        ready
            .send(proxy.address)
            .map_err(|_| "proxy startup cancelled")?;
    }
    let mut last_control_poll = Instant::now();
    while !proxy.stop.load(Ordering::Acquire) {
        if last_control_poll.elapsed() >= Duration::from_millis(250) {
            last_control_poll = Instant::now();
            if settings.stop_generation()? != generation {
                proxy.stop.store(true, Ordering::Release);
                break;
            }
        }
        match listener.accept() {
            Ok((local, _)) => {
                if proxy.active.load(Ordering::Relaxed) >= 512 {
                    drop(local);
                    continue;
                }
                proxy.active.fetch_add(1, Ordering::Relaxed);
                let p = proxy.clone();
                thread::spawn(move || {
                    if let Err(e) = p.handle(local) {
                        eprintln!("Proxy stream ended: {e}");
                    }
                    p.active.fetch_sub(1, Ordering::Relaxed);
                });
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(15))
            }
            Err(e) => return Err(e.into()),
        }
    }
    let _ = tx.send(());
    let _ = manager.join();
    proxy.devices.release();
    Ok(())
}
