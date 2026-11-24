//! Test both stages of node authentication with a provisioned fixture identity.
use garlic_relay_data::control::{Capability, Hello, Message};
use garlic_relay_session::{connect, identity, Inbound, SessionError};
use std::{fs, time::Duration};

fn main() -> Result<(), SessionError> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [origin, ca, id, ring, mode @ ..] = args.as_slice() else {
        return Err(
            "usage: check_identity HTTPS_ORIGIN CA_PEM IDENTITY_JSON KEYS_JSON [legacy]".into(),
        );
    };
    let legacy = match mode {
        [] => false,
        [operation] if operation == "legacy" => {
            if reqwest::Url::parse(origin)?.host_str() != Some("127.0.0.1") {
                return Err("use a loopback legacy panel fixture".into());
            }
            true
        }
        _ => return Err("unrecognized check_identity mode".into()),
    };
    let ca = fs::read(ca)?;
    let identity = identity::Identity::load(&serde_json::from_slice(&fs::read(id)?)?)?;
    let keys = serde_json::from_slice(&fs::read(ring)?)?;
    let timeout = Duration::from_secs(15);
    let grant = identity::login(origin, &ca, &identity, &keys, "lab-fixture", timeout)?;
    if legacy {
        let mut builder = reqwest::blocking::Client::builder()
            .https_only(true)
            .no_proxy()
            .tls_built_in_root_certs(false)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout);
        for certificate in reqwest::Certificate::from_pem_bundle(&ca)? {
            builder = builder.add_root_certificate(certificate);
        }
        let response = builder
            .build()?
            .get(reqwest::Url::parse(origin)?.join("v1/infra/legacy")?)
            .bearer_auth(&grant.token)
            .send()?;
        println!("Legacy worker API status {}", response.status().as_u16());
        if response.status().is_success() {
            let document: serde_json::Value = response.json()?;
            if let Some(object) = document.as_object() {
                println!(
                    "Legacy worker response fields: {:?}",
                    object.keys().collect::<Vec<_>>()
                );
            }
        }
        return Ok(());
    }
    println!("original authority verified the Rust membership proof and node signature");
    let hello = Hello {
        protocol_version: 1,
        worker_id: uuid::Uuid::new_v4(),
        node_id: identity.node_id,
        capabilities: vec![
            Capability::Duplex,
            Capability::Window,
            Capability::HalfClose,
            Capability::Reset,
            Capability::Resume,
            Capability::StructuredErrors,
        ],
        max_streams: 32,
    };
    let mut relay = connect(
        &origin.parse()?,
        &ca,
        &grant.token,
        &grant.authority,
        &hello,
        timeout,
    )?;
    if !matches!(
        relay.read(timeout)?,
        Inbound::Control(Message::UserSync { .. })
    ) {
        return Err("expected initial user snapshot".into());
    }
    println!("fresh node grant accepted for original exchange-v2 relay");
    relay.send_control(&Message::Drain)?;
    match relay.read(timeout)? {
        Inbound::Control(Message::Drained) => {
            println!("original coordinator acknowledged worker drain")
        }
        other => return Err(format!("expected Drained, got {other:?}").into()),
    }
    relay.close()
}
