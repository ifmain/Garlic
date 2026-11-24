use std::{
    io,
    net::{IpAddr, SocketAddr},
};
use tokio::{io::AsyncReadExt, net::TcpStream};

const MAX_HEADER_BYTES: usize = 108;

pub(crate) async fn peer(
    stream: &mut TcpStream,
    socket_peer: SocketAddr,
    trust_loopback: bool,
) -> io::Result<SocketAddr> {
    if !trust_loopback || !socket_peer.ip().is_loopback() {
        return Ok(socket_peer);
    }
    let mut first = [0];
    if stream.peek(&mut first).await? == 0 || first != [b'P'] {
        return Ok(socket_peer);
    }

    let mut header = Vec::with_capacity(MAX_HEADER_BYTES);
    while header.len() < MAX_HEADER_BYTES {
        let byte = stream.read_u8().await?;
        header.push(byte);
        if byte == b'\n' {
            return parse(&header);
        }
    }
    Err(invalid_header())
}

fn invalid_header() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid PROXY protocol header")
}

fn parse(header: &[u8]) -> io::Result<SocketAddr> {
    let line = header.strip_suffix(b"\r\n").ok_or_else(invalid_header)?;
    let line = std::str::from_utf8(line).map_err(|_| invalid_header())?;
    let mut words = line.split(' ');
    if words.next() != Some("PROXY") {
        return Err(invalid_header());
    }
    let ipv4 = match words.next() {
        Some("TCP4") => true,
        Some("TCP6") => false,
        _ => return Err(invalid_header()),
    };
    let mut address = || {
        words
            .next()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .filter(|ip| ip.is_ipv4() == ipv4)
            .ok_or_else(invalid_header)
    };
    let source = address()?;
    let _ = address()?;
    let mut port = || {
        words
            .next()
            .and_then(|value| value.parse::<u16>().ok())
            .ok_or_else(invalid_header)
    };
    let source_port = port()?;
    let _ = port()?;
    if words.next().is_some() {
        return Err(invalid_header());
    }
    Ok(SocketAddr::new(source, source_port))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{io::AsyncWriteExt, net::TcpListener};

    #[tokio::test]
    async fn loopback_header_preserves_tls_bytes_and_obeys_trust_setting() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let header = b"PROXY TCP4 192.0.2.31 127.0.0.1 57098 443\r\n\x16\x03\x01";
        for trust in [false, true] {
            let mut client = TcpStream::connect(listener.local_addr().unwrap())
                .await
                .unwrap();
            client.write_all(header).await.unwrap();
            let (mut stream, socket_peer) = listener.accept().await.unwrap();
            let forwarded = peer(&mut stream, socket_peer, trust).await.unwrap();
            if trust {
                assert_eq!(forwarded, "192.0.2.31:57098".parse().unwrap());
                assert_eq!(stream.read_u8().await.unwrap(), 0x16);
            } else {
                assert_eq!(forwarded, socket_peer);
                assert_eq!(stream.read_u8().await.unwrap(), b'P');
            }
        }
        assert!(parse(b"PROXY UNKNOWN\r\n").is_err());
        assert!(parse(b"PROXY TCP4 ::1 127.0.0.1 57098 443\r\n").is_err());
    }
}
