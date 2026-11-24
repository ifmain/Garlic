use super::{Endpoint, Proxy};
use garlic_relay_data::control::Route;
use std::{
    collections::BTreeMap,
    sync::{Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

pub(super) struct RouteHealth {
    enabled: bool,
    samples: Mutex<BTreeMap<String, (Instant, Option<u64>)>>,
    changed: Condvar,
}

impl RouteHealth {
    pub(super) fn new(enabled: bool) -> Self {
        Self {
            enabled,
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
                        match s.get(&endpoint.url).filter(|(t, _)| t.elapsed() < Duration::from_secs(45)) {
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
                cost + if e.route == Route::ThroughRu { 40 } else { 0 },
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
                    .map(|load| start.elapsed().as_millis() as u64 + load * 2);
                p.health
                    .samples
                    .lock()
                    .unwrap()
                    .insert(e.url.clone(), (Instant::now(), cost));
                p.health.changed.notify_all();
            }
            thread::sleep(Duration::from_secs(15));
        });
    }
}
