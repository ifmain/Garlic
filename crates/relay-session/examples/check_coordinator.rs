//! Reverse compatibility fixture. A deployed exit connects to this loopback TLS listener.
use garlic_relay_data::control::{Authority, Capability, Message, Route, StreamOpen, UserSnapshot};
use garlic_relay_data::stream::{self, Content, StreamFrame};
use garlic_relay_data::{Direction, Frame};
use garlic_relay_session::coordinator::{accept, CoordinatorConnection};
use garlic_relay_session::{auth::verify_node, Inbound, SessionError};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TIMEOUT: Duration = Duration::from_secs(15);

fn now() -> Result<u64, SessionError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

fn send(
    relay: &mut CoordinatorConnection,
    flow: [u8; 16],
    content: Content<'_>,
) -> Result<(), SessionError> {
    let bytes = StreamFrame {
        stream_id: flow,
        content,
    }
    .encode()
    .map_err(|_| "invalid fixture frame")?;
    relay.send_data(&Frame {
        direction: Direction::One,
        flow_id: flow,
        payload: &bytes,
    })
}

fn run(dir: &Path) -> Result<(), SessionError> {
    let certs = CertificateDer::pem_slice_iter(&fs::read(dir.join("cert.pem"))?)
        .collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(dir.join("key.pem"))?;
    let config =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()?
            .with_no_client_auth()
            .with_single_cert(certs, key)?;
    let listener = TcpListener::bind(("127.0.0.1", 18447))?;
    println!("waiting for the original exit on the local TLS fixture");
    let (stream, _) = listener.accept()?;
    let authority: Authority =
        serde_json::from_slice(&fs::read(dir.join("d/state.authority.json"))?)?;
    let tls = StreamOwned::new(ServerConnection::new(Arc::new(config))?, stream);
    let (hello, mut relay) = accept(
        tls,
        |token| {
            verify_node(
                &dir.join("d/users.json"),
                &dir.join("keys.json"),
                &authority,
                token,
                now()?,
            )
        },
        TIMEOUT,
    )?;
    println!(
        "authenticated original worker with {} stream slots",
        hello.max_streams
    );
    let snapshot: UserSnapshot = serde_json::from_slice(&fs::read(dir.join("snapshot.json"))?)?;
    let user_id = snapshot
        .users
        .iter()
        .find(|u| u.enabled && u.username == "sample")
        .ok_or("missing fixture account")?
        .id;
    relay.send_control(&Message::UserSync { snapshot })?;
    let flow_id = "21ad11e9-c192-403b-ad76-49b092ee73ba".parse()?;
    let request_id = "b143ebc1-c2d0-4ed7-abd4-21aa711b5e44".parse()?;
    let open = StreamOpen {
        control_version: 1,
        data_version: 1,
        node_id: hello.node_id,
        channel_id: "7039cf00-64ab-4347-8e32-8b45079ae33f".parse()?,
        user_id,
        flow_id,
        stream_id: "977d9183-456b-4910-a150-acc1382cc028".parse()?,
        connect_authority: "127.0.0.1:443".into(),
        preferred_exit: None,
        route: Route::ThroughRu,
        capabilities: vec![
            Capability::Duplex,
            Capability::HalfClose,
            Capability::Window,
            Capability::Reset,
            Capability::Resume,
            Capability::StructuredErrors,
        ],
        window_size: 1048576,
        deadline_unix_ms: now()? * 1000 + 30000,
        authority,
    };
    relay.send_control(&Message::Open {
        request_id,
        open,
        admin: false,
    })?;
    match relay.read(TIMEOUT)? {
        Inbound::Control(Message::Opened {
            request_id: id,
            error: None,
        }) if id == request_id => {}
        other => {
            return Err(format!("original worker declined the fixture open: {other:?}").into())
        }
    }
    let flow = *flow_id.as_bytes();
    send(&mut relay, flow, Content::Open)?;
    let payload = b"reverse mixed relay check";
    let mut echo = Vec::new();
    let mut accepted = false;
    loop {
        let Inbound::Data {
            direction: Direction::Two,
            flow_id: id,
            bytes,
        } = relay.read(TIMEOUT)?
        else {
            return Err("unexpected non-data response from original worker".into());
        };
        if id != flow {
            return Err("unexpected flow".into());
        }
        let (frame, n) = stream::decode(&bytes).map_err(|_| "invalid original worker frame")?;
        if n != bytes.len() || frame.stream_id != flow {
            return Err("incorrect frame identity".into());
        }
        match frame.content {
            Content::Opened if !accepted => {
                accepted = true;
                send(
                    &mut relay,
                    flow,
                    Content::Data {
                        offset: 0,
                        bytes: payload,
                    },
                )?;
            }
            Content::Window { .. } => {}
            Content::Data { offset, bytes } if accepted => {
                echo.extend_from_slice(bytes);
                if !payload.starts_with(&echo) {
                    return Err("echo differs".into());
                }
                send(
                    &mut relay,
                    flow,
                    Content::Window {
                        offset,
                        credit: 1048576,
                    },
                )?;
                if echo == payload {
                    send(&mut relay, flow, Content::HalfClose(1))?;
                }
            }
            Content::Close if echo == payload => break,
            other => return Err(format!("unexpected original worker response: {other:?}").into()),
        }
    }
    println!("original exit confirmed reverse echo and half-close");
    relay.send_control(&Message::CloseFlow { flow_id })?;
    relay.close()
}

fn main() -> Result<(), SessionError> {
    let dir = std::env::args_os()
        .nth(1)
        .ok_or("usage: check_coordinator SYNTHETIC_LAB_DIR")?;
    run(Path::new(&dir))
}
