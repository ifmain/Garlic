use super::{Endpoint, Proxy};
use garlic_relay_data::control::Route;
use std::{
    collections::BTreeMap,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

pub(super) struct RouteHealth {
    pub(super) enabled: bool,
    policy: ProbePolicy,
    samples: Mutex<BTreeMap<String, (Instant, Option<u64>)>>,
    changed: Condvar,
}

struct ProbePolicy {
    guest: Option<release_cache::guest::Guest>,
    interval_seconds: u64,
    ingress_penalty_ms: u64,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Connection {
    pub(super) kind: String,
    pub(super) address: String,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Data {
    format: String,
    id: String,
    enabled: bool,
    interval_seconds: u64,
    ingress_penalty_ms: u64,
    #[serde(default)]
    pub(super) connections: Vec<Connection>,
}

fn load_guest(path: &std::path::Path, id: &str) -> Option<release_cache::guest::Guest> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path.with_extension("gpf-module"))
        .ok()?
        .take(16385)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 16384 {
        return None;
    }
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    if manifest["id"] != id || manifest["format"] != "gpf/2" {
        return None;
    }
    let guest = release_cache::guest::Guest::load(manifest["wasm"].as_str()?).ok()?;
    guest.call("route_score", (0, 0, 0)).ok()?;
    Some(guest)
}
pub(super) fn read(path: &std::path::Path) -> Option<Data> {
    use std::io::Read;

    if !cfg!(feature = "alpha") {
        return None;
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(8193)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 8192 {
        return None;
    }
    let v: Data = serde_json::from_slice(&bytes).ok()?;
    (v.enabled
        && v.format == "garlic-plugin-data/1"
        && load_guest(path, &v.id).is_some()
        && v.connections.len() <= 24
        && (3..=60).contains(&v.interval_seconds)
        && v.ingress_penalty_ms <= 250)
        .then_some(v)
}
impl ProbePolicy {
    fn load(path: &std::path::Path) -> Self {
        let data = read(path);
        if let Some(v) = data {
            if let Some(guest) = load_guest(path, &v.id) {
                return Self {
                    guest: Some(guest),
                    interval_seconds: v.interval_seconds,
                    ingress_penalty_ms: v.ingress_penalty_ms,
                };
            }
        }
        Self {
            guest: None,
            interval_seconds: 15,
            ingress_penalty_ms: 40,
        }
    }
}

impl RouteHealth {
    pub(super) fn new(enabled: bool, data: &std::path::Path) -> Self {
        Self {
            enabled,
            policy: ProbePolicy::load(data),
            samples: Mutex::default(),
            changed: Condvar::new(),
        }
    }

    pub(super) fn order(
        &self,
        endpoints: &mut [&Endpoint],
        failures: &Mutex<BTreeMap<String, Instant>>,
        cooldown: Duration,
    ) {
        let samples = self.samples.lock().unwrap();
        let samples = if self.enabled {
            self.changed
                .wait_timeout_while(samples, Duration::from_millis(450), |s| {
                    let mut pending = false;
                    for endpoint in endpoints.iter() {
                        match s
                            .get(&endpoint.url)
                            .filter(|(t, _)| t.elapsed() < Duration::from_secs(45))
                        {
                            Some((_, Some(_))) => return false,
                            Some((_, None)) => (),
                            None => pending = true,
                        }
                    }
                    pending
                })
                .unwrap()
                .0
        } else {
            samples
        };
        let failed = failures.lock().unwrap();
        endpoints.sort_by_key(|e| {
            let failed_recently = failed.get(&e.url).is_some_and(|t| t.elapsed() < cooldown);
            let sample = samples
                .get(&e.url)
                .filter(|(t, _)| t.elapsed() < Duration::from_secs(45));
            let (rank, cost) = match sample {
                Some((_, Some(cost))) => (0, *cost),
                None => (1, 0),
                Some((_, None)) => (2, 0),
            };
            (
                failed_recently,
                rank,
                cost + if self.policy.guest.is_none() && e.route == Route::ThroughRu {
                    40
                } else {
                    0
                },
            )
        });
    }
}

pub(super) fn start(proxy: &Arc<Proxy>) {
    if !proxy.health.enabled {
        return;
    }
    // Probe the first direct and ingress candidates concurrently even with a long exit list.
    let mut direct = proxy
        .endpoints
        .iter()
        .enumerate()
        .filter(|(_, e)| e.route == Route::Direct);
    let mut ingress = proxy
        .endpoints
        .iter()
        .enumerate()
        .filter(|(_, e)| e.route == Route::ThroughRu);
    let mut indices = Vec::new();
    loop {
        let pair = [direct.next(), ingress.next()];
        if pair.iter().all(Option::is_none) {
            break;
        }
        indices.extend(pair.into_iter().flatten().map(|(i, _)| i));
    }
    let interval = Duration::from_secs(proxy.health.policy.interval_seconds);
    let count = indices.len().min(4);
    for lane in 0..count {
        let group: Vec<_> = indices.iter().skip(lane).step_by(count).copied().collect();
        let owner = Arc::downgrade(proxy);
        thread::spawn(move || loop {
            for index in &group {
                let Some(p) = owner.upgrade() else { return };
                if p.stop.load(std::sync::atomic::Ordering::Acquire) {
                    return;
                }
                let e = &p.endpoints[*index];
                let start = Instant::now();
                let cost = p
                    .api
                    .route_probe(&e.url, &e.session.token)
                    .ok()
                    .map(|load| {
                        let latency = start.elapsed().as_millis().min(i32::MAX as u128) as i32;
                        let ingress = i32::from(e.route == Route::ThroughRu);
                        let default = latency as u64
                            + load * 2
                            + if p.health.policy.guest.is_some() {
                                ingress as u64 * p.health.policy.ingress_penalty_ms
                            } else {
                                0
                            };
                        p.health
                            .policy
                            .guest
                            .as_ref()
                            .and_then(|g| {
                                g.call("route_score", (latency, load as i32, ingress)).ok()
                            })
                            .and_then(|n| u64::try_from(n).ok())
                            .unwrap_or(default)
                    });
                p.health
                    .samples
                    .lock()
                    .unwrap()
                    .insert(e.url.clone(), (Instant::now(), cost));
                p.health.changed.notify_all();
            }
            thread::sleep(interval);
        });
    }
}

#[cfg(all(test,feature="alpha"))]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD,Engine as _};

    #[test]
    fn external_guest_controls_route_score() {
        let dir=std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir(&dir).unwrap();
        let path=dir.join("client.installed-gpf");
        std::fs::write(&path,br#"{"enabled":true,"id":"fixture","format":"garlic-plugin-data/1","interval_seconds":5,"ingress_penalty_ms":0}"#).unwrap();
        assert!(ProbePolicy::load(&path).guest.is_none());
        // Export route_score(i32,i32,i32)->i32 and select a different input each time.
        for arg in [0u8,1] {
            let mut wasm=b"\0asm\x01\0\0\0\x01\x08\x01\x60\x03\x7f\x7f\x7f\x01\x7f\x03\x02\x01\0\x07\x0f\x01\x0broute_score\0\0\x0a\x06\x01\x04\0\x20".to_vec();
            wasm.extend([arg,11]);
            let manifest=serde_json::json!({"id":"fixture","format":"gpf/2","wasm":STANDARD.encode(wasm)});
            std::fs::write(path.with_extension("gpf-module"),manifest.to_string()).unwrap();
            let guest=ProbePolicy::load(&path).guest.unwrap();
            let a=guest.call("route_score",(10,90,0)).unwrap();
            let b=guest.call("route_score",(70,20,1)).unwrap();
            assert_eq!(a<b,arg==0);
        }
        std::fs::write(path.with_extension("gpf-module"),b"corrupt").unwrap();
        assert!(ProbePolicy::load(&path).guest.is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
