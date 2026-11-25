use garlic_relay_data::control::{ErrorCode, Hello, Message, ProxyError, StreamOpen};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;

#[derive(Clone)]
pub enum Outbound {
    Control(Message),
    Data(Uuid, Vec<u8>),
}
type WorkerTx = mpsc::Sender<Outbound>;

struct Worker {
    hello: Hello,
    label: String,
    tx: WorkerTx,
    active: usize,
    draining: bool,
}
struct Flow {
    open: StreamOpen,
    exit_node: String,
    worker: Option<Uuid>,
    detached: Option<Instant>,
    awaiting_client: Option<Instant>,
    client: broadcast::Sender<Arc<[u8]>>,
}
struct Pending {
    worker: Uuid,
    reply: oneshot::Sender<Result<(), ProxyError>>,
}
#[derive(Default)]
struct State {
    workers: HashMap<Uuid, Worker>,
    flows: HashMap<Uuid, Flow>,
    pending: HashMap<Uuid, Pending>,
}
#[derive(Default)]
pub struct Hub(Mutex<State>, tokio::sync::Notify);

impl State {
    fn select(&self, node: Option<&str>, except: Option<Uuid>) -> Option<Uuid> {
        self.workers
            .iter()
            .filter(|(id, w)| {
                Some(**id) != except
                    && !w.draining
                    && w.active < (w.hello.max_streams as usize)
                    && node.is_none_or(|n| w.hello.node_id == n || w.label == n)
            })
            .min_by_key(|(id, w)| (w.active, **id))
            .map(|(id, _)| *id)
    }

    // Account across all authenticated worker connections of a logical exit.
    // A reconnect or a second device does not grant another allocation.
    fn select_for_open(&self, node: Option<&str>, user: Uuid) -> Result<Uuid, ErrorCode> {
        #[derive(Default)]
        struct Usage {
            capacity: usize,
            users: HashMap<Uuid, usize>,
        }
        let mut exits: HashMap<&str, Usage> = HashMap::new();
        for w in self
            .workers
            .values()
            .filter(|w| !w.draining && node.is_none_or(|n| w.hello.node_id == n || w.label == n))
        {
            exits.entry(&w.hello.node_id).or_default().capacity += w.hello.max_streams as usize;
        }
        if exits.is_empty() {
            return Err(ErrorCode::RouteUnavailable);
        }
        for flow in self.flows.values() {
            if let Some(exit) = exits.get_mut(flow.exit_node.as_str()) {
                *exit.users.entry(flow.open.user_id).or_default() += 1;
            }
        }
        self.workers
            .iter()
            .filter(|(_, w)| {
                if w.draining
                    || w.active >= w.hello.max_streams as usize
                    || w.tx.capacity() == 0
                    || !node.is_none_or(|n| w.hello.node_id == n || w.label == n)
                {
                    return false;
                }
                let usage = &exits[w.hello.node_id.as_str()];
                let held = usage.users.get(&user).copied().unwrap_or(0);
                let participants = usage.users.len() + usize::from(held == 0);
                held < usage.capacity.div_ceil(participants.max(2))
            })
            .min_by_key(|(id, w)| {
                (
                    // Rendezvous hashing keeps an account on the same eligible exit
                    // across streams. Load balances workers within that exit.
                    std::cmp::Reverse({
                        use sha2::{Digest, Sha256};
                        Sha256::new().chain_update(user.as_bytes())
                            .chain_update(w.hello.node_id.as_bytes()).finalize()
                    }),
                    (w.active as u64 + 1) * 1_048_576 / w.hello.max_streams,
                    **id,
                )
            })
            .map(|(id, _)| *id)
            .ok_or(ErrorCode::CapacityExhausted)
    }

    fn discard(&mut self, id: Uuid) {
        if let Some(flow) = self.flows.remove(&id) {
            if let Some(w) = flow.worker.and_then(|worker| self.workers.get_mut(&worker)) {
                w.active = w.active.saturating_sub(1);
                let _ =
                    w.tx.try_send(Outbound::Control(Message::CloseFlow { flow_id: id }));
            }
        }
    }

    fn handoff(&mut self, old: Option<Uuid>) {
        let candidates: Vec<_> = self
            .flows
            .iter()
            .filter(|(_, f)| f.worker == old)
            .map(|(id, f)| (*id, f.exit_node.clone()))
            .collect();
        for (id, node) in candidates {
            if let Some(worker) = old.and_then(|w| self.workers.get_mut(&w)) {
                worker.active = worker.active.saturating_sub(1);
            }
            let replacement = self.select(Some(&node), old);
            let attached = replacement.filter(|wid| {
                let w = self.workers.get_mut(wid).unwrap();
                if w.tx
                    .try_send(Outbound::Control(Message::AdoptFlow { flow_id: id }))
                    .is_ok()
                {
                    w.active += 1;
                    true
                } else {
                    false
                }
            });
            let f = self.flows.get_mut(&id).unwrap();
            f.worker = attached;
            if attached.is_some() {
                f.detached = None;
            } else {
                f.detached.get_or_insert_with(Instant::now);
            }
        }
    }
}

impl Hub {
    pub fn load_percent(&self) -> u64 {
        let state = self.0.lock().unwrap();
        let (used, capacity) = state.workers.values().filter(|w| !w.draining)
            .fold((0u64, 0u64), |(used, capacity), w| (used + w.active as u64, capacity + w.hello.max_streams));
        if capacity == 0 { 100 } else { (used * 100 / capacity).min(100) }
    }

    pub fn active_streams(&self) -> usize {
        self.0.lock().unwrap().flows.len()
    }
    pub fn register(&self, hello: Hello, label: String, tx: WorkerTx) -> bool {
        let mut s = self.0.lock().unwrap();
        if s.workers.contains_key(&hello.worker_id) {
            return false;
        }
        s.workers.insert(
            hello.worker_id,
            Worker {
                hello,
                label,
                tx,
                active: 0,
                draining: false,
            },
        );
        s.handoff(None);
        self.1.notify_waiters();
        true
    }
    pub fn remove_worker(&self, id: Uuid) {
        let mut s = self.0.lock().unwrap();
        s.workers.remove(&id);
        s.pending.retain(|_, p| p.worker != id);
        s.handoff(Some(id));
        self.1.notify_waiters();
    }
    pub fn drain(&self, id: Uuid) {
        let mut s = self.0.lock().unwrap();
        if let Some(w) = s.workers.get_mut(&id) {
            w.draining = true;
        }
        s.handoff(Some(id));
        self.1.notify_waiters();
    }
    pub fn cleanup(&self, now_ms: u64) {
        let mut s = self.0.lock().unwrap();
        let expired: Vec<_> = s
            .flows
            .iter()
            .filter(|(_, f)| {
                f.open.deadline_unix_ms <= now_ms
                    || f.awaiting_client.is_some_and(|t| t.elapsed() > Duration::from_secs(60))
                    || f.detached
                        .is_some_and(|t| t.elapsed() > Duration::from_secs(60))
            })
            .map(|(id, _)| *id)
            .collect();
        for id in expired {
            s.discard(id);
            self.1.notify_waiters();
        }
    }
    pub fn opened(&self, worker: Uuid, id: Uuid, error: Option<ProxyError>) {
        let mut s = self.0.lock().unwrap();
        if s.pending.get(&id).is_some_and(|p| p.worker == worker) {
            if let Some(p) = s.pending.remove(&id) {
                let _ = p.reply.send(error.map_or(Ok(()), Err));
            }
        }
    }
    pub async fn open(&self, open: StreamOpen, admin: bool) -> Result<(), ProxyError> {
        let request_id = Uuid::new_v4();
        let id = open.flow_id;
        let rx = {
            let mut s = self.0.lock().unwrap();
            if s.flows.contains_key(&id)
                || s.flows.values().any(|f| f.open.stream_id == open.stream_id)
            {
                return Err(ErrorCode::InvalidRequest.into());
            }
            let worker = s.select_for_open(open.preferred_exit.as_deref(), open.user_id)?;
            let w = s.workers.get_mut(&worker).unwrap();
            w.tx.try_send(Outbound::Control(Message::Open {
                request_id,
                open: open.clone(),
                admin,
            }))
            .map_err(|_| ErrorCode::CapacityExhausted)?;
            w.active += 1;
            let node = w.hello.node_id.clone();
            let (reply, rx) = oneshot::channel();
            s.pending.insert(request_id, Pending { worker, reply });
            let (client, _) = broadcast::channel(128);
            s.flows.insert(
                id,
                Flow {
                    open,
                    exit_node: node,
                    worker: Some(worker),
                    client,
                    detached: None,
                    awaiting_client: Some(Instant::now()),
                },
            );
            rx
        };
        // A canceled HTTP request must also release its reservation.
        struct Reservation<'a> {
            hub: &'a Hub,
            request: Uuid,
            flow: Option<Uuid>,
        }
        impl Drop for Reservation<'_> {
            fn drop(&mut self) {
                let mut s = self.hub.0.lock().unwrap();
                s.pending.remove(&self.request);
                if let Some(id) = self.flow {
                    s.discard(id);
                }
            }
        }
        let mut reservation = Reservation {
            hub: self,
            request: request_id,
            flow: Some(id),
        };
        tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|_| ErrorCode::RouteUnavailable)?
            .map_err(|_| ErrorCode::RouteUnavailable)??;
        reservation.flow = None;
        Ok(())
    }
    pub fn subscribe(
        &self,
        id: Uuid,
        user: Uuid,
    ) -> Result<(Uuid, broadcast::Receiver<Arc<[u8]>>), ErrorCode> {
        let mut s = self.0.lock().unwrap();
        let flow_id = if s.flows.contains_key(&id) {
            id
        } else {
            s.flows.values().find(|f| f.open.stream_id == id)
                .map(|f| f.open.flow_id).ok_or(ErrorCode::RouteUnavailable)?
        };
        let f = s.flows.get_mut(&flow_id).ok_or(ErrorCode::RouteUnavailable)?;
        if f.open.user_id != user {
            return Err(ErrorCode::Forbidden);
        }
        f.awaiting_client = None;
        Ok((f.open.flow_id, f.client.subscribe()))
    }
    pub async fn upstream(&self, id: Uuid, bytes: Vec<u8>) -> Result<(), ErrorCode> {
        let handoff = async {
            loop {
                let changed = self.1.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                let candidate = {
                    let state = self.0.lock().unwrap();
                    let flow = state.flows.get(&id).ok_or(ErrorCode::RouteUnavailable)?;
                    flow.worker
                        .and_then(|wid| state.workers.get(&wid).map(|w| (wid, w.tx.clone())))
                };
                let Some((wid, tx)) = candidate else {
                    changed.await;
                    continue;
                };
                let slot = tokio::select! {
                    _=changed.as_mut()=>continue,
                    result=tx.reserve_owned()=>match result {
                        Ok(slot)=>slot,
                        Err(_)=>{changed.await;continue;}
                    }
                };
                let state = self.0.lock().unwrap();
                if state.flows.get(&id).is_some_and(|f| f.worker == Some(wid)) {
                    slot.send(Outbound::Data(id, bytes));
                    return Ok(());
                }
            }
        };
        tokio::time::timeout(Duration::from_secs(60), handoff)
            .await
            .map_err(|_| ErrorCode::RouteUnavailable)?
    }
    pub fn downstream(&self, worker: Uuid, id: Uuid, bytes: Vec<u8>) {
        let s = self.0.lock().unwrap();
        if let Some(f) = s.flows.get(&id).filter(|f| f.worker == Some(worker)) {
            let _ = f.client.send(Arc::from(bytes));
        }
    }
    pub fn close(&self, id: Uuid, user: Uuid) -> Result<(), ErrorCode> {
        let mut s = self.0.lock().unwrap();
        let id = if s.flows.contains_key(&id) {
            id
        } else {
            s.flows
                .iter()
                .find_map(|(flow, f)| (f.open.stream_id == id).then_some(*flow))
                .unwrap_or(id)
        };
        if s.flows.get(&id).is_some_and(|f| f.open.user_id != user) {
            return Err(ErrorCode::Forbidden);
        }
        s.discard(id);
        self.1.notify_waiters();
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use garlic_relay_data::control::{Authority, Route};

    fn test_open(user: Uuid) -> StreamOpen {
        StreamOpen {
            control_version: 1,
            data_version: 1,
            node_id: "client".into(),
            channel_id: Uuid::new_v4(),
            user_id: user,
            flow_id: Uuid::new_v4(),
            stream_id: Uuid::new_v4(),
            connect_authority: "example.org:443".into(),
            preferred_exit: None,
            route: Route::Direct,
            capabilities: vec![],
            window_size: 65536,
            deadline_unix_ms: u64::MAX,
            authority: Authority {
                owner_id: Uuid::new_v4(),
                epoch: 1,
            },
        }
    }

    #[test]
    fn cleanup_releases_only_unclaimed_reservations() {
        let hub = Hub::default();
        let user = Uuid::new_v4();
        let new_flow = || Flow {
            open: test_open(user),
            exit_node: "exit".into(),
            worker: None,
            client: broadcast::channel(4).0,
            detached: None,
            awaiting_client: Some(Instant::now() - Duration::from_secs(61)),
        };
        let stale = new_flow();
        let active = new_flow();
        let stale_id = stale.open.flow_id;
        let active_id = active.open.flow_id;
        let stream_id = active.open.stream_id;
        {
            let mut state = hub.0.lock().unwrap();
            state.flows.insert(stale_id, stale);
            state.flows.insert(active_id, active);
        }
        assert!(matches!(hub.subscribe(stale_id, Uuid::new_v4()), Err(ErrorCode::Forbidden)));
        let (_id, _receiver) = hub.subscribe(stream_id, user).unwrap();
        hub.cleanup(0);
        let state = hub.0.lock().unwrap();
        assert!(!state.flows.contains_key(&stale_id));
        assert!(state.flows.contains_key(&active_id));
    }

    #[test]
    fn changing_worker_load_preserves_eligible_exit() {
        let hub = Hub::default();
        let user = Uuid::new_v4();
        let mut receivers = vec![];
        for name in ["first", "second"] {
            let (tx, rx) = mpsc::channel(8);
            receivers.push(rx);
            assert!(hub.register(Hello {
                protocol_version: 1, worker_id: Uuid::new_v4(), node_id: name.into(),
                capabilities: vec![], max_streams: 32,
            }, name.into(), tx));
        }
        let mut state = hub.0.lock().unwrap();
        let chosen = state.select_for_open(None, user).unwrap();
        state.workers.get_mut(&chosen).unwrap().active = 12;
        assert_eq!(state.select_for_open(None, user), Ok(chosen));
        state.workers.get_mut(&chosen).unwrap().draining = true;
        assert_ne!(state.select_for_open(None, user).unwrap(), chosen);
    }

    #[tokio::test]
    async fn pending_flow_reserves_a_user_share_until_cancelled() {
        let hub = Hub::default();
        let worker_id = Uuid::new_v4();
        let (tx, mut worker) = mpsc::channel(8);
        assert!(hub.register(
            Hello {
                protocol_version: 1,
                worker_id,
                node_id: "exit".into(),
                capabilities: vec![],
                max_streams: 2
            },
            "test".into(),
            tx
        ));
        let user = Uuid::new_v4();
        let open = test_open(user);
        {
            let mut pending = Box::pin(hub.open(open, false));
            tokio::select! {
                _=worker.recv()=>(),
                _=&mut pending=>panic!("unexpected completion before worker acknowledgement"),
            }
            let state = hub.0.lock().unwrap();
            assert_eq!(
                state.select_for_open(None, user),
                Err(ErrorCode::CapacityExhausted)
            );
            assert_eq!(state.select_for_open(None, Uuid::new_v4()), Ok(worker_id));
        }
        assert_eq!(
            hub.0.lock().unwrap().select_for_open(None, user),
            Ok(worker_id)
        );
    }
}
