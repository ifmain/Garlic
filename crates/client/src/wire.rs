use crate::{api::Api, settings::origin};
use garlic_relay_data::{
    envelope,
    exchange::{Packet, Record},
    gex4,
    stream::{self, Content, StreamFrame},
};
use garlic_state::StateError;
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, ServerName},
    ClientConfig, ClientConnection, RootCertStore, StreamOwned,
};
use std::{
    collections::{BTreeMap, VecDeque},
    env,
    io::{self, Read, Write},
    net::{Shutdown, TcpStream},
    sync::Arc,
    time::{Duration, Instant},
};
use tungstenite::{
    client::IntoClientRequest,
    http::{header::AUTHORIZATION, HeaderValue},
    Message, WebSocket,
};
use uuid::Uuid;

fn trace(event: &str, sequence: u64, size: usize) {
    if std::env::var_os("GARLIC_FIXTURE_TRACE").is_some() {
        eprintln!("exchange {event} sequence={sequence} bytes={size}");
    }
}
const WINDOW: usize = 1 << 20;
enum Transport {
    Plain(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}
impl Read for Transport {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Plain(s) => s.read(b),
            Self::Tls(s) => s.read(b),
        }
    }
}
impl Write for Transport {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        match self {
            Self::Plain(s) => s.write(b),
            Self::Tls(s) => s.write(b),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Plain(s) => s.flush(),
            Self::Tls(s) => s.flush(),
        }
    }
}
impl Transport {
    fn socket(&self) -> &TcpStream {
        match self {
            Self::Plain(s) => s,
            Self::Tls(s) => &s.sock,
        }
    }
}
#[cfg(windows)]
fn upload_ready(socket: &TcpStream) -> io::Result<bool> {
    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Networking::WinSock::{WSAGetLastError, WSAPoll, WSAPOLLFD, POLLRDNORM};
    let mut fd = WSAPOLLFD { fd: socket.as_raw_socket() as usize, events: POLLRDNORM, revents: 0 };
    match unsafe { WSAPoll(&mut fd, 1, 0) } {
        -1 => Err(io::Error::from_raw_os_error(unsafe { WSAGetLastError() })),
        count => Ok(count != 0),
    }
}
fn timed(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClientProtocol {
    Gex1,
    Gex2,
    Gex3,
    Gex4,
    Module(release_cache::profile::Module),
}

impl ClientProtocol {
    pub fn negotiate(info: &serde_json::Value) -> Result<Self, StateError> {
        let policy = env::var("GARLIC_ALPHA_PROTOCOLS").ok();
        Self::select(info, policy.as_deref())
    }
    fn select(info: &serde_json::Value, policy: Option<&str>) -> Result<Self, StateError> {
        let enabled = |n: &str| policy.is_none_or(|s| s.split(',').any(|v| v.trim()==n));
        let offered = |n: u8| info["exchange_versions"].as_array()
            .is_some_and(|v| v.iter().any(|value| value == n));
        if enabled("4") && info["exchange_extensions"]
            .as_array()
            .is_some_and(|v| v.iter().any(|n| n == gex4::CAPABILITY))
        {
            Ok(Self::Gex4)
        } else if enabled("3") && offered(3) {
            Ok(Self::Gex3)
        } else if enabled("2") && offered(2) {
            Ok(Self::Gex2)
        } else if policy.is_none() && (info["exchange_version"] == 1 || offered(1)) {
            Ok(Self::Gex1)
        } else {
            Err("no compatible exchange protocol".into())
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Gex1 => "GEX1",
            Self::Gex2 => "GEX2",
            Self::Gex3 => "GEX3",
            Self::Gex4 => "GEX4",
            Self::Module(_) => "Module",
        }
    }

    fn selective(self) -> bool {
        matches!(self, Self::Gex3 | Self::Module(release_cache::profile::Module {body: release_cache::profile::Body::SelectiveAck, ..}))
    }
    fn tlv(self) -> bool {
        matches!(self, Self::Gex4 | Self::Module(release_cache::profile::Module {frame: release_cache::profile::Frame::Tlv, ..}))
    }

    fn version(self) -> Option<u8> {
        match self {
            Self::Gex1 => None, Self::Gex2 => Some(2),
            Self::Gex3 => Some(3), Self::Gex4 => Some(4), Self::Module(m) => Some(m.version),
        }
    }
}

struct Lane {
    socket: WebSocket<Transport>,
    decoder: envelope::Decoder,
    message_id: std::num::NonZeroU32,
}
impl Lane {
    fn send(&mut self, protocol: ClientProtocol, frame: Message) -> Result<(), StateError> {
        let Message::Binary(data) = frame else {return Ok(self.socket.send(frame)?);};
        let encoded;
        let payload = if protocol.selective() {
            let ack = Packet::decode(&data).map_err(|_| "invalid outgoing packet")?.acknowledgement;
            encoded = garlic_relay_data::exchange::gex3_wire(data.to_vec(), ack, 0)
                .map_err(|_| "invalid selective acknowledgement packet")?;
            encoded.as_slice()
        } else { data.as_ref() };
        if protocol.tlv() {
            self.socket.send(Message::Binary(gex4::wrap(payload).into()))?;
        } else if protocol == ClientProtocol::Gex1 {
            self.socket.send(Message::Binary(data))?;
        } else {
            for part in envelope::fragments(self.message_id,payload).map_err(|_| "invalid fragment size")? {
                self.socket.send(Message::Binary(part.into()))?;
            }
            self.message_id=std::num::NonZeroU32::new(self.message_id.get().wrapping_add(1))
                .unwrap_or(std::num::NonZeroU32::MIN);
        }
        Ok(())
    }
}

pub struct Exchange {
    pub(crate) record_limit: usize,
    protocol: ClientProtocol,
    traffic: Arc<crate::metrics::Traffic>,
    lanes: Vec<Lane>,
    last_poll: Instant,
    flow: [u8; 16],
    next: u64,
    received: u64,
    pending: VecDeque<(u64, Vec<u8>, Instant)>,
    gaps: BTreeMap<u64, Vec<u8>>,
    opened: bool,
    peer_closed: bool,
    ended: bool,
    tx_offset: u64,
    tx_credit: usize,
    unconfirmed: VecDeque<(u64, Vec<u8>, Instant)>,
    stream_ack: Option<u64>,
    rx_offset: u64,
    rx_gaps: BTreeMap<u64, Vec<u8>>,
    rx_buffered: usize,
    early: Vec<u8>,
    connected_response: bool,
}
impl Exchange {
    fn send(&mut self, frame: Message) -> Result<(), StateError> {
        loop {
            let Some(lane) = self.lanes.first_mut() else {
                return Err("exchange transport is closed".into());
            };
            match lane.send(self.protocol,frame.clone()) {
                Ok(()) => return Ok(()),
                Err(e) if self.lanes.len() == 1 => return Err(e.into()),
                Err(_) => {
                    self.lanes.remove(0);
                }
            }
        }
    }
    pub fn new(
        api: &Api,
        protocol: ClientProtocol,
        endpoint: &str,
        token: &str,
        device: Option<&str>,
        flow: Uuid,
        timeout: Duration,
        traffic: Arc<crate::metrics::Traffic>,
    ) -> Result<Self, StateError> {
        let exchange_id = Uuid::new_v4();
        let mut lanes = Vec::with_capacity(2);
        for active in [true, false] {
            lanes.push(Self::lane(
                api,
                protocol,
                endpoint,
                token,
                device,
                exchange_id,
                active,
                timeout,
            )?);
        }
        Ok(Self {
            protocol,
            record_limit: 16384,
            traffic,
            lanes,
            last_poll: Instant::now(),
            flow: *flow.as_bytes(),
            next: 1,
            received: 0,
            pending: Default::default(),
            gaps: Default::default(),
            opened: false,
            peer_closed: false,
            ended: false,
            tx_offset: 0,
            tx_credit: WINDOW,
            unconfirmed: Default::default(),
            stream_ack: None,
            rx_offset: 0,
            rx_gaps: BTreeMap::new(),
            rx_buffered: 0,
            early: vec![],
            connected_response: false,
        })
    }
    fn lane(
        api: &Api,
        protocol: ClientProtocol,
        endpoint: &str,
        token: &str,
        device: Option<&str>,
        exchange_id: Uuid,
        active: bool,
        timeout: Duration,
    ) -> Result<Lane, StateError> {
        let mut url = reqwest::Url::parse(&origin(endpoint)?)?;
        let host = url
            .host_str()
            .ok_or("missing exchange host")?
            .trim_matches(['[', ']'])
            .to_owned();
        let port = url.port_or_known_default().ok_or("missing exchange port")?;
        let tcp = api
            .exchange_peers(&host, port)?
            .into_iter()
            .find_map(|addr| TcpStream::connect_timeout(&addr, timeout).ok())
            .ok_or("could not connect to exchange")?;
        tcp.set_nodelay(true)?;
        tcp.set_write_timeout(Some(timeout))?;
        tcp.set_read_timeout(Some(timeout))?;
        let tls = url.scheme() == "https";
        let sock = if tls {
            let mut roots =
                RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            if let Some(ca) = api.exchange_ca(endpoint)? {
                roots.add(CertificateDer::from(ca))?;
            }
            if let Some(ca) = env::var_os("GARLIC_CA_CERT") {
                for c in CertificateDer::pem_slice_iter(&std::fs::read(ca)?) {
                    roots.add(c?)?;
                }
            }
            let cfg = ClientConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_safe_default_protocol_versions()?
            .with_root_certificates(roots)
            .with_no_client_auth();
            Transport::Tls(Box::new(StreamOwned::new(
                ClientConnection::new(Arc::new(cfg), ServerName::try_from(host)?)?,
                tcp,
            )))
        } else {
            Transport::Plain(tcp)
        };
        url.set_scheme(if tls { "wss" } else { "ws" })
            .map_err(|_| "invalid exchange scheme")?;
        url.set_path(&format!("/v1/exchange/{}", exchange_id));
        let mut req = url.as_str().into_client_request()?;
        req.headers_mut().insert(
            "x-garlic-client-release",
            HeaderValue::from_static("gex4-r1"),
        );
        req.headers_mut().insert(
            "x-garlic-client-platform",
            HeaderValue::from_static(std::env::consts::OS),
        );
        let mut auth = HeaderValue::from_str(&format!("Bearer {token}"))?;
        auth.set_sensitive(true);
        req.headers_mut().insert(AUTHORIZATION, auth);
        if let Some(device) = device {
            let mut h = HeaderValue::from_str(device)?;
            h.set_sensitive(true);
            req.headers_mut().insert("x-garlic-device-token", h);
        }
        let header = protocol.version().map(|n| HeaderValue::from(u16::from(n)));
        if let Some(version) = &header {
            req.headers_mut().insert("x-garlic-exchange-version", version.clone());
        }
        let (ws, response) =
            tungstenite::client(req, sock).map_err(|e| format!("exchange upgrade failed: {e}"))?;
        if header.is_some() && response.headers().get("x-garlic-exchange-version") != header.as_ref() {
            return Err("server did not confirm selected exchange version".into());
        }
        let ready = Packet {
            sequence: 0,
            acknowledgement: 0,
            flag: active,
            records: vec![],
        }
        .encode()
        .map_err(|_| "invalid exchange startup")?;
        let mut lane = Lane { socket: ws, decoder: Default::default(), message_id: std::num::NonZeroU32::MIN };
        lane.send(protocol,Message::Binary(ready.into()))?;
        lane.socket.get_ref()
            .socket()
            .set_read_timeout(Some(Duration::from_millis(6)))?;
        Ok(lane)
    }
    fn advertise(&mut self) -> Result<(), StateError> {
        let packet = Packet {
            sequence: 0,
            acknowledgement: self.received,
            flag: true,
            records: vec![],
        }
        .encode()
        .map_err(|_| "invalid exchange advertisement")?;
        self.send(Message::Binary(packet.into()))?;
        self.last_poll = Instant::now();
        Ok(())
    }
    fn send_record(&mut self, kind: u8, bytes: &[u8]) -> Result<(), StateError> {
        if self.pending.len() >= 1024 {
            return Err("outgoing exchange window exhausted".into());
        }
        let packet = Packet {
            sequence: self.next,
            acknowledgement: self.received,
            flag: true,
            records: vec![Record {
                owner: self.flow,
                kind,
                payload: bytes,
            }],
        };
        let data = packet
            .encode()
            .map_err(|_| "could not encode exchange record")?;
        trace("send", self.next, data.len());
        self.send(Message::Binary(data.clone().into()))?;
        self.pending.push_back((self.next, data, Instant::now()));
        self.next = self
            .next
            .checked_add(1)
            .ok_or("exchange sequence exhausted")?;
        Ok(())
    }
    fn send_frame(&mut self, content: Content<'_>) -> Result<(), StateError> {
        let bytes = StreamFrame {
            stream_id: self.flow,
            content,
        }
        .encode()
        .map_err(|_| "invalid stream frame")?;
        self.send_record(0, &bytes)
    }
    fn accept(&mut self, data: &[u8], local: &mut TcpStream) -> Result<(), StateError> {
        let packet = if self.protocol.selective() {
            Packet::decode_gex3(data).map_err(|_| "invalid selective acknowledgement")?.0
        } else {
            Packet::decode(data).map_err(|_| "invalid exchange packet")?
        };
        for record in packet.records {
            if record.owner != self.flow {
                return Err("unexpected flow on exchange".into());
            }
            match record.kind {
                0 => {
                    let mut frames = record.payload;
                    while !frames.is_empty() {
                        let (frame, n) =
                            stream::decode(frames).map_err(|_| "invalid stream data")?;
                        frames = &frames[n..];
                        if frame.stream_id != self.flow {
                            return Err("mismatched stream owner".into());
                        }
                        trace(
                            match frame.content {
                                Content::Opened => "opened",
                                Content::Data { .. } => "data",
                                Content::Window { .. } => "window",
                                _ => "other",
                            },
                            self.received,
                            n,
                        );
                        match frame.content {
                            Content::Opened => self.opened = true,
                            Content::Window { offset, credit } => {
                                if offset < self.tx_offset
                                    && self.stream_ack.is_none_or(|old| offset >= old)
                                {
                                    self.stream_ack = Some(offset);
                                    while self
                                        .unconfirmed
                                        .front()
                                        .is_some_and(|(seq, _, _)| *seq <= offset)
                                    {
                                        self.unconfirmed.pop_front();
                                    }
                                    let in_flight: usize = self
                                        .unconfirmed
                                        .iter()
                                        .map(|(_, data, _)| data.len())
                                        .sum();
                                    self.tx_credit =
                                        (credit as usize).min(WINDOW).saturating_sub(in_flight);
                                }
                            }
                            Content::Data { offset, bytes } => {
                                trace("stream-rx", offset, bytes.len());
                                self.receive_data(offset, bytes, local)?;
                                if self.rx_offset > 0 {
                                    self.send_frame(Content::Window {
                                        offset: self.rx_offset - 1,
                                        credit: (WINDOW - self.rx_buffered) as u32,
                                    })?;
                                }
                            }
                            Content::HalfClose(2) => {
                                if !self.peer_closed {
                                    self.peer_closed = true;
                                    if self.connected_response {
                                        local.shutdown(Shutdown::Write)?;
                                    }
                                }
                            }
                            Content::Close => self.ended = true,
                            Content::Reset(_) => return Err("exit reset the TCP stream".into()),
                            _ => {}
                        }
                    }
                }
                1 => self.send_record(2, record.payload)?,
                2 => {}
                3 => self.ended = true,
                _ => return Err("unknown exchange record".into()),
            }
        }
        Ok(())
    }
    fn receive_data(
        &mut self,
        offset: u64,
        bytes: &[u8],
        local: &mut TcpStream,
    ) -> Result<(), StateError> {
        if offset < self.rx_offset {
            return Ok(());
        }
        if offset > self.rx_offset {
            if let Some(seen) = self.rx_gaps.get(&offset) {
                if seen != bytes {
                    return Err("conflicting stream retransmission".into());
                }
            } else {
                if self.rx_buffered + bytes.len() > WINDOW || self.rx_gaps.len() >= 4096 {
                    return Err("incoming stream window exceeded".into());
                }
                self.rx_gaps.insert(offset, bytes.to_vec());
                self.rx_buffered += bytes.len();
            }
            return Ok(());
        }
        self.deliver(bytes, local)?;
        while let Some(buffered) = self.rx_gaps.remove(&self.rx_offset) {
            self.rx_buffered -= buffered.len();
            self.deliver(&buffered, local)?;
        }
        Ok(())
    }
    fn deliver(&mut self, bytes: &[u8], local: &mut TcpStream) -> Result<(), StateError> {
        if self.connected_response {
            local.write_all(bytes)?;
            self.traffic
                .rx
                .fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
        } else if self.early.len() + bytes.len() <= WINDOW {
            self.early.extend_from_slice(bytes);
        } else {
            return Err("destination sent too much data during connection setup".into());
        }
        self.rx_offset = self
            .rx_offset
            .checked_add(1)
            .ok_or("stream sequence overflow")?;
        Ok(())
    }
    fn poll(&mut self, local: &mut TcpStream) -> Result<bool, StateError> {
        if self.last_poll.elapsed() >= Duration::from_millis(250) {
            self.advertise()?;
        }
        let mut received = false;
        for index in (0..self.lanes.len()).rev() {
            let Some(lane) = self.lanes.get_mut(index) else {
                continue;
            };
            let msg = match lane.socket.read() {
                Ok(message) => message,
                Err(tungstenite::Error::Io(e)) if timed(&e) => continue,
                Err(e) if self.lanes.len() == 1 => return Err(e.into()),
                Err(_) => {
                    self.lanes.remove(index);
                    continue;
                }
            };
            if matches!(msg, Message::Close(_)) && self.lanes.len() > 1 {
                self.lanes.remove(index);
                continue;
            }
            received = true;
            self.handle_message(index, msg, local)?;
        }
        Ok(received)
    }
    fn handle_message(
        &mut self,
        index: usize,
        msg: Message,
        local: &mut TcpStream,
    ) -> Result<(), StateError> {
        match msg {
            Message::Binary(data) => {
                let assembled;
                let data = if self.protocol.tlv() {
                    gex4::unwrap(&data).map_err(|_| "invalid TLV header")?
                } else if self.protocol == ClientProtocol::Gex1 {
                    data.as_ref()
                } else {
                    let Some(packet) = self.lanes[index].decoder.receive(&data)
                        .map_err(|_| "invalid exchange fragment")? else {return Ok(());};
                    assembled = packet;
                    assembled.as_slice()
                };
                let p = if self.protocol.selective() {
                    Packet::decode_gex3(data).map_err(|_| "invalid selective acknowledgement")?.0
                } else { Packet::decode(data).map_err(|_| "invalid exchange packet")? };
                trace("recv", p.sequence, data.len());
                if p.acknowledgement >= self.next {
                    return Err("invalid exchange acknowledgement".into());
                }
                while self
                    .pending
                    .front()
                    .is_some_and(|v| v.0 <= p.acknowledgement)
                {
                    self.pending.pop_front();
                }
                let seq = p.sequence;
                let ack = p.flag || seq != 0;
                if seq > self.received {
                    if seq - self.received > 64
                        || (seq > self.received + 1
                            && self.gaps.values().map(Vec::len).sum::<usize>() + data.len()
                                > 8 << 20)
                    {
                        return Err("incoming exchange window exceeded".into());
                    }
                    self.gaps.entry(seq).or_insert_with(|| data.to_vec());
                    while let Some(bytes) = self
                        .gaps
                        .remove(&self.received.checked_add(1).ok_or("sequence exhausted")?)
                    {
                        self.accept(&bytes, local)?;
                        self.received += 1;
                    }
                }
                if ack {
                    let bytes = Packet {
                        sequence: 0,
                        acknowledgement: self.received,
                        flag: false,
                        records: vec![],
                    }
                    .encode()
                    .map_err(|_| "invalid ack")?;
                    self.send(Message::Binary(bytes.into()))?;
                }
            }
            Message::Ping(_) => {
                if let Err(e) = self.lanes[index].socket.flush() {
                    if self.lanes.len() == 1 {
                        return Err(e.into());
                    }
                    self.lanes.remove(index);
                }
            }
            Message::Pong(_) => {}
            Message::Close(_) => return Err(tungstenite::Error::ConnectionClosed.into()),
            _ => return Err("unexpected exchange message".into()),
        }
        Ok(())
    }
    pub fn open(&mut self, local: &mut TcpStream, timeout: Duration) -> Result<(), StateError> {
        self.send_frame(Content::Open)?;
        let end = Instant::now() + timeout;
        let mut last_open = Instant::now();
        while !self.opened {
            if self.ended { return Err("destination closed during setup".into()); }
            if Instant::now() >= end {
                return Err("destination open timed out".into());
            }
            self.poll(local)?;
            if !self.opened && last_open.elapsed() >= Duration::from_millis(500) {
                self.send_frame(Content::Open)?;
                last_open = Instant::now();
            }
        }
        Ok(())
    }
    pub fn relay(
        &mut self,
        local: &mut TcpStream,
        stop: &std::sync::atomic::AtomicBool,
    ) -> Result<(), StateError> {
        self.connected_response = true;
        local.write_all(&self.early)?;
        self.traffic.rx.fetch_add(
            self.early.len() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.early.clear();
        if self.peer_closed {
            local.shutdown(Shutdown::Write)?;
        }
        local.set_read_timeout(Some(Duration::from_millis(10)))?;
        local.set_write_timeout(Some(Duration::from_secs(20)))?;
        let mut eof = false;
        let mut buf = [0; 16384];
        let mut ping = Instant::now();
        while !self.ended
            && !(eof && self.peer_closed)
            && !stop.load(std::sync::atomic::Ordering::Acquire)
        {
            // Service queued upload without waiting on an idle loopback socket between download bursts.
            for _ in 0..if cfg!(windows) { 2 } else { 1 } {
                let n = self.tx_credit.min(buf.len()).min(self.record_limit);
                if eof || n == 0 || self.pending.len() >= 256 { break; }
                #[cfg(windows)]
                if !upload_ready(local)? { break; }
                match local.read(&mut buf[..n]) {
                    Ok(0) => {
                        eof = true;
                        self.send_frame(Content::HalfClose(1))?;
                    }
                    Ok(n) => {
                        self.send_frame(Content::Data {
                            offset: self.tx_offset,
                            bytes: &buf[..n],
                        })?;
                        self.traffic
                            .tx
                            .fetch_add(n as u64, std::sync::atomic::Ordering::Relaxed);
                        self.unconfirmed.push_back((
                            self.tx_offset,
                            buf[..n].to_vec(),
                            Instant::now(),
                        ));
                        self.tx_offset = self
                            .tx_offset
                            .checked_add(1)
                            .ok_or("stream sequence overflow")?;
                        self.tx_credit -= n;
                    }
                    Err(e) if timed(&e) => {}
                    Err(e) => return Err(e.into()),
                }
            }
            // Drain incoming frames in bounded bursts before waiting for local upload.
            // Fragmented downloads otherwise pay one idle read timeout per fragment.
            for _ in 0..16 {
                if !self.poll(local)? { break; }
                if self.ended || self.peer_closed { break; }
            }
            if let Some((_, bytes, sent)) = self
                .pending
                .front_mut()
                .filter(|(_, _, sent)| sent.elapsed() >= Duration::from_secs(1))
            {
                let retransmission = Message::Binary(bytes.clone().into());
                *sent = Instant::now();
                self.send(retransmission)?;
            }
            if let Some((seq, data, sent)) = self.unconfirmed.front_mut() {
                if sent.elapsed() >= Duration::from_secs(1) && self.pending.len() < 256 {
                    *sent = Instant::now();
                    let seq = *seq;
                    let bytes = data.clone();
                    self.send_frame(Content::Data {
                        offset: seq,
                        bytes: &bytes,
                    })?;
                }
            }
            if ping.elapsed() >= Duration::from_secs(10) {
                self.send(Message::Ping(Vec::new().into()))?;
                ping = Instant::now();
            }
        }
        for lane in &mut self.lanes {
            let _ = lane.socket.close(None);
        }
        Ok(())
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::ClientProtocol;
    use serde_json::json;

    #[test]
    fn requires_an_explicit_shared_capability() {
        let legacy = json!({"exchange_version":1,"exchange_versions":[1,2,3]});
        assert_eq!(
            ClientProtocol::negotiate(&legacy).unwrap(),
            ClientProtocol::Gex3
        );
        let future = json!({"exchange_extensions":["unknown", "gex4-tlv"]});
        assert_eq!(
            ClientProtocol::negotiate(&future).unwrap(),
            ClientProtocol::Gex4
        );
        assert!(ClientProtocol::negotiate(&json!({"exchange_versions":[4]})).is_err());
        assert_eq!(ClientProtocol::select(&legacy,Some("2")).unwrap(),ClientProtocol::Gex2);
        assert!(ClientProtocol::select(&legacy,Some("")).is_err());
    }
}
