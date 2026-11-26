use crate::proxy::{receive_request, Request};
use garlic_relay_data::control::valid_connect_authority;
use garlic_state::StateError;
use hickory_proto::{
    op::{Header, Message, MessageType, Query, ResponseCode},
    rr::{Name, RData, RecordType},
    serialize::binary::BinDecodable,
};
use serde::Deserialize;
use socket2::{Domain, Protocol, Socket, Type};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, TcpStream as StdTcpStream},
    os::windows::io::AsRawSocket,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{self as aio, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream, UdpSocket},
    task::JoinSet,
    time::timeout,
};
use windows_sys::Win32::Networking::WinSock::{
    setsockopt, WSAGetLastError, IPPROTO_IP, IPPROTO_IPV6, IPV6_UNICAST_IF, IP_UNICAST_IF,
};

const LISTEN: &str = "127.0.0.1:10999";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_CONNECTIONS: usize = 128;

#[derive(Deserialize)]
pub(super) struct Egress {
    ipv4: Option<u32>,
    ipv6: Option<u32>,
    dns: Vec<String>,
}

impl Egress {
    fn socket(&self, target: SocketAddr, kind: Type, protocol: Protocol) -> io::Result<Socket> {
        let (index, level, opt) = match target {
            SocketAddr::V4(_) => (self.ipv4.map(u32::to_be), IPPROTO_IP, IP_UNICAST_IF),
            SocketAddr::V6(_) => (self.ipv6, IPPROTO_IPV6, IPV6_UNICAST_IF),
        };
        let index = index.filter(|index| *index != 0).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NetworkUnreachable, "no direct interface")
        })?;
        let socket = Socket::new(Domain::for_address(target), kind, Some(protocol))?;
        let index = index.to_ne_bytes();
        // Winsock requires network byte order for the IPv4 index and host order for IPv6.
        let result =
            unsafe { setsockopt(socket.as_raw_socket() as _, level, opt, index.as_ptr(), 4) };
        if result != 0 {
            return Err(io::Error::from_raw_os_error(unsafe { WSAGetLastError() }));
        }
        socket.set_nonblocking(true)?;
        Ok(socket)
    }

    async fn tcp(&self, target: SocketAddr) -> io::Result<TcpStream> {
        let socket = self.socket(target, Type::STREAM, Protocol::TCP)?;
        TcpSocket::from_std_stream(socket.into())
            .connect(target)
            .await
    }

    async fn dns_response(
        &self,
        server: SocketAddr,
        request: &Message,
    ) -> Result<Message, StateError> {
        let query = request.to_vec()?;
        let socket = self.socket(server, Type::DGRAM, Protocol::UDP)?;
        let wildcard = if server.is_ipv4() {
            Ipv4Addr::UNSPECIFIED.into()
        } else {
            Ipv6Addr::UNSPECIFIED.into()
        };
        socket.bind(&SocketAddr::new(wildcard, 0).into())?;
        let socket = UdpSocket::from_std(socket.into())?;
        socket.connect(server).await?;
        socket.send(&query).await?;
        let mut bytes = vec![0; 4096];
        let received = socket.recv(&mut bytes).await?;
        bytes.truncate(received);
        // The last record can be incomplete in a truncated UDP response.
        let header = Header::from_bytes(&bytes)?;
        if header.metadata.id != request.metadata.id
            || header.metadata.message_type != MessageType::Response
        {
            return Err("invalid direct DNS response".into());
        }
        if header.metadata.truncation {
            let mut tcp = self.tcp(server).await?;
            tcp.write_u16(query.len().try_into()?).await?;
            tcp.write_all(&query).await?;
            bytes.resize(tcp.read_u16().await?.into(), 0);
            tcp.read_exact(&mut bytes).await?;
        }
        let reply = Message::from_vec(&bytes)?;
        if reply.metadata.id != request.metadata.id
            || reply.metadata.message_type != MessageType::Response
            || reply.metadata.truncation
            || reply.queries != request.queries
        {
            return Err("invalid direct DNS response".into());
        }
        Ok(reply)
    }

    async fn resolve(&self, name: &Name, kind: RecordType) -> Vec<IpAddr> {
        if (kind == RecordType::A && self.ipv4.is_none())
            || (kind == RecordType::AAAA && self.ipv6.is_none())
        {
            return Vec::new();
        }
        let mut request = Message::query();
        request.metadata.recursion_desired = true;
        request.queries.push(Query::query(name.clone(), kind));
        for server in &self.dns {
            let Ok(mut server) = format!("{server}:53")
                .parse::<SocketAddr>()
                .or_else(|_| format!("[{server}]:53").parse())
            else {
                continue;
            };
            if let SocketAddr::V6(server) = &mut server {
                if server.ip().is_unicast_link_local() && server.scope_id() == 0 {
                    server.set_scope_id(self.ipv6.unwrap_or_default());
                }
            }
            if !usable_address(server.ip()) {
                continue;
            }
            let Ok(Ok(reply)) = timeout(ATTEMPT_TIMEOUT, self.dns_response(server, &request)).await
            else {
                continue;
            };
            match reply.metadata.response_code {
                ResponseCode::NoError => {
                    let mut owner = name;
                    // Follow only aliases attached to the requested owner.
                    for _ in 0..16 {
                        let mut addresses = Vec::new();
                        let mut alias = None;
                        for record in reply.answers.iter().filter(|record| &record.name == owner) {
                            match &record.data {
                                RData::A(ip) if kind == RecordType::A => {
                                    addresses.push(IpAddr::V4(ip.0))
                                }
                                RData::AAAA(ip) if kind == RecordType::AAAA => {
                                    addresses.push(IpAddr::V6(ip.0))
                                }
                                RData::CNAME(target) => alias = Some(&target.0),
                                _ => (),
                            }
                        }
                        if !addresses.is_empty() {
                            return addresses;
                        }
                        let Some(alias) = alias else { break };
                        owner = alias;
                    }
                    return Vec::new();
                }
                ResponseCode::NXDomain => return Vec::new(),
                _ => continue,
            }
        }
        Vec::new()
    }

    async fn connect(&self, authority: &str) -> Result<TcpStream, StateError> {
        if !valid_connect_authority(authority) {
            return Err("invalid direct proxy destination".into());
        }
        let (host, port) = authority.rsplit_once(':').ok_or("missing port")?;
        let port = port.parse()?;
        let host = host.trim_start_matches('[').trim_end_matches(']');
        let addresses = if let Ok(address) = host.parse::<IpAddr>() {
            vec![address]
        } else {
            let mut name = Name::from_utf8(host)?;
            name.set_fqdn(true);
            let (mut ipv4, ipv6) = tokio::join!(
                self.resolve(&name, RecordType::A),
                self.resolve(&name, RecordType::AAAA)
            );
            ipv4.extend(ipv6);
            ipv4
        };
        for ip in addresses.into_iter().filter(|ip| usable_address(*ip)) {
            if let Ok(Ok(stream)) =
                timeout(ATTEMPT_TIMEOUT, self.tcp(SocketAddr::new(ip, port))).await
            {
                return Ok(stream);
            }
        }
        Err("no reachable direct proxy destination".into())
    }
}

fn usable_address(ip: IpAddr) -> bool {
    !ip.is_loopback() && !ip.is_unspecified() && !ip.is_multicast() && ip == ip.to_canonical()
}

pub(super) struct DirectProxy {
    listener: TcpListener,
    egress: Arc<Egress>,
}

impl DirectProxy {
    pub(super) async fn bind(egress: Egress) -> Result<Self, StateError> {
        let listener = TcpListener::bind(LISTEN)
            .await
            .map_err(|error| format!("direct proxy {LISTEN}: {error}"))?;
        Ok(Self {
            listener,
            egress: Arc::new(egress),
        })
    }

    pub(super) async fn serve(self) -> Result<(), StateError> {
        let mut connections = JoinSet::new();
        println!("Garlic direct CONNECT/SOCKS5 proxy listening on {LISTEN}");
        loop {
            tokio::select! {
                incoming = self.listener.accept(), if connections.len() < MAX_CONNECTIONS => {
                    let (local, _) = incoming?;
                    let egress = self.egress.clone();
                    connections.spawn(async move { let _ = handle(local, &egress).await; });
                }
                _ = connections.join_next(), if !connections.is_empty() => (),
            }
        }
    }
}

struct ShutdownOnDrop(Option<StdTcpStream>);

impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        if let Some(stream) = &self.0 {
            let _ = stream.shutdown(Shutdown::Both);
        }
    }
}

async fn handle(local: TcpStream, egress: &Egress) -> Result<(), StateError> {
    // Reuse the regular proxy handshake. Shutdown interrupts blocking reads on
    // timeout or when the VPN aborts this connection.
    let mut local = local.into_std()?;
    local.set_nonblocking(false)?;
    let mut interrupt = ShutdownOnDrop(Some(local.try_clone()?));
    let (local, request) = timeout(
        REQUEST_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            let request = receive_request(&mut local);
            (local, request)
        }),
    )
    .await??;
    interrupt.0 = None;
    local.set_nonblocking(true)?;
    let mut local = TcpStream::from_std(local)?;
    let Request::Connect(authority, socks) = request? else {
        local.write_all(b"HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\nContent-Length: 0\r\n\r\n").await?;
        return Ok(());
    };
    let mut remote = match timeout(REQUEST_TIMEOUT, egress.connect(&authority)).await {
        Ok(Ok(remote)) => remote,
        _ => {
            let reply: &[u8] = if socks {
                &[5, 4, 0, 1, 0, 0, 0, 0, 0, 0]
            } else {
                b"HTTP/1.1 502 Bad Gateway\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
            };
            local.write_all(reply).await?;
            return Ok(());
        }
    };
    let reply: &[u8] = if socks {
        &[5, 0, 0, 1, 0, 0, 0, 0, 0, 0]
    } else {
        b"HTTP/1.1 200 Connection Established\r\n\r\n"
    };
    local.write_all(reply).await?;
    aio::copy_bidirectional(&mut local, &mut remote).await?;
    Ok(())
}

#[cfg(test)]
#[test]
#[ignore = "requires active Windows VPN routes and a public IP echo URL"]
fn splits_egress_on_windows() {
    let egress =
        serde_json::from_str(&std::env::var("GARLIC_DIRECT_TEST_EGRESS").unwrap()).unwrap();
    let url = std::env::var("GARLIC_DIRECT_TEST_URL").unwrap();
    let runtime = super::runtime().unwrap();
    let proxy = runtime.block_on(DirectProxy::bind(egress)).unwrap();
    let server = runtime.spawn(proxy.serve());

    let address = |proxy: &str| {
        let result = std::process::Command::new("curl.exe")
            .args([
                "-fsS",
                "--max-time",
                "20",
                "--noproxy",
                "",
                "--proxy",
                proxy,
                &url,
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout)
            .unwrap()
            .trim()
            .parse::<IpAddr>()
            .unwrap()
    };
    let vpn_address = address("");
    let socks_address = address(&format!("socks5h://{LISTEN}"));
    let connect_address = address(&format!("http://{LISTEN}"));
    assert!(vpn_address != socks_address, "SOCKS5 used the VPN egress");
    assert!(
        socks_address == connect_address,
        "SOCKS5 and HTTP CONNECT used different egresses"
    );

    server.abort();
    assert!(runtime.block_on(server).unwrap_err().is_cancelled());
    assert!(
        StdTcpStream::connect(LISTEN).is_err(),
        "listener survived shutdown"
    );
}
