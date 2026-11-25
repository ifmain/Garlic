use crate::{
    http::{header, Failure, Shared},
    streams,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use bytes::Bytes;
use garlic_relay_data::{
    exchange::{Packet, Record},
    stream,
};
use garlic_relay_session::SessionError;
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    ops::Bound::{Excluded, Unbounded},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    sync::{broadcast, mpsc},
    task::JoinSet,
};
use uuid::Uuid;

type Key = (Uuid, Uuid);
type Lanes = BTreeMap<Uuid, mpsc::Sender<Outbound>>;
// The deployed GEX1 client accepts at most 32 packets beyond its contiguous acknowledgement.
const SEND_WINDOW: usize = 32;
pub(crate) const SHAPE_PROFILE: &str = "agent-tls-2026-replay-r1";

pub(crate) fn enabled_versions() -> Result<Option<Vec<u8>>, Failure> {
    let configured = match std::env::var("GARLIC_EXCHANGE_VERSIONS") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(None),
        Err(_) => return Err(invalid_policy()),
    };
    parse_versions(&configured)
        .map(Some)
        .ok_or_else(invalid_policy)
}

fn invalid_policy() -> Failure {
    Failure(
        StatusCode::SERVICE_UNAVAILABLE,
        "invalid exchange protocol configuration".into(),
    )
}

fn parse_versions(configured: &str) -> Option<Vec<u8>> {
    let mut versions = Vec::new();
    for item in configured.split(',') {
        let version = item.trim().parse::<u8>().ok()?;
        if !matches!(version, 2..=4 | 128..=255) {
            return None;
        }
        versions.push(version);
    }
    versions.sort_unstable();
    versions.dedup();
    Some(versions)
}

mod codec;
pub(crate) mod modules;
use codec::LaneWire;
use modules::Module;

#[derive(Clone)]
struct Outbound {
    data: Arc<[u8]>,
    feedback: (u64, u32),
}

#[derive(Default)]
pub struct Exchanges(Mutex<HashMap<Key, mpsc::Sender<Event>>>);
enum Event {
    Join(Uuid, mpsc::Sender<Outbound>),
    Leave(Uuid),
    Packet(Uuid, Module, Bytes),
    FromExit(Uuid, Arc<[u8]>),
    SourceClosed(Uuid),
}
struct Pending {
    seq: u64,
    data: Arc<[u8]>,
    sent: Option<(Instant, Uuid)>,
    selectively_acked: bool,
}
struct Carrier {
    lanes: Lanes,
    received: u64,
    next: u64,
    highest_transmitted: u64,
    unacked: VecDeque<Pending>,
    pending_bytes: usize,
    gaps: BTreeMap<u64, (Module, Bytes)>,
    gap_bytes: usize,
    flows: HashSet<Uuid>,
    pumps: JoinSet<()>,
}
impl Default for Carrier {
    fn default() -> Self {
        Self {
            lanes: Default::default(),
            received: 0,
            next: 1,
            highest_transmitted: 0,
            unacked: Default::default(),
            pending_bytes: 0,
            gaps: Default::default(),
            gap_bytes: 0,
            flows: Default::default(),
            pumps: Default::default(),
        }
    }
}

pub async fn upgrade(
    State(app): State<Shared>,
    Path(exchange): Path<Uuid>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, Failure> {
    if exchange.is_nil() {
        return Err(Failure(
            StatusCode::BAD_REQUEST,
            "invalid exchange identifier".into(),
        ));
    }
    let user = streams::user(app.clone(), &headers).await?;
    let requested = header(&headers, "x-garlic-exchange-version").unwrap_or("1");
    let number = requested
        .parse::<u8>()
        .ok()
        .filter(|n| n.to_string() == requested)
        .ok_or_else(|| {
            Failure(
                StatusCode::BAD_REQUEST,
                "unsupported client exchange version".into(),
            )
        })?;
    let Some(version) = app.modules.get(number) else {
        eprintln!("exchange rejected: disabled protocol {number}");
        return Err(Failure(
            StatusCode::BAD_REQUEST,
            "exchange version disabled".into(),
        ));
    };
    let release = match header(&headers, "x-garlic-client-release") {
        Some("gex4-r1") => "gex4-r1",
        _ => "unknown",
    };
    let platform = match header(&headers, "x-garlic-client-platform") {
        Some("android") => "android",
        Some("windows") => "windows",
        Some("linux") => "linux",
        _ => "unknown",
    };
    let frame_limit = version.limit();
    let mut response = ws
        .max_message_size(frame_limit)
        .max_frame_size(frame_limit)
        .on_upgrade(move |socket| async move {
            if std::env::var("GARLIC_LOG_CLIENT_ACCOUNT").as_deref() == Ok("1") {
                eprintln!("exchange_open protocol={number} reported_release={release} reported_platform={platform} account={}", user.id);
            } else {
                eprintln!("exchange_open protocol={number} reported_release={release} reported_platform={platform}");
            }
            if let Err(e) = lane(socket, app, (user.id, exchange), headers, version).await {
                eprintln!("Client exchange ended: {e}");
            }
        });
    {
        response
            .headers_mut()
            .insert("x-garlic-exchange-version", HeaderValue::from(u16::from(number)));
    }
    Ok(response)
}

async fn lane(
    mut ws: WebSocket,
    app: Shared,
    key: Key,
    headers: HeaderMap,
    version: Module,
) -> Result<(), SessionError> {
    let tx = {
        let mut registry = app.exchanges.0.lock().unwrap();
        if let Some(tx) = registry.get(&key).filter(|tx| !tx.is_closed()) {
            tx.clone()
        } else {
            let (tx, rx) = mpsc::channel(256);
            registry.insert(key, tx.clone());
            tokio::spawn(run(app.clone(), key, tx.clone(), rx));
            tx
        }
    };
    let lane = Uuid::new_v4();
    let mut wire = LaneWire::new(version);
    let (out, mut outgoing) = mpsc::channel(32);
    tx.send(Event::Join(lane, out))
        .await
        .map_err(|_| "exchange closed")?;
    let mut recheck = tokio::time::interval_at(
        tokio::time::Instant::now() + Duration::from_secs(5),
        Duration::from_secs(5),
    );
    recheck.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let result:Result<(),SessionError>=async{
        loop{tokio::select!{
            _=recheck.tick()=>{
                if streams::user(app.clone(),&headers).await.is_err(){
                    let _=ws.send(Message::Close(None)).await;
                    return Err("client authorization expired or device changed".into());
                }
            }
            inbound=ws.recv()=>match inbound.transpose()?{
                Some(Message::Binary(data))=>{
                    if let Some(packet)=wire.receive(data)? {
                        tx.send(Event::Packet(lane,version,packet)).await.map_err(|_|"exchange closed")?;
                    }
                }
                Some(Message::Ping(data))=>ws.send(Message::Pong(data)).await?,
                Some(Message::Pong(_))=>{},
                None|Some(Message::Close(_))=>break,
                _=>return Err("expected binary exchange packet".into()),
            },
            out=outgoing.recv()=>{let Some(data)=out else{break;};wire.send(&mut ws,data).await?;}
        }} Ok(())
    }.await;
    let _ = tx.send(Event::Leave(lane)).await;
    result
}

impl Carrier {
    fn send(
        lanes: &Lanes,
        preferred: Option<Uuid>,
        payload: &Arc<[u8]>,
        feedback: (u64, u32),
    ) -> Option<Uuid> {
        let message = Outbound {
            data: payload.clone(),
            feedback,
        };
        preferred
            .and_then(|id| lanes.get_key_value(&id))
            .into_iter()
            .chain(lanes.iter().filter(|(id, _)| Some(**id) != preferred))
            .find_map(|(id, lane)| lane.try_send(message.clone()).ok().map(|_| *id))
    }
    fn feedback(&self) -> (u64, u32) {
        let selective = self
            .gaps
            .range(..=self.received.saturating_add(32))
            .fold(0, |bits, (&seq, _)| bits | (1 << (seq - self.received - 1)));
        (self.received, selective)
    }
    fn ack(&self, lane: Uuid) {
        let packet = Packet {
            sequence: 0,
            acknowledgement: self.received,
            flag: false,
            records: vec![],
        };
        if let Ok(data) = packet.encode() {
            Self::send(&self.lanes, Some(lane), &Arc::from(data), self.feedback());
        }
    }
    fn flush_outbound(&mut self) {
        let feedback = self.feedback();
        for pending in self.unacked.iter_mut().take(SEND_WINDOW) {
            if pending.sent.is_some() {
                continue;
            }
            let Some(lane) = Self::send(&self.lanes, None, &pending.data, feedback) else {
                break;
            };
            pending.sent = Some((Instant::now(), lane));
            self.highest_transmitted = pending.seq;
        }
    }
    fn retry_outbound(&mut self, now: Instant) {
        let feedback = self.feedback();
        for pending in self.unacked.iter_mut().take(SEND_WINDOW) {
            if pending.selectively_acked {
                continue;
            }
            let due = pending
                .sent
                .filter(|(sent, _)| now.saturating_duration_since(*sent) >= Duration::from_secs(1));
            if let Some((_, previous)) = due {
                let preferred = self
                    .lanes
                    .range((Excluded(previous), Unbounded))
                    .next()
                    .or_else(|| self.lanes.first_key_value())
                    .map(|(&id, _)| id);
                if let Some(lane) = Self::send(&self.lanes, preferred, &pending.data, feedback) {
                    pending.sent = Some((now, lane));
                }
            }
        }
        self.flush_outbound();
    }
    fn confirm(&mut self, ack: u64, selective: u32) -> Result<(), SessionError> {
        if ack > self.highest_transmitted {
            return Err("invalid client exchange acknowledgement".into());
        }
        let last_selected = u64::from(u32::BITS - selective.leading_zeros());
        if last_selected > self.highest_transmitted - ack {
            return Err("invalid selective client acknowledgement".into());
        }
        while self.unacked.front().is_some_and(|entry| entry.seq <= ack) {
            self.pending_bytes -= self.unacked.pop_front().unwrap().data.len();
        }
        if selective != 0 {
            for pending in self
                .unacked
                .iter_mut()
                .take_while(|p| p.seq - ack <= last_selected)
            {
                let bit = (pending.seq - ack - 1) as u32;
                pending.selectively_acked |= selective & (1 << bit) != 0;
            }
        }
        self.flush_outbound();
        Ok(())
    }
    fn confirm_lane(&mut self, lane: Uuid, ack: u64, selective: u32) -> Result<bool, SessionError> {
        if let Err(error) = self.confirm(ack, selective) {
            // This packet has not changed exchange state. Other lanes can still deliver it.
            self.lanes.remove(&lane);
            if self.lanes.is_empty() {
                return Err(error);
            }
            eprintln!("Client exchange lane declined: {error}");
            return Ok(false);
        }
        Ok(true)
    }
    fn queue(&mut self, id: Uuid, kind: u8, data: &[u8]) -> Result<(), SessionError> {
        let record = Record {
            owner: *id.as_bytes(),
            kind,
            payload: data,
        };
        let packet = Packet {
            sequence: self.next,
            acknowledgement: self.received,
            flag: true,
            records: vec![record],
        };
        let encoded: Arc<[u8]> = packet
            .encode()
            .map_err(|_| "invalid outgoing client packet")?
            .into();
        if self.unacked.len() >= 512 || self.pending_bytes + encoded.len() > 8 * 1024 * 1024 {
            return Err("client exchange window exhausted".into());
        }
        self.pending_bytes += encoded.len();
        self.unacked.push_back(Pending {
            seq: self.next,
            data: encoded,
            sent: None,
            selectively_acked: false,
        });
        self.next = self
            .next
            .checked_add(1)
            .ok_or("exchange sequence exhausted")?;
        self.flush_outbound();
        Ok(())
    }
    async fn consume(
        &mut self,
        app: &Shared,
        user: Uuid,
        tx: &mpsc::Sender<Event>,
        version: Module,
        bytes: &[u8],
    ) -> Result<(), SessionError> {
        let (packet, _) = version.packet(bytes)?;
        for r in packet.records {
            let flow = Uuid::from_bytes(r.owner);
            if !self.flows.contains(&flow) {
                let (id, mut source) = app
                    .hub
                    .subscribe(flow, user)
                    .map_err(|_| "unknown flow for this user")?;
                if id != flow {
                    return Err("exchange owner must be flow identifier".into());
                }
                self.flows.insert(flow);
                let tx = tx.clone();
                self.pumps.spawn(async move {
                    loop {
                        match source.recv().await {
                            Ok(data) => {
                                if tx.send(Event::FromExit(flow, data)).await.is_err() {
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(_))
                            | Err(broadcast::error::RecvError::Closed) => {
                                let _ = tx.send(Event::SourceClosed(flow)).await;
                                break;
                            }
                        }
                    }
                });
            }
            match r.kind {
                0 => {
                    let mut data = r.payload;
                    while !data.is_empty() {
                        let (frame, n) =
                            stream::decode(data).map_err(|_| "invalid client stream data")?;
                        if frame.stream_id != r.owner {
                            return Err("inner flow identifier mismatch".into());
                        }
                        data = &data[n..];
                    }
                    app.hub
                        .upstream(flow, r.payload.to_vec())
                        .await
                        .map_err(|_| "exit unavailable")?;
                }
                1 => self.queue(flow, 2, r.payload)?,
                2 => {}
                3 => {
                    app.hub
                        .close(flow, user)
                        .map_err(|_| "flow ownership changed")?;
                    self.flows.remove(&flow);
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
    async fn inbound(
        &mut self,
        lane: Uuid,
        app: &Shared,
        user: Uuid,
        tx: &mpsc::Sender<Event>,
        version: Module,
        bytes: Bytes,
    ) -> Result<(), SessionError> {
        let (packet, selective) = version.packet(&bytes)?;
        let seq = packet.sequence;
        let ack = packet.acknowledgement;
        let flag = packet.flag;
        if !self.confirm_lane(lane, ack, selective)? {
            return Ok(());
        }
        if seq > self.received {
            if seq - self.received > 64 {
                return Err("client sequence outside exchange window".into());
            }
            if let std::collections::btree_map::Entry::Vacant(entry) = self.gaps.entry(seq) {
                if self.gap_bytes.saturating_add(bytes.len()) > 8 * 1024 * 1024
                    && seq != self.received + 1
                {
                    return Err("client exchange receive buffer exhausted".into());
                }
                self.gap_bytes += bytes.len();
                entry.insert((version, bytes));
            }
            while let Some((version, data)) = self.gaps.remove(&(self.received + 1)) {
                self.gap_bytes -= data.len();
                self.consume(app, user, tx, version, &data).await?;
                self.received += 1;
                if self.received == u64::MAX {
                    return Err("exchange sequence exhausted".into());
                }
            }
        }
        if flag || seq != 0 {
            self.ack(lane);
        }
        Ok(())
    }
}

async fn run(app: Shared, key: Key, tx: mpsc::Sender<Event>, mut rx: mpsc::Receiver<Event>) {
    let (user, _) = key;
    let mut carrier = Carrier::default();
    let mut idle = Instant::now();
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _=tick.tick()=>{
                if carrier.lanes.is_empty() && idle.elapsed()>Duration::from_secs(60){break;}
                carrier.retry_outbound(Instant::now());
            }
            event=rx.recv()=>match event{
                None=>break,
                Some(Event::Join(id,lane))=>{
                    carrier.lanes.insert(id,lane);
                    carrier.flush_outbound();
                    idle=Instant::now();
                }
                Some(Event::Leave(id))=>{carrier.lanes.remove(&id);idle=Instant::now();}
                Some(Event::Packet(id,version,data))=>{
                    if !carrier.lanes.contains_key(&id){continue;}
                    if let Err(e)=carrier.inbound(id,&app,user,&tx,version,data).await{eprintln!("Client exchange packet declined: {e}");break;}
                }
                Some(Event::FromExit(flow,bytes))=>{
                    if carrier.flows.contains(&flow) && carrier.queue(flow,0,&bytes).is_err(){break;}
                }
                Some(Event::SourceClosed(flow))=>{
                    if carrier.flows.remove(&flow) {
                        let _=app.hub.close(flow,user);
                        let reset=stream::StreamFrame{stream_id:*flow.as_bytes(),content:stream::Content::Reset(1)}.encode().unwrap();
                        if carrier.queue(flow,0,&reset).is_err(){break;}
                    }
                }
            }
        }
    }
    for flow in carrier.flows {
        let _ = app.hub.close(flow, user);
    }
    let mut map = app.exchanges.0.lock().unwrap();
    if map
        .get(&key)
        .is_some_and(|current| current.same_channel(&tx))
    {
        map.remove(&key);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rollout_policy_validates_the_whole_list() {
        assert_eq!(super::parse_versions("4, 2,4"), Some(vec![2, 4]));
        for bad in ["", "1", "1,2", "1,", "1,0", "1,5", "2,typo"] {
            assert_eq!(super::parse_versions(bad), None);
        }
    }
    use super::*;

    #[test]
    fn queues_packets_beyond_the_native_receive_window() {
        let mut carrier = Carrier::default();
        let (sender, mut receiver) = mpsc::channel(64);
        carrier.lanes.insert(Uuid::from_u128(1), sender);
        let flow = Uuid::from_u128(2);
        for _ in 0..=SEND_WINDOW {
            carrier.queue(flow, 2, b"").unwrap();
        }
        for sequence in 1..=SEND_WINDOW as u64 {
            let bytes = receiver.try_recv().unwrap();
            assert_eq!(Packet::decode(&bytes.data).unwrap().sequence, sequence);
        }
        assert!(receiver.try_recv().is_err());
        assert!(carrier.confirm(SEND_WINDOW as u64 + 1, 0).is_err());
        carrier.confirm(1, 0).unwrap();
        let bytes = receiver.try_recv().unwrap();
        assert_eq!(
            Packet::decode(&bytes.data).unwrap().sequence,
            SEND_WINDOW as u64 + 1
        );
    }

    #[test]
    fn invalid_lane_feedback_preserves_other_lane_and_pending_data() {
        let mut carrier = Carrier::default();
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let (a, mut first_queue) = mpsc::channel(4);
        let (b, mut second_queue) = mpsc::channel(4);
        carrier.lanes.insert(first, a);
        carrier.lanes.insert(second, b);
        carrier.queue(Uuid::from_u128(3), 2, b"").unwrap();
        let original = first_queue.try_recv().unwrap();
        assert!(!carrier.confirm_lane(first, 2, 0).unwrap());
        assert!(!carrier.lanes.contains_key(&first));
        carrier.retry_outbound(Instant::now() + Duration::from_secs(2));
        assert_eq!(second_queue.try_recv().unwrap().data, original.data);
        assert!(carrier.confirm_lane(second, 1, 0).unwrap());
        assert!(carrier.unacked.is_empty());
        assert!(carrier.confirm_lane(second, 1, 1).is_err());
    }

    #[test]
    fn retries_in_flight_packets_on_another_lane() {
        let mut carrier = Carrier::default();
        let (first, mut first_queue) = mpsc::channel(4);
        let (second, mut second_queue) = mpsc::channel(4);
        carrier.lanes.insert(Uuid::from_u128(1), first);
        carrier.lanes.insert(Uuid::from_u128(2), second);
        carrier.queue(Uuid::from_u128(3), 2, b"").unwrap();
        carrier.queue(Uuid::from_u128(3), 2, b"").unwrap();
        let originals = [
            first_queue.try_recv().unwrap(),
            first_queue.try_recv().unwrap(),
        ];
        carrier.retry_outbound(Instant::now() + Duration::from_secs(2));
        for original in originals {
            assert_eq!(second_queue.try_recv().unwrap().data, original.data);
        }
        assert!(first_queue.try_recv().is_err());
    }

    #[test]
    fn selective_feedback_avoids_resending_buffered_packets() {
        let mut carrier = Carrier::default();
        let (sender, mut receiver) = mpsc::channel(8);
        carrier.lanes.insert(Uuid::from_u128(1), sender);
        for _ in 0..2 {
            carrier.queue(Uuid::from_u128(2), 2, b"").unwrap();
            receiver.try_recv().unwrap();
        }
        assert!(carrier.confirm(0, 0b100).is_err());
        carrier.confirm(0, 0b10).unwrap();
        carrier.retry_outbound(Instant::now() + Duration::from_secs(2));
        let resent = receiver.try_recv().unwrap();
        assert_eq!(Packet::decode(&resent.data).unwrap().sequence, 1);
        assert!(receiver.try_recv().is_err());
        carrier.gaps.insert(
            2,
            (
                Module {
                    version: 3,
                    frame: modules::Frame::Fragments,
                    body: modules::Body::SelectiveAck,
                    record_size: None,
                },
                Default::default(),
            ),
        );
        carrier.gaps.insert(
            32,
            (
                Module {
                    version: 3,
                    frame: modules::Frame::Fragments,
                    body: modules::Body::SelectiveAck,
                    record_size: None,
                },
                Default::default(),
            ),
        );
        assert_eq!(carrier.feedback(), (0, 0x8000_0002));
    }
}
