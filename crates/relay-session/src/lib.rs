//! Authenticated startup of a worker channel on the Garlic coordinator.
use garlic_relay_data::control::{Authority, Hello, Message as Control};
use garlic_relay_data::{envelope, Direction, Frame};
use rustls::pki_types::{pem::PemObject, CertificateDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::error::Error;
use std::io;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tungstenite::client::IntoClientRequest;
use tungstenite::http::{header::AUTHORIZATION, HeaderValue, Uri};
use tungstenite::{Message, WebSocket};

pub mod auth;
pub mod coordinator;
pub mod destination;
pub mod endpoint;
pub mod exit;
pub mod identity;
pub mod runner;

pub type WorkerConnection = RelayConnection<StreamOwned<ClientConnection, TcpStream>>;
pub type SessionError = Box<dyn Error + Send + Sync>;

pub struct RelayConnection<S> {
    socket: WebSocket<S>,
    timeouts: TcpStream,
    decoder: envelope::Decoder,
    next_id: NonZeroU32,
}

#[derive(Debug)]
pub enum Inbound {
    Control(Control),
    Data {
        direction: Direction,
        flow_id: [u8; 16],
        bytes: Vec<u8>,
    },
    Closed,
}

impl<S: io::Read + io::Write> RelayConnection<S> {
    fn new(socket: WebSocket<S>, timeouts: TcpStream) -> Self {
        Self {
            socket,
            timeouts,
            decoder: envelope::Decoder::default(),
            next_id: NonZeroU32::MIN,
        }
    }

    fn send_logical(&mut self, kind: u8, body: &[u8]) -> Result<(), SessionError> {
        let mut payload = Vec::with_capacity(body.len() + 1);
        payload.push(kind);
        payload.extend_from_slice(body);
        let id = self.next_id;
        self.next_id = NonZeroU32::new(id.get().wrapping_add(1)).unwrap_or(NonZeroU32::MIN);
        let records =
            envelope::fragments(id, &payload).map_err(|_| "relay message exceeds record limit")?;
        for record in records {
            self.socket.write(Message::Binary(record.into()))?;
        }
        self.socket.flush()?;
        Ok(())
    }

    pub fn send_control(&mut self, message: &Control) -> Result<(), SessionError> {
        self.send_logical(1, &serde_json::to_vec(message)?)
    }

    pub fn send_data(&mut self, frame: &Frame<'_>) -> Result<(), SessionError> {
        self.send_logical(0, &frame.encode().map_err(|_| "invalid relay data frame")?)
    }

    pub fn read(&mut self, timeout: Duration) -> Result<Inbound, SessionError> {
        let started = Instant::now();
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Err(
                    io::Error::new(io::ErrorKind::TimedOut, "relay receive timed out").into(),
                );
            }
            self.timeouts.set_read_timeout(Some(remaining))?;
            let record = match self.socket.read()? {
                Message::Binary(record) => record,
                Message::Ping(_) => {
                    self.socket.flush()?;
                    continue;
                }
                Message::Pong(_) => continue,
                Message::Close(_) => return Ok(Inbound::Closed),
                _ => {
                    return Err("unexpected WebSocket message after exchange-v2 negotiation".into())
                }
            };
            let Some(message) = self
                .decoder
                .receive(&record)
                .map_err(|_| "invalid relay record")?
            else {
                continue;
            };
            let (&kind, body) = message.split_first().ok_or("empty relay message")?;
            match kind {
                0 => {
                    let frame =
                        garlic_relay_data::decode(body).map_err(|_| "invalid relay data frame")?;
                    return Ok(Inbound::Data {
                        direction: frame.direction,
                        flow_id: frame.flow_id,
                        bytes: frame.payload.to_vec(),
                    });
                }
                1 => return Ok(Inbound::Control(serde_json::from_slice(body)?)),
                2 => self.send_logical(3, body)?,
                3 => continue,
                4 => return Ok(Inbound::Closed),
                _ => return Err("unknown logical relay message".into()),
            }
        }
    }

    pub fn close(mut self) -> Result<(), SessionError> {
        self.send_logical(4, &[])?;
        self.socket.close(None)?;
        Ok(())
    }
}

/// A connection is established only after the coordinator confirms the worker
/// identifier, control version and authority obtained during authentication.
pub fn connect(
    coordinator: &Uri,
    ca_pem: &[u8],
    node_token: &str,
    authority: &Authority,
    hello: &Hello,
    timeout: Duration,
) -> Result<WorkerConnection, SessionError> {
    connect_at(
        coordinator,
        None,
        ca_pem,
        node_token,
        authority,
        hello,
        timeout,
    )
}

fn connect_at(
    coordinator: &Uri,
    peer: Option<SocketAddr>,
    ca_pem: &[u8],
    node_token: &str,
    authority: &Authority,
    hello: &Hello,
    timeout: Duration,
) -> Result<WorkerConnection, SessionError> {
    if coordinator.scheme_str() != Some("https")
        || coordinator.query().is_some()
        || !matches!(coordinator.path(), "" | "/")
    {
        return Err("expected an HTTPS coordinator origin".into());
    }
    let address = coordinator
        .authority()
        .ok_or("missing coordinator authority")?;
    if address.as_str().contains('@') {
        return Err("credentials must be supplied separately from the coordinator URL".into());
    }
    let host = address.host().trim_start_matches('[').trim_end_matches(']');
    let name = ServerName::try_from(host.to_owned())?;
    let mut roots = RootCertStore::empty();
    for cert in CertificateDer::pem_slice_iter(ca_pem) {
        roots.add(cert?)?;
    }
    if roots.is_empty() {
        return Err("empty coordinator trust store".into());
    }
    let tls =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()?
            .with_root_certificates(roots)
            .with_no_client_auth();
    let mut last_error = None;
    let peers = match peer {
        Some(peer) => vec![peer],
        None => (host, address.port_u16().unwrap_or(443))
            .to_socket_addrs()?
            .collect(),
    };
    let stream = peers
        .into_iter()
        .find_map(|addr| match TcpStream::connect_timeout(&addr, timeout) {
            Ok(stream) => Some(stream),
            Err(err) => {
                last_error = Some(err);
                None
            }
        })
        .ok_or_else(|| last_error.unwrap_or_else(|| io::Error::other("no coordinator address")))?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let stream = StreamOwned::new(ClientConnection::new(Arc::new(tls), name)?, stream);
    let mut req = format!("wss://{address}/v1/infinity/relay").into_client_request()?;
    let mut bearer = HeaderValue::from_str(&format!("Bearer {node_token}"))?;
    bearer.set_sensitive(true);
    req.headers_mut().insert(AUTHORIZATION, bearer);
    req.headers_mut()
        .insert("x-garlic-relay-drain-ack", HeaderValue::from_static("1"));
    req.headers_mut()
        .insert("x-garlic-exchange-version", HeaderValue::from_static("2"));
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|_| io::Error::other("unable to generate a relay seed"))?;
    let seed: String = seed.iter().map(|byte| format!("{byte:02x}")).collect();
    req.headers_mut()
        .insert("x-garlic-shape-seed", HeaderValue::from_str(&seed)?);
    req.headers_mut().insert(
        "x-garlic-shape-profile",
        HeaderValue::from_static("agent-tls-2026-replay-r1"),
    );
    let (mut socket, upgrade) = tungstenite::client(req, stream)?;
    let agreed = upgrade.headers();
    if agreed
        .get("x-garlic-exchange-version")
        .and_then(|v| v.to_str().ok())
        != Some("2")
        || agreed
            .get("x-garlic-owner-id")
            .and_then(|v| v.to_str().ok())
            != Some(authority.owner_id.to_string().as_str())
        || agreed
            .get("x-garlic-owner-epoch")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            != Some(authority.epoch)
    {
        return Err("relay upgrade did not confirm the negotiated version and authority".into());
    }
    socket.send(Message::Text(serde_json::to_string(hello)?.into()))?;
    loop {
        match socket.read()? {
            Message::Text(data) => {
                let reply: serde_json::Value = serde_json::from_str(&data)?;
                if reply.get("type").and_then(|v| v.as_str()) == Some("rejected") {
                    let reason = reply
                        .get("reason")
                        .or_else(|| reply.get("error"))
                        .and_then(|v| {
                            v.as_str()
                                .or_else(|| v.get("code").and_then(|code| code.as_str()))
                        })
                        .unwrap_or("unspecified reason");
                    return Err(format!("coordinator rejected worker hello: {reason}").into());
                }
                let Control::Ready {
                    protocol_version,
                    worker_id,
                    authority: peer,
                } = serde_json::from_value(reply)?
                else {
                    return Err("expected relay Ready".into());
                };
                if protocol_version != hello.protocol_version
                    || worker_id != hello.worker_id
                    || &peer != authority
                {
                    return Err("coordinator Ready disagrees with the authenticated session".into());
                }
                let timeouts = socket.get_ref().get_ref().try_clone()?;
                return Ok(RelayConnection::new(socket, timeouts));
            }
            Message::Ping(_) => socket.flush()?,
            Message::Pong(_) => {}
            _ => return Err("relay closed or sent unexpected data during startup".into()),
        }
    }
}
