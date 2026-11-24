//! Integration fixture for a local isolated coordinator and a fresh test identity.
use garlic_relay_data::control::{Authority, Capability, Hello};
use garlic_relay_session::{connect, Inbound, SessionError};
use serde_json::Value;
use std::{fs, time::Duration};

fn main() -> Result<(), SessionError> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (worker_mode, args) = match args.first().map(String::as_str) {
        Some("--serve-loopback-fixture") => (true, &args[1..]),
        _ => (false, args.as_slice()),
    };
    let [origin, trust_file, session_file] = args else {
        return Err("usage: check_worker [--serve-loopback-fixture] HTTPS_ORIGIN CA_PEM TEST_NODE_SESSION_JSON".into());
    };
    let session: Value = serde_json::from_slice(&fs::read(session_file)?)?;
    let node = session["principal"]["node_id"]
        .as_str()
        .ok_or("missing node id")?;
    let token = session["token"].as_str().ok_or("missing node session")?;
    let authority: Authority = serde_json::from_value(session["authority"].clone())?;
    let hello = Hello {
        protocol_version: 1,
        worker_id: uuid::Uuid::new_v4(),
        node_id: node.to_owned(),
        capabilities: vec![
            Capability::Duplex,
            Capability::HalfClose,
            Capability::Window,
            Capability::Reset,
            Capability::Resume,
            Capability::StructuredErrors,
        ],
        max_streams: 64,
    };
    let mut socket = connect(
        &origin.parse()?,
        &fs::read(trust_file)?,
        token,
        &authority,
        &hello,
        Duration::from_secs(10),
    )?;
    println!("original coordinator returned Ready for the fixture worker and authority");
    if worker_mode {
        let target = std::net::SocketAddr::from(([127, 0, 0, 1], 443));
        let mut worker = garlic_relay_session::exit::Worker::new(&authority, 64, |open| {
            (open.connect_authority == "127.0.0.1:443").then_some(target)
        });
        worker.run(&mut socket)?;
        return Ok(());
    }
    match socket.read(Duration::from_secs(12))? {
        Inbound::Control(garlic_relay_data::control::Message::UserSync { snapshot }) => {
            println!(
                "received exchange-v2 UserSync at revision {}",
                snapshot.revision
            );
        }
        _ => return Err("expected the initial coordinator account snapshot".into()),
    }
    use garlic_relay_data::control::Message;
    socket.send_control(&Message::Drain)?;
    match socket.read(Duration::from_secs(12))? {
        Inbound::Control(Message::Drained) => {
            println!("original coordinator acknowledged the worker drain")
        }
        other => {
            return Err(format!(
                "expected original coordinator drain acknowledgement, got {other:?}"
            )
            .into())
        }
    }
    socket.close()?;
    Ok(())
}
