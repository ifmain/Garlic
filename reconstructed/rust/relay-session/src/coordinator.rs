//! Receive authenticated exit workers on the coordinator side of Relay v2.
use crate::{RelayConnection, SessionError};
use garlic_relay_data::control::{
    Authority, Capability, ErrorCode, Hello, Message as Control, ProxyError,
};
use rustls::{ServerConnection, StreamOwned};
use std::net::TcpStream;
use std::time::Duration;
use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::http::{header::AUTHORIZATION, HeaderValue, StatusCode};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::Message;

pub type CoordinatorConnection = RelayConnection<StreamOwned<ServerConnection, TcpStream>>;

/// Produced by the application's credential verifier from a live node session.
pub struct NodeSession {
    pub node_id: String,
    pub authority: Authority,
}

fn reject(status: StatusCode) -> ErrorResponse {
    let mut response = ErrorResponse::new(None);
    *response.status_mut() = status;
    response
}

/// Authenticate the upgrade and require the announced node to match its bearer.
/// The caller provides its existing account and session validation.
pub fn accept(
    tls: StreamOwned<ServerConnection, TcpStream>,
    verify: impl FnOnce(&str) -> Result<NodeSession, SessionError>,
    timeout: Duration,
) -> Result<(Hello, CoordinatorConnection), SessionError> {
    let timeouts = tls.get_ref().try_clone()?;
    timeouts.set_read_timeout(Some(timeout))?;
    timeouts.set_write_timeout(Some(timeout))?;
    let mut session = None;
    let check = |req: &Request, mut response: Response| {
        if req.uri().path_and_query().map(|p| p.as_str()) != Some("/v1/infinity/relay") {
            return Err(reject(StatusCode::NOT_FOUND));
        }
        let headers = req.headers();
        let token = headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .filter(|s| !s.is_empty())
            .ok_or_else(|| reject(StatusCode::UNAUTHORIZED))?;
        let valid = verify(token).map_err(|_| reject(StatusCode::UNAUTHORIZED))?;
        if headers
            .get("x-garlic-exchange-version")
            .and_then(|v| v.to_str().ok())
            != Some("2")
        {
            return Err(reject(StatusCode::BAD_REQUEST));
        }
        let out = response.headers_mut();
        out.insert("x-garlic-exchange-version", HeaderValue::from_static("2"));
        if headers
            .get("x-garlic-relay-drain-ack")
            .and_then(|v| v.to_str().ok())
            == Some("1")
        {
            out.insert("x-garlic-relay-drain-ack", HeaderValue::from_static("1"));
        }
        out.insert(
            "x-garlic-owner-id",
            valid
                .authority
                .owner_id
                .to_string()
                .parse()
                .map_err(|_| reject(StatusCode::INTERNAL_SERVER_ERROR))?,
        );
        out.insert("x-garlic-owner-epoch", valid.authority.epoch.into());
        session = Some(valid);
        Ok(response)
    };
    let config = WebSocketConfig::default()
        .max_message_size(Some(65536))
        .max_frame_size(Some(65536));
    let mut socket = match tungstenite::accept_hdr_with_config(tls, check, Some(config)) {
        Ok(socket) => socket,
        Err(tungstenite::HandshakeError::Failure(err)) => return Err(err.into()),
        Err(tungstenite::HandshakeError::Interrupted(_)) => {
            return Err("worker upgrade interrupted".into())
        }
    };
    let verified = session.ok_or("missing node session")?;
    let hello: Hello = loop {
        match socket.read()? {
            Message::Text(data) => break serde_json::from_str(&data)?,
            Message::Ping(_) => socket.flush()?,
            Message::Pong(_) => {}
            _ => return Err("expected a worker hello".into()),
        }
    };
    if let Some(code) = handshake_error(&hello, &verified.node_id) {
        let error = ProxyError::from(code);
        let reply = serde_json::json!({"type":"rejected","protocol_version":1,"error":error});
        socket.send(Message::Text(serde_json::to_string(&reply)?.into()))?;
        return Err(error.message.into());
    }
    let ready = Control::Ready {
        protocol_version: 1,
        worker_id: hello.worker_id,
        authority: verified.authority,
    };
    socket.send(Message::Text(serde_json::to_string(&ready)?.into()))?;
    Ok((hello, RelayConnection::new(socket, timeouts)))
}

pub fn handshake_error(hello: &Hello, node_id: &str) -> Option<ErrorCode> {
    if hello.protocol_version != 1 {
        return Some(ErrorCode::IncompatibleDataVersion);
    }
    if hello.node_id != node_id {
        return Some(ErrorCode::Forbidden);
    }
    if hello.worker_id.is_nil() || !(1..=1024).contains(&hello.max_streams) {
        return Some(ErrorCode::InvalidRequest);
    }
    if !Capability::ALL
        .iter()
        .all(|cap| hello.capabilities.contains(cap))
    {
        return Some(ErrorCode::UnsupportedCapability);
    }
    None
}
