use axum::http::Request;
use bytes::Bytes;
use sha2::{Digest, Sha224};
use std::{env, error::Error, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};
type Result<T = ()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
const TARGET: &[u8] = b"echo.garlic-fixture.test";
const PAYLOAD: &[u8] = b"garlic legacy local echo";
const VALID: &[u8] = b"fixture-grpc-password";

async fn sock_server(port: u16) -> Result {
    let socket = TcpListener::bind(("127.0.0.1", port)).await?;
    println!("SOCKS fixture port {port}");
    loop {
        let (mut session, _) = socket.accept().await?;
        tokio::spawn(async move {
            let reply: Result = async {
                let mut greeting = [0; 3];
                session.read_exact(&mut greeting).await?;
                assert_eq!(greeting, [5, 1, 0]);
                session.write_all(&[5, 0]).await?;
                let mut head = [0; 4];
                session.read_exact(&mut head).await?;
                let ip = match head[3] {
                    1 => 4,
                    3 => usize::from(session.read_u8().await?),
                    4 => 16,
                    _ => return Err("unsupported fixture address".into()),
                };
                let mut target = vec![0; ip];
                session.read_exact(&mut target).await?;
                let endpoint = session.read_u16().await?;
                assert_eq!(head[..3], [5, 1, 0]);
                if head[3] == 3 {
                    assert_eq!(target, TARGET);
                }
                assert_eq!(endpoint, 18099);
                println!("SOCKS5 CONNECT port {port} address type {}", head[3]);
                session.write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 0, 0]).await?;
                let mut buffer = [0; 1024];
                loop {
                    let n = session.read(&mut buffer).await?;
                    if n == 0 {
                        break;
                    }
                    session.write_all(&buffer[..n]).await?;
                }
                session.shutdown().await?;
                Ok(())
            }
            .await;
            if let Err(e) = reply {
                eprintln!("SOCKS peer ended: {e}");
            }
        });
    }
}

async fn vless(port: u16, auth: bool, addr: u8, command: u8) -> Result {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    let mut request = vec![0];
    request.extend(
        [if auth && port == 18831 {
            0x11
        } else if auth {
            0x33
        } else {
            0x22
        }; 16],
    );
    // The test ID is represented in compact form as bytes in the handshake.
    request.extend([0, command]);
    request.extend(18099_u16.to_be_bytes());
    request.push(addr);
    match addr {
        1 => request.extend([127, 0, 0, 1]),
        2 => {
            request.push(TARGET.len() as u8);
            request.extend(TARGET);
        }
        3 => request.extend([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        _ => unreachable!(),
    }
    request.extend(PAYLOAD);
    stream.write_all(&request).await?;
    if command == 1 {
        let mut head = [0; 2];
        match timeout(Duration::from_secs(4), stream.read_exact(&mut head)).await {
            Ok(Ok(_)) => {
                let mut echo = vec![0; PAYLOAD.len()];
                stream.read_exact(&mut echo).await?;
                println!(
                    "VLESS {port} auth={auth} addr={addr} response={head:?} echo={}",
                    echo == PAYLOAD
                );
                stream.shutdown().await?;
                let eof = stream.read_u8().await.is_err();
                println!("VLESS {port} EOF={eof}");
            }
            _ => println!("VLESS {port} auth={auth} addr={addr} rejected"),
        }
    } else {
        println!(
            "VLESS command {command} {:?}",
            timeout(Duration::from_secs(4), stream.read_u8()).await
        );
    }
    Ok(())
}

fn hunk(data: &[u8]) -> Bytes {
    let mut frame = vec![0; 5];
    frame.push(10);
    let mut n = data.len();
    while n >= 128 {
        frame.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    frame.push(n as u8);
    frame.extend(data);
    let size = (frame.len() - 5) as u32;
    frame[1..5].copy_from_slice(&size.to_be_bytes());
    frame.into()
}

fn decode(mut body: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    while !body.is_empty() {
        if body.len() < 5 || body[0] != 0 {
            return Err("invalid gRPC frame".into());
        }
        let n = u32::from_be_bytes(body[1..5].try_into()?) as usize;
        let message = body.get(5..5 + n).ok_or("short gRPC frame")?;
        body = &body[5 + n..];
        if message.first() != Some(&10) {
            return Err("invalid protobuf Hunk".into());
        }
        let mut size = 0;
        let mut found = false;
        for (i, value) in message[1..].iter().copied().take(4).enumerate() {
            size |= usize::from(value & 127) << (7 * i);
            if value & 128 == 0 {
                let data = &message[i + 2..];
                if data.len() != size {
                    return Err("invalid Hunk length".into());
                }
                out.extend(data);
                found = true;
                break;
            }
        }
        if !found {
            return Err("invalid Hunk varint".into());
        }
    }
    Ok(out)
}
async fn grpc(path: &str, authorized: bool, content: &str, payload: &[u8]) -> Result {
    let socket = TcpStream::connect(("127.0.0.1", 18833)).await?;
    let (mut sender, connection) = h2::client::handshake(socket).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    let request = Request::post(format!("http://localhost{path}"))
        .header("content-type", content)
        .header("te", "trailers")
        .body(())?;
    let (response, mut stream) = sender.send_request(request, false)?;
    let password = env::var("GARLIC_FIXTURE_GRPC_PASSWORD")
        .unwrap_or_else(|_| String::from_utf8_lossy(VALID).into_owned());
    let auth = if authorized {
        format!("{:x}", Sha224::digest(password.as_bytes()))
    } else {
        "0".repeat(56)
    };
    let mut packet = auth.into_bytes();
    packet.extend(b"\r\n");
    if env::args().nth(1).as_deref() == Some("probe-relay") {
        packet.extend([1, 1, 127, 0, 0, 1]);
    } else {
        packet.extend([1, 3, TARGET.len() as u8]);
        packet.extend(TARGET);
    }
    packet.extend(18099_u16.to_be_bytes());
    packet.extend(b"\r\n");

    stream.send_data(hunk(&packet[..20]), false)?;
    stream.send_data(hunk(&packet[20..]), false)?;
    for data in payload.chunks(12_000) {
        stream.send_data(hunk(data), false)?;
    }
    stream.send_data(Bytes::new(), true)?;
    let response = timeout(Duration::from_secs(25), response).await??;
    println!(
        "gRPC {path} auth={authorized} http={} content={:?} grpc-status={:?}",
        response.status(),
        response.headers().get("content-type"),
        response.headers().get("grpc-status")
    );
    let mut rx = response.into_body();
    let mut body = Vec::new();
    while let Some(data) = timeout(Duration::from_secs(25), rx.data()).await? {
        let data = data?;
        rx.flow_control().release_capacity(data.len())?;
        body.extend(data);
    }
    let echo = decode(&body)? == payload;
    println!(
        "gRPC {path} echo={echo} payload={} bytes={} trailer={:?}",
        payload.len(),
        body.len(),
        rx.trailers()
            .await?
            .and_then(|v| v.get("grpc-status").cloned())
    );
    Ok(())
}

async fn echo() -> Result {
    let listener = TcpListener::bind(("127.0.0.1", 18099)).await?;
    println!("echo fixture ready");
    loop {
        let (connection, _) = listener.accept().await?;
        tokio::spawn(async move {
            let (mut reader, mut writer) = connection.into_split();
            let _ = tokio::io::copy(&mut reader, &mut writer).await;
            let _ = writer.shutdown().await;
        });
    }
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result {
    if env::args().nth(1).as_deref() == Some("echo") {
        return echo().await;
    }
    if env::args().nth(1).as_deref() == Some("probe-relay") {
        let big: Vec<u8> = (0..250_000).map(|i| (i % 239) as u8).collect();
        return grpc("//Tun", true, "application/grpc", &big).await;
    }
    if env::args().nth(1).as_deref() == Some("socks") {
        return tokio::try_join!(sock_server(18841), sock_server(18842)).map(|_| ());
    }
    if env::args().nth(1).as_deref() == Some("probe-signed") {
        return grpc("//Tun", true, "application/grpc", PAYLOAD).await;
    }
    for (port, auth, addr, command) in [
        (18831, true, 2, 1),
        (18832, true, 1, 1),
        (18831, true, 3, 1),
        (18831, false, 2, 1),
        (18831, true, 2, 2),
    ] {
        if let Err(error) = vless(port, auth, addr, command).await {
            println!("VLESS structure error: {error}");
        }
    }
    for (path, auth, content) in [
        ("//Tun", true, "application/grpc"),
        ("//Tun", false, "application/grpc"),
        ("//Tun", true, "application/grpc+proto"),
        ("/Tun", true, "application/grpc"),
        ("/fixture/Tun", true, "application/grpc+proto"),
    ] {
        if let Err(error) = grpc(path, auth, content, PAYLOAD).await {
            println!("gRPC {path} probe error: {error}");
        }
    }
    let big: Vec<u8> = (0..250_000).map(|i| (i % 239) as u8).collect();
    grpc("//Tun", true, "application/grpc", &big).await?;
    Ok(())
}
