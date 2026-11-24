//! Worker-side stream pump. The caller supplies destination policy and registered node identity.
use crate::{Inbound, SessionError, WorkerConnection};
use garlic_relay_data::control::{
    Authority, ErrorCode, Message, Role, StreamOpen, Subscription, UserSnapshot,
};
use garlic_relay_data::stream::{self, Content, StreamFrame};
use garlic_relay_data::{Direction, Frame};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

mod pool;
pub use pool::FlowPool;
use pool::{Slot, State};

struct Sent {
    seq: u64,
    bytes: Vec<u8>,
    sent_at: Instant,
    retries: u8,
}

struct Flow {
    open: StreamOpen,
    target: SocketAddr,
    socket: Option<TcpStream>,
    next_rx: u64,
    next_tx: u64,
    credit: usize,
    pending_ack: VecDeque<Sent>,
    last_ack: Option<u64>,
    rtt: Option<(f64, f64)>,
    replay: bool,
    pending_write: VecDeque<u8>,
    peer_finished: bool,
    write_closed: bool,
    read_closed: bool,
}

impl Flow {
    fn new(open: StreamOpen, target: SocketAddr) -> Self {
        Self {
            credit: open.window_size as usize,
            open,
            target,
            socket: None,
            next_rx: 0,
            next_tx: 0,
            pending_ack: VecDeque::new(),
            last_ack: None,
            rtt: None,
            replay: false,
            pending_write: VecDeque::new(),
            peer_finished: false,
            write_closed: false,
            read_closed: false,
        }
    }

    fn send(&self, relay: &mut WorkerConnection, content: Content<'_>) -> Result<(), SessionError> {
        let bytes = StreamFrame {
            // The deployed relay uses the flow UUID in both binary headers.
            stream_id: *self.open.flow_id.as_bytes(),
            content,
        }
        .encode()
        .map_err(|_| "invalid outgoing stream frame")?;
        relay.send_data(&Frame {
            direction: Direction::Two,
            flow_id: *self.open.flow_id.as_bytes(),
            payload: &bytes,
        })
    }

    fn acknowledge(&self, relay: &mut WorkerConnection) -> Result<(), SessionError> {
        self.send(
            relay,
            Content::Window {
                offset: self.next_rx.saturating_sub(1),
                credit: self.open.window_size - self.pending_write.len() as u32,
            },
        )
    }

    fn update_window(&mut self, offset: u64, credit: u32) {
        if offset >= self.next_tx || self.last_ack.is_some_and(|old| offset < old) {
            return;
        }
        self.last_ack = Some(offset);
        while self
            .pending_ack
            .front()
            .is_some_and(|sent| sent.seq <= offset)
        {
            let sent = self.pending_ack.pop_front().unwrap();
            if sent.retries == 0 && sent.seq == offset {
                let sample = sent.sent_at.elapsed().as_secs_f64();
                self.rtt = Some(match self.rtt {
                    Some((mean, variance)) => (
                        0.875 * mean + 0.125 * sample,
                        0.75 * variance + 0.25 * (mean - sample).abs(),
                    ),
                    None => (sample, sample / 2.0),
                });
            }
        }
        let outstanding: usize = self.pending_ack.iter().map(|sent| sent.bytes.len()).sum();
        self.credit = (credit.min(self.open.window_size) as usize).saturating_sub(outstanding);
    }

    fn retransmit(&mut self, relay: &mut WorkerConnection) -> Result<(), SessionError> {
        if self.replay {
            for sent in &self.pending_ack {
                self.send(
                    relay,
                    Content::Data {
                        offset: sent.seq,
                        bytes: &sent.bytes,
                    },
                )?;
            }
            let now = Instant::now();
            for sent in &mut self.pending_ack {
                sent.sent_at = now;
                sent.retries = sent.retries.saturating_add(1);
            }
            if self.next_rx > 0 {
                self.acknowledge(relay)?;
            }
            if self.read_closed {
                self.send(relay, Content::HalfClose(2))?;
            }
            self.replay = false;
        } else if let Some(sent) = self.pending_ack.front() {
            let base = self
                .rtt
                .map(|(mean, variance)| mean + 4.0 * variance)
                .unwrap_or(1.0)
                .max(1.0);
            let delay = (base * f64::from(1_u32 << sent.retries.min(6))).min(60.0);
            if sent.sent_at.elapsed() >= Duration::from_secs_f64(delay) {
                self.send(
                    relay,
                    Content::Data {
                        offset: sent.seq,
                        bytes: &sent.bytes,
                    },
                )?;
                let sent = self.pending_ack.front_mut().unwrap();
                sent.sent_at = Instant::now();
                sent.retries = sent.retries.saturating_add(1);
            }
        }
        Ok(())
    }

    fn receive(
        &mut self,
        relay: &mut WorkerConnection,
        frame: StreamFrame<'_>,
    ) -> Result<bool, SessionError> {
        if frame.stream_id != *self.open.flow_id.as_bytes() {
            return Ok(false);
        }
        match frame.content {
            Content::Open if self.socket.is_none() => {
                match TcpStream::connect_timeout(&self.target, Duration::from_secs(3)) {
                    Ok(socket) => {
                        if socket.set_nonblocking(true).is_err() {
                            self.send(relay, Content::Reset(5))?;
                            return Ok(false);
                        }
                        self.socket = Some(socket);
                        self.send(relay, Content::Opened)?;
                    }
                    Err(_) => {
                        self.send(relay, Content::Reset(5))?;
                        return Ok(false);
                    }
                }
            }
            Content::Reset(_) | Content::Close => return Ok(false),
            _ if self.socket.is_none() => return Ok(false),
            Content::Open => self.send(relay, Content::Opened)?,
            Content::Data { offset, .. } if offset < self.next_rx => self.acknowledge(relay)?,
            Content::Data { offset, bytes }
                if offset == self.next_rx
                    && !self.peer_finished
                    && bytes.len() <= self.open.window_size as usize - self.pending_write.len() =>
            {
                self.pending_write.extend(bytes);
                self.next_rx += 1;
                self.acknowledge(relay)?;
            }
            Content::Window { offset, credit } => self.update_window(offset, credit),
            Content::HalfClose(1) => self.peer_finished = true,
            Content::HalfClose(2) => {
                self.read_closed = true;
                let _ = self.socket.as_ref().unwrap().shutdown(Shutdown::Read);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn poll(&mut self, relay: &mut WorkerConnection) -> Result<bool, SessionError> {
        if self.socket.is_none() {
            return Ok(true);
        }
        self.retransmit(relay)?;
        if !self.pending_write.is_empty() {
            let data = self.pending_write.as_slices().0;
            match self.socket.as_mut().unwrap().write(data) {
                Ok(0) => return Ok(false),
                Ok(n) => {
                    self.pending_write.drain(..n);
                    self.acknowledge(relay)?;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    self.send(relay, Content::Reset(5))?;
                    return Ok(false);
                }
            }
        }
        if self.peer_finished && self.pending_write.is_empty() && !self.write_closed {
            if self
                .socket
                .as_ref()
                .unwrap()
                .shutdown(Shutdown::Write)
                .is_err()
            {
                self.send(relay, Content::Reset(5))?;
                return Ok(false);
            }
            self.write_closed = true;
        }
        if self.credit > 0 && !self.read_closed {
            let mut buf = [0u8; 16384];
            let limit = self.credit.min(buf.len());
            match self.socket.as_mut().unwrap().read(&mut buf[..limit]) {
                Ok(0) => {
                    self.read_closed = true;
                    self.send(relay, Content::HalfClose(2))?;
                }
                Ok(n) => {
                    let offset = self.next_tx;
                    self.pending_ack.push_back(Sent {
                        seq: offset,
                        bytes: buf[..n].to_vec(),
                        sent_at: Instant::now(),
                        retries: 0,
                    });
                    self.next_tx += 1;
                    self.credit -= n;
                    self.send(
                        relay,
                        Content::Data {
                            offset,
                            bytes: &buf[..n],
                        },
                    )?;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    self.send(relay, Content::Reset(5))?;
                    return Ok(false);
                }
            }
        }
        if self.read_closed && self.write_closed && self.pending_ack.is_empty() {
            self.send(relay, Content::Close)?;
            return Ok(false);
        }
        Ok(true)
    }
}

/// The caller resolves permitted destinations once. No second lookup occurs when
/// opening a stream, so the connection uses precisely the approved address.
pub struct Worker<P> {
    pool: FlowPool,
    worker_id: Uuid,
    select_target: P,
    capacity: usize,
    users: Option<UserSnapshot>,
    flows: HashMap<[u8; 16], Slot>,
    draining: bool,
}

impl<P: Fn(&StreamOpen) -> Option<SocketAddr>> Worker<P> {
    pub fn new(owner: &Authority, capacity: usize, select_target: P) -> Self {
        Self::with_pool(
            FlowPool::new(owner.clone(), capacity, Duration::from_secs(60)),
            Uuid::new_v4(),
            capacity,
            select_target,
        )
    }

    pub fn with_pool(pool: FlowPool, worker_id: Uuid, capacity: usize, select_target: P) -> Self {
        Self {
            pool,
            worker_id,
            capacity,
            select_target,
            users: None,
            flows: HashMap::new(),
            draining: false,
        }
    }

    fn admit(&mut self, open: StreamOpen, admin: bool, now: u64) -> Result<(), ErrorCode> {
        open.validate(&self.pool.authority, now)
            .map_err(|e| e.code)?;
        let users = self.users.as_ref().ok_or(ErrorCode::Unauthorized)?;
        let user = users
            .users
            .iter()
            .find(|u| u.id == open.user_id && u.enabled)
            .ok_or(ErrorCode::Unauthorized)?;
        if admin && user.role != Role::Admin {
            return Err(ErrorCode::Forbidden);
        }
        if matches!(user.subscription, Subscription::ExpiresAt {expires_at} if expires_at <= now / 1000)
        {
            return Err(ErrorCode::SubscriptionExpired);
        }
        if self.draining || self.flows.len() >= self.capacity {
            return Err(ErrorCode::CapacityExhausted);
        }
        let target = (self.select_target)(&open).ok_or(ErrorCode::InvalidDestination)?;
        let id = *open.flow_id.as_bytes();
        if self.flows.contains_key(&id) {
            return Err(ErrorCode::InvalidRequest);
        }
        let slot = self
            .pool
            .insert(id, self.worker_id, Flow::new(open, target))?;
        self.flows.insert(id, slot);
        Ok(())
    }

    pub fn run(&mut self, relay: &mut WorkerConnection) -> Result<(), SessionError> {
        self.run_until(relay, || false)
    }

    /// The exit initiates a graceful drain. The coordinator acknowledges after
    /// it has dealt with the channel's assignments.
    pub fn run_until(
        &mut self,
        relay: &mut WorkerConnection,
        shutdown: impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        let result = self.run_connection(relay, shutdown);
        let parked = Instant::now();
        for (_, slot) in self.flows.drain() {
            let mut assignment = slot.lock().unwrap();
            if assignment.state == State::Attached(self.worker_id) {
                assignment.state = State::Parked(parked);
            }
        }
        self.users = None;
        self.draining = false;
        self.pool.expire();
        result
    }

    fn run_connection(
        &mut self,
        relay: &mut WorkerConnection,
        shutdown: impl Fn() -> bool,
    ) -> Result<(), SessionError> {
        let mut last_cleanup = Instant::now();
        let mut drain_started = None;
        loop {
            if !self.draining && shutdown() {
                self.draining = true;
                drain_started = Some(Instant::now());
                relay.send_control(&Message::Drain)?;
            }
            if drain_started
                .is_some_and(|start: Instant| start.elapsed() >= Duration::from_secs(15))
            {
                return Err("timed out waiting for relay drain acknowledgement".into());
            }
            if last_cleanup.elapsed() >= Duration::from_secs(2) {
                self.pool.expire();
                last_cleanup = Instant::now();
            }
            let idle = if self.flows.is_empty() {
                Duration::from_secs(2)
            } else {
                Duration::from_millis(5)
            };
            let msg = match relay.read(idle) {
                Ok(message) => Some(message),
                Err(e) if receive_timeout(&*e) => None,
                Err(e) => return Err(e),
            };
            match msg {
                Some(Inbound::Control(Message::UserSync { snapshot })) => {
                    if self
                        .users
                        .as_ref()
                        .is_none_or(|old| old.revision <= snapshot.revision)
                    {
                        self.users = Some(snapshot);
                    }
                }
                Some(Inbound::Control(Message::Open {
                    request_id,
                    open,
                    admin,
                })) => {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
                    let error = self.admit(open, admin, now).err().map(Into::into);
                    relay.send_control(&Message::Opened { request_id, error })?;
                }
                Some(Inbound::Control(Message::CloseFlow { flow_id })) => {
                    if let Some(slot) = self.flows.remove(flow_id.as_bytes()) {
                        let mut assignment = slot.lock().unwrap();
                        if assignment.state == State::Attached(self.worker_id) {
                            assignment.state = State::Closed;
                        }
                        drop(assignment);
                        self.pool.expire();
                    }
                }
                Some(Inbound::Control(Message::AdoptFlow { flow_id })) if !self.draining => {
                    if self.flows.len() < self.capacity
                        || self.flows.contains_key(flow_id.as_bytes())
                    {
                        if let Some(slot) = self.pool.adopt(flow_id.as_bytes(), self.worker_id) {
                            self.flows.insert(*flow_id.as_bytes(), slot);
                        }
                    }
                }
                Some(Inbound::Control(Message::Drained)) if self.draining => return Ok(()),
                Some(Inbound::Control(Message::Drain)) => {
                    relay.send_logical(4, &[])?;
                    return Ok(());
                }
                Some(Inbound::Data {
                    direction: Direction::One,
                    flow_id,
                    bytes,
                }) => {
                    if let Some(slot) = self.flows.get(&flow_id) {
                        let mut assignment = slot.lock().unwrap();
                        if assignment.state != State::Attached(self.worker_id) {
                            continue;
                        }
                        let mut input = bytes.as_slice();
                        while !input.is_empty() {
                            let Ok((frame, n)) = stream::decode(input) else {
                                assignment.flow.send(relay, Content::Reset(1))?;
                                assignment.state = State::Closed;
                                break;
                            };
                            if !assignment.flow.receive(relay, frame)? {
                                assignment.state = State::Closed;
                                break;
                            }
                            input = &input[n..];
                        }
                    }
                }
                Some(Inbound::Closed) => return Ok(()),
                _ => {}
            }
            let mut finished = Vec::new();
            for (id, slot) in &self.flows {
                let mut assignment = slot.lock().unwrap();
                if assignment.state != State::Attached(self.worker_id) {
                    finished.push(*id);
                } else if !assignment.flow.poll(relay)? {
                    assignment.state = State::Closed;
                    finished.push(*id);
                }
            }
            if !finished.is_empty() {
                for id in finished {
                    self.flows.remove(&id);
                }
                self.pool.expire();
            }
        }
    }
}

fn receive_timeout(e: &(dyn std::error::Error + 'static)) -> bool {
    let io =
        e.downcast_ref::<io::Error>()
            .or_else(|| match e.downcast_ref::<tungstenite::Error>() {
                Some(tungstenite::Error::Io(err)) => Some(err),
                _ => None,
            });
    io.is_some_and(|e| {
        matches!(
            e.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        )
    })
}
