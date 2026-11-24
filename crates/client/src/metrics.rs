use garlic_relay_data::control::Route;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct Traffic {
    pub rx: AtomicU64,
    pub tx: AtomicU64,
}

#[derive(Default)]
struct Recent {
    last: Value,
    transport: BTreeMap<String, Value>,
    last_failure: Value,
    errors: BTreeMap<String, u64>,
    changes: u64,
    ping: Option<u64>,
    modes: BTreeMap<&'static str, u64>,
}

#[derive(Default)]
pub struct Metrics {
    active: AtomicU64,
    total: AtomicU64,
    failed: AtomicU64,
    recent: Mutex<Recent>,
}

pub struct Active<'a>(&'a AtomicU64);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}
impl Metrics {
    pub fn begin(&self) -> Active<'_> {
        self.total.fetch_add(1, Ordering::Relaxed);
        self.active.fetch_add(1, Ordering::Relaxed);
        Active(&self.active)
    }
    pub fn failed(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
    }
    pub fn route_error(&self, origin: &str, via: Route, message: &str) {
        let mut recent = self.recent.lock().unwrap();
        let reason = message.chars().take(200).collect::<String>();
        let group = if reason.starts_with("HTTP ") {
            reason.split(':').next().unwrap_or("HTTP")
        } else {
            "transport"
        };
        *recent.errors.entry(group.to_owned()).or_default() += 1;
        recent.last_failure = json!({"origin":origin,"via":via,"error":reason});
    }
    pub fn opened(
        &self,
        origin: &str,
        via: Route,
        version: &Value,
        ping: u64,
        protocol: &'static str,
        record_limit: usize,
        relay_protocol: &Value,
    ) {
        let mut r = self.recent.lock().unwrap();
        if !r.last.is_null() && (r.last["origin"] != origin || r.last["via"] != json!(via)) {
            r.changes += 1;
        }
        r.last = json!({"origin":origin,"via":via});
        r.ping = Some(ping);
        r.transport.insert(origin.to_owned(),json!({
            "confirmed":true, "protocol":protocol, "peer_version":version, "record_bytes":record_limit,
            "confirmed_at_unix_ms":SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
            "segment":if via==Route::Direct {"client_to_exit"} else {"client_to_entry"},
            "fallback_reason":null,"relay_protocol":relay_protocol,"shape_profile":null
        }));
        *r.modes.entry(protocol).or_default() += 1;
    }
    pub fn status<'a>(
        &self,
        traffic: &Traffic,
        bind: std::net::SocketAddr,
        endpoints: impl Iterator<Item = (&'a str, Route)>,
        failed: &BTreeMap<String, Instant>,
        cooldown: std::time::Duration,
    ) -> Value {
        let r = self.recent.lock().unwrap();
        let routes:Vec<_>=endpoints.map(|(origin,via)| json!({"origin":origin,"via":via,
            "cooling":failed.get(origin).is_some_and(|t|t.elapsed()<cooldown),"transport":r.transport.get(origin)})).collect();
        let ready = routes.iter().any(|v| v["cooling"] == false);
        json!({"version":env!("CARGO_PKG_VERSION"),"service":"garlic-internal-network",
            "network_ready":ready,"vpn_initializing":false,"tunnel":false,
            "proxy_bind":bind.to_string(),"pac_url":format!("http://{bind}/proxy.pac"),
            "active_streams":self.active.load(Ordering::Relaxed),"total_streams":self.total.load(Ordering::Relaxed),
            "failed_streams":self.failed.load(Ordering::Relaxed),"tx_bytes":traffic.tx.load(Ordering::Relaxed),
            "rx_bytes":traffic.rx.load(Ordering::Relaxed),"routes":routes,"last_route":r.last,
            "last_route_failure":r.last_failure,"route_changes":r.changes,"error_counts":r.errors,
            "ping_ms":r.ping,"transport":r.last["origin"].as_str().and_then(|origin|r.transport.get(origin)),
            "transport_modes":{ "GEX1":r.modes.get("GEX1").copied().unwrap_or(0),
                "GEX2":r.modes.get("GEX2").copied().unwrap_or(0), "GEX3":r.modes.get("GEX3").copied().unwrap_or(0),
                "GEX4":r.modes.get("GEX4").copied().unwrap_or(0) }})
    }
}
