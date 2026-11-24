use super::model::*;
use crate::files;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::{Path, PathBuf},
    sync::Mutex,
};
use uuid::Uuid;

pub const LEASE_MS: u64 = 30_000;
const CHANNEL_MS: u64 = 60_000;
const QUOTA: usize = 1024;
pub struct Jobs {
    state: Mutex<State>,
    path: PathBuf,
}
pub struct State {
    pub disk: Disk,
    pub channels: BTreeMap<Uuid, Channel>,
    reservations: BTreeMap<Uuid, Reservation>,
    history: VecDeque<Value>,
    seq: u64,
    opened: u64,
    closed: u64,
    finished: BTreeMap<Uuid, u64>,
    reaped: u64,
    /// Counters for completed and failed tasks by authenticated executor.
    node_counts: BTreeMap<String, (u64, u64)>,
}
impl Jobs {
    pub fn new(path: &Path) -> Result<Self> {
        let disk = if path.exists() {
            files::read(path)?
        } else {
            Disk::default()
        };
        // Version 0.9.2 records do not include their completion time on disk.
        let loaded_at = millis();
        let finished = disk
            .records
            .iter()
            .filter(|r| !r.active())
            .map(|r| (r.task.task_id, loaded_at))
            .collect();
        Ok(Self {
            state: Mutex::new(State {
                disk,
                channels: Default::default(),
                reservations: Default::default(),
                history: Default::default(),
                seq: 0,
                opened: 0,
                closed: 0,
                finished,
                reaped: 0,
                node_counts: Default::default(),
            }),
            path: path.to_owned(),
        })
    }
    pub fn with<T>(&self, f: impl FnOnce(&mut State, &Path, u64) -> Result<T>) -> Result<T> {
        let now = millis();
        let mut state = self.state.lock().map_err(|_| "task state unavailable")?;
        state.expire(&self.path, now)?;
        f(&mut state, &self.path, now)
    }
}
impl State {
    fn persist(&mut self, path: &Path, disk: Disk) -> Result<()> {
        files::save(path, &disk)?;
        self.disk = disk;
        Ok(())
    }
    fn event(&mut self, kind: &str, detail: impl ToString) {
        self.seq += 1;
        if self.history.len() == 1024 {
            self.history.pop_front();
        }
        self.history
            .push_back(json!({"seq":self.seq,"kind":kind,"detail":detail.to_string()}));
    }
    fn expire(&mut self, path: &Path, now: u64) -> Result<()> {
        let expired: Vec<_> = self
            .disk
            .leases
            .iter()
            .filter(|l| l.until_unix_ms <= now)
            .cloned()
            .collect();
        let old_results: BTreeSet<_> = self
            .finished
            .iter()
            .filter(|(_, at)| now.saturating_sub(**at) >= 300_000)
            .map(|(id, _)| *id)
            .collect();
        if !expired.is_empty() || !old_results.is_empty() {
            let mut disk = self.disk.clone();
            for lease in &expired {
                disk.retry(lease, now)?;
            }
            disk.leases.retain(|l| l.until_unix_ms > now);
            disk.records
                .retain(|r| !old_results.contains(&r.task.task_id));
            disk.retries.retain(|r| !old_results.contains(&r.task_id));
            disk.leases.retain(|l| !old_results.contains(&l.task_id));
            self.persist(path, disk)?;
            self.finished.retain(|id, _| !old_results.contains(id));
            self.reaped += old_results.len() as u64;
            for l in &expired {
                if self.disk.record(l.task_id).is_ok_and(|r| !r.active()) {
                    self.finished.entry(l.task_id).or_insert(now);
                }
            }
        }
        let old: Vec<_> = self
            .channels
            .iter()
            .filter(|(_, c)| c.seen.saturating_add(CHANNEL_MS) <= now)
            .map(|(id, _)| *id)
            .collect();
        for id in old {
            self.channels.remove(&id);
            self.closed += 1;
            self.event("channel_closed", id);
        }
        self.reservations.retain(|_, r| r.until > now);
        Ok(())
    }
    pub fn submit(&mut self, path: &Path, task: Task, user: Uuid) -> Result<(bool, Record)> {
        if task.input.len() > 1 << 20 {
            return Err(QueueError::Invalid.into());
        }
        if let Some(record) = self
            .disk
            .records
            .iter()
            .find(|r| r.task.task_id == task.task_id)
        {
            if record.owner_id == Some(user) && record.task == task {
                return Ok((false, record.clone()));
            }
            return Err(QueueError::Conflict.into());
        }
        if self
            .disk
            .records
            .iter()
            .filter(|r| r.active() && r.owner_id == Some(user))
            .count()
            >= QUOTA
        {
            return Err(QueueError::Capacity.into());
        }
        let record = Record::new(task, user);
        let mut disk = self.disk.clone();
        disk.records.push(record.clone());
        self.persist(path, disk)?;
        self.event("task_submitted", record.task.task_id);
        Ok((true, record))
    }
    pub fn get(&self, id: Uuid, user: Uuid, admin: bool) -> Result<Record> {
        let r = self.disk.record(id)?;
        if r.owner_id != Some(user) && !admin {
            return Err(QueueError::Missing.into());
        }
        Ok(r.clone())
    }
    fn checked(&mut self, id: Uuid, worker: &str, node: &str, now: u64) -> Result<&mut Channel> {
        let c = self.channels.get_mut(&id).ok_or(QueueError::Missing)?;
        if c.reg.node_id != node || c.reg.worker_id != worker {
            return Err(QueueError::Conflict.into());
        }
        c.seen = now;
        Ok(c)
    }
    pub fn open(&mut self, reg: Registration, node: &str, now: u64) -> Result<Value> {
        if reg.protocol_version != 1 {
            return Err(QueueError::Unsupported.into());
        }
        if reg.node_id != node {
            return Err(QueueError::Forbidden.into());
        }
        if reg.worker_id.is_empty()
            || reg.worker_id.len() > 255
            || reg.capacity == 0
            || reg.capacity > 4096
            || reg.services.len() > 256
            || reg.operations.is_empty()
            || reg.capabilities.len() > 64
        {
            return Err(QueueError::Invalid.into());
        }
        if self
            .channels
            .values()
            .filter(|c| c.phase == Phase::Warmup)
            .count()
            >= 8
        {
            return Err(QueueError::Capacity.into());
        }
        let id = Uuid::new_v4();
        self.channels.insert(
            id,
            Channel {
                reg,
                phase: Phase::Warmup,
                seen: now,
            },
        );
        self.opened += 1;
        self.event("channel_opened", id);
        self.channel_view(id)
    }
    fn busy(&self, id: Uuid) -> bool {
        self.disk.leases.iter().any(|l| {
            l.channel == id
                && self
                    .disk
                    .record(l.task_id)
                    .is_ok_and(|r| r.status == Status::Leased)
        })
    }
    fn channel_view(&self, id: Uuid) -> Result<Value> {
        let c = self.channels.get(&id).ok_or(QueueError::Missing)?;
        Ok(
            json!({"protocol_version":1,"channel_id":id,"worker_id":c.reg.worker_id,"phase":c.phase,
            "busy":self.busy(id),"remaining_ms":CHANNEL_MS}),
        )
    }
    fn count(&self, phase: Phase) -> usize {
        self.channels.values().filter(|c| c.phase == phase).count()
    }
    pub fn ready(&mut self, id: Uuid, req: WorkerRequest, node: &str, now: u64) -> Result<Value> {
        let active = self.count(Phase::Active);
        let target = self.target();
        let c = self.checked(id, &req.worker_id, node, now)?;
        if c.phase == Phase::Draining {
            return Err(QueueError::Conflict.into());
        }
        if !c.reg.reserve && active < target {
            c.phase = Phase::Active;
        }
        self.event("channel_ready", id);
        self.channel_view(id)
    }
    fn target(&self) -> usize {
        let busy = self.disk.records.iter().filter(|r| r.active()).count();
        (4 + busy).min(40)
    }
    pub fn drain(&mut self, id: Uuid, req: WorkerRequest, node: &str, now: u64) -> Result<Value> {
        self.checked(id, &req.worker_id, node, now)?.phase = Phase::Draining;
        let view = self.channel_view(id)?;
        if !self.busy(id) {
            self.channels.remove(&id);
            self.closed += 1;
        }
        self.event("channel_drained", id);
        Ok(view)
    }
    pub fn claim(
        &mut self,
        path: &Path,
        id: Uuid,
        req: WorkerRequest,
        node: &str,
        now: u64,
    ) -> Result<Value> {
        let active = self.count(Phase::Active);
        let target = self.target();
        let c = self.checked(id, &req.worker_id, node, now)?;
        if c.phase == Phase::Warmup && active < target {
            c.phase = Phase::Active;
        }
        if c.phase != Phase::Active {
            return Err(QueueError::Conflict.into());
        }
        if self.busy(id) {
            return Err(QueueError::Conflict.into());
        }
        let c = &self.channels[&id];
        let task = self
            .disk
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.status == Status::Pending
                    && c.supports(&r.task)
                    && !self.disk.retries.iter().any(|retry| {
                        retry.task_id == r.task.task_id && retry.retry_at_unix_ms > now
                    })
            })
            .max_by_key(|(index, r)| (r.task.priority, std::cmp::Reverse(*index)))
            .map(|(_, r)| r.task.clone());
        let Some(task) = task else {
            return Ok(
                json!({"task":null,"lease_id":null,"lease_ttl_ms":0,"lease_deadline_unix_ms":0}),
            );
        };
        let mut disk = self.disk.clone();
        let record = disk.record_mut(task.task_id)?;
        record.status = Status::Leased;
        record.attempts = record.attempts.saturating_add(1);
        let lease = Lease {
            task_id: task.task_id,
            channel: id,
            node_id: node.into(),
            worker: req.worker_id,
            id: Uuid::new_v4(),
            until_unix_ms: now + LEASE_MS,
            max_attempts: 5,
            extra: Default::default(),
        };
        let view = json!({"task":task,"lease_id":lease.id,"lease_ttl_ms":LEASE_MS,"lease_deadline_unix_ms":lease.until_unix_ms});
        disk.leases.push(lease);
        disk.retries.retain(|r| r.task_id != task.task_id);
        self.persist(path, disk)?;
        self.event("task_leased", task.task_id);
        Ok(view)
    }
    pub fn lease(&self, channel: Uuid, req: &LeaseRequest, node: &str) -> Result<&Lease> {
        self.disk
            .leases
            .iter()
            .find(|l| {
                l.channel == channel
                    && l.id == req.lease_id
                    && l.task_id == req.task_id
                    && l.worker == req.worker_id
                    && l.node_id == node
            })
            .ok_or_else(|| QueueError::Conflict.into())
    }
    pub fn renew(
        &mut self,
        path: &Path,
        id: Uuid,
        req: LeaseRequest,
        node: &str,
        now: u64,
    ) -> Result<Value> {
        self.checked(id, &req.worker_id, node, now)?;
        self.lease(id, &req, node)?;
        let mut disk = self.disk.clone();
        let l = disk
            .leases
            .iter_mut()
            .find(|l| l.id == req.lease_id)
            .unwrap();
        l.until_unix_ms = now + LEASE_MS;
        let max = l.max_attempts;
        let r = disk.record(req.task_id)?;
        if r.status != Status::Leased {
            return Err(QueueError::Conflict.into());
        }
        let view = json!({"task_id":req.task_id,"lease_id":req.lease_id,"lease_ttl_ms":LEASE_MS,
            "lease_deadline_unix_ms":now+LEASE_MS,"remaining_ms":LEASE_MS,"attempts":r.attempts,"max_attempts":max});
        self.persist(path, disk)?;
        Ok(view)
    }
    pub fn cancel(
        &mut self,
        path: &Path,
        id: Uuid,
        req: LeaseRequest,
        node: &str,
        now: u64,
    ) -> Result<()> {
        self.checked(id, &req.worker_id, node, now)?;
        let l = self.lease(id, &req, node)?.clone();
        let mut disk = self.disk.clone();
        disk.retry(&l, now)?;
        disk.leases.retain(|s| s.id != l.id);
        self.persist(path, disk)?;
        self.event("task_cancelled", req.task_id);
        Ok(())
    }
    pub fn complete(
        &mut self,
        path: &Path,
        id: Uuid,
        body: Completion,
        node: &str,
        now: u64,
    ) -> Result<()> {
        let req = body.ids;
        self.checked(id, &req.worker_id, node, now)?;
        self.lease(id, &req, node)?;
        let result = body.result;
        let mut disk = self.disk.clone();
        let r = disk.record_mut(req.task_id)?;
        if !matches!(result.status, Status::Completed | Status::Failed) || r.task != result.task {
            return Err(QueueError::Conflict.into());
        }
        if r.status != Status::Leased {
            if r.status == result.status
                && r.result == result.result
                && r.error == result.error
                && r.exchange == result.exchange
            {
                return Ok(());
            }
            return Err(QueueError::Conflict.into());
        }
        r.status = result.status;
        r.result = result.result;
        r.exchange = result.exchange;
        r.error = result.error;
        if let Some(code) = result.extra.get("failure_code") {
            r.extra.insert("failure_code".into(), code.clone());
        }
        self.persist(path, disk)?;
        let count = self.node_counts.entry(node.into()).or_default();
        self.finished.insert(req.task_id, now);
        if result.status == Status::Completed {
            count.0 += 1;
        } else {
            count.1 += 1;
        }
        self.event("task_completed", req.task_id);
        Ok(())
    }
    pub fn capacity(&mut self, req: Reserve, user: Uuid, now: u64) -> Result<Value> {
        if !(1..=40).contains(&req.requested_slots)
            || !(1..=300_000).contains(&req.ttl_ms)
            || req.requested_bytes > 1 << 30
        {
            return Err(QueueError::Invalid.into());
        }
        let active = self
            .disk
            .records
            .iter()
            .filter(|r| r.owner_id == Some(user) && r.active())
            .count();
        let held: usize = self
            .reservations
            .values()
            .filter(|r| r.user == user)
            .map(|r| r.slots as usize)
            .sum();
        let quota = QUOTA.saturating_sub(active + held);
        let reserved: u32 = self.reservations.values().map(|r| r.slots).sum();
        let slots = req
            .requested_slots
            .min(20)
            .min(quota as u32)
            .min(40u32.saturating_sub(reserved));
        let available =
            (256u64 << 20).saturating_sub(self.reservations.values().map(|r| r.bytes).sum::<u64>());
        let bytes = req.requested_bytes.min(available);
        if slots == 0 || (req.requested_bytes > 0 && bytes == 0) {
            return Err(QueueError::Capacity.into());
        }
        let id = Uuid::new_v4();
        let ttl = req.ttl_ms;
        self.reservations.insert(
            id,
            Reservation {
                user,
                slots,
                bytes,
                until: now + ttl,
            },
        );
        Ok(
            json!({"reservation_id":id,"granted_slots":slots,"granted_bytes":bytes,
            "remaining_ms":ttl,"expires_unix_ms":now+ttl,"user_active_tasks":active,"user_remaining_quota":quota-slots as usize}),
        )
    }
    pub fn release(&mut self, id: Uuid, user: Uuid) -> Result<()> {
        if !self.reservations.get(&id).is_some_and(|r| r.user == user) {
            return Err(QueueError::Missing.into());
        }
        self.reservations.remove(&id);
        Ok(())
    }
    pub fn events(&self) -> Value {
        json!(self.history)
    }
    pub fn stats(&self, owner: &garlic_relay_data::control::Authority) -> Value {
        let count = |s| self.disk.records.iter().filter(|r| r.status == s).count();
        json!({"protocol_version":1,"authority":owner,"max_active":40,"max_warmup":8,
            "target_active":self.target(),"active":self.count(Phase::Active),"warmup":self.count(Phase::Warmup),
            "draining":self.count(Phase::Draining),"sleeping_capacity":40usize.saturating_sub(self.target()),
            "channel_sessions":self.channels.len(),"channels_opened":self.opened,"channels_closed":self.closed,
            "pending":count(Status::Pending),"leased":count(Status::Leased),"busy":count(Status::Leased),
            "completed":count(Status::Completed),"failed":count(Status::Failed),"shutting_down":false,"aged_promotions":0,
            "records_reaped":self.reaped,"scheduler_events":self.seq,"reserved_slots":self.reservations.values().map(|r|u64::from(r.slots)).sum::<u64>(),
            "reserved_bytes":self.reservations.values().fold(0u64,|a,r|a.saturating_add(r.bytes)),
            "stream_contract":{"capabilities":garlic_relay_data::control::Capability::ALL,
            "control_version":1,"data_version":1,"max_frame_payload":1048576,"max_window":1048576,"min_window":1024}})
    }
    pub fn nodes(&self, now: u64) -> Value {
        let mut nodes = BTreeMap::new();
        for (id, c) in &self.channels {
            let reg = &c.reg;
            let counts = self
                .node_counts
                .get(&reg.node_id)
                .copied()
                .unwrap_or_default();
            let node=nodes.entry(&reg.node_id).or_insert_with(||json!({"node_id":reg.node_id,
                "services":reg.services,"operations":reg.operations,"capabilities":reg.capabilities,"capacity":reg.capacity,
                "busy":0,"active":0,"warmup":0,"draining":0,"last_seen_ms":CHANNEL_MS,"health":"healthy",
                "ewma_latency_ms":0,"completed_tasks":counts.0,"failed_tasks":counts.1,"slowdowns":0}));
            let phase = match c.phase {
                Phase::Active => "active",
                Phase::Warmup => "warmup",
                Phase::Draining => "draining",
            };
            node[phase] = json!(node[phase].as_u64().unwrap_or(0) + 1);
            node["last_seen_ms"] = json!(node["last_seen_ms"]
                .as_u64()
                .unwrap_or(CHANNEL_MS)
                .min(now.saturating_sub(c.seen)));
            if self.busy(*id) {
                node["busy"] = json!(node["busy"].as_u64().unwrap_or(0) + 1);
            }
        }
        json!(nodes.into_values().collect::<Vec<_>>())
    }
}
