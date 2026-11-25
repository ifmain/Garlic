//! Bounded outbound transport capabilities. Only a configured network slot can select one.
use super::*;
use base64::engine::general_purpose::STANDARD;
use std::sync::OnceLock;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream as Socket;

trait Duplex: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Unpin> Duplex for T {}
type Stream = Box<dyn Duplex>;
pub(super) fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("connector runtime")
    })
}
#[derive(Default, serde::Deserialize, Clone)]
#[serde(default)]
pub(super) struct Login {
    pub user: String,
    pub password: String,
    pub pin: String,
}

#[derive(Clone)]
pub(super) struct Peer {
    pub kind: String,
    pub address: String,
    pub host: String,
    pub port: u16,
    pub login: Login,
}
pub(super) fn code(kind: &str) -> Option<i32> {
    ["http", "https", "socks4", "socks5", "ssh"]
        .iter()
        .position(|k| *k == kind)
        .map(|n| n as i32 + 1)
}
impl Peer {
    pub fn new(_slot: &str, kind: &str, address: &str, login: Login) -> Result<Self, StateError> {
        let u = url::Url::parse(address)?;
        if code(kind).is_none()
            || u.scheme() != kind
            || !u.username().is_empty()
            || u.password().is_some()
            || u.query().is_some()
            || u.fragment().is_some()
            || !matches!(u.path(), "" | "/")
        {
            return Err("invalid proxy address".into());
        }
        let host = u
            .host_str()
            .ok_or("missing proxy host")?
            .trim_matches(['[', ']'])
            .to_owned();
        let port = u
            .port_or_known_default()
            .unwrap_or(if kind == "ssh" { 22 } else { 1080 });
        if port == 0
            || login.user.len() > 255
            || login.password.len() > 255
            || login.user.contains('\0')
        {
            return Err("invalid proxy settings".into());
        }
        Ok(Self {
            kind: kind.into(),
            address: address.into(),
            host,
            port,
            login,
        })
    }
    async fn stream(&self, dest: &str) -> Result<Stream, StateError> {
        let (h, p) = dest.rsplit_once(':').ok_or("destination requires port")?;
        let host = h.trim_matches(['[', ']']);
        let port: u16 = p.parse()?;
        if !valid_connect_authority(dest)
            || host.len() > 255
            || dest.bytes().any(|c| c <= 32 || c == 127)
        {
            return Err("invalid destination".into());
        }
        let sock = Socket::connect((self.host.as_str(), self.port)).await?;
        sock.set_nodelay(true)?;
        if self.kind == "ssh" {
            if !self.login.pin.starts_with("SHA256:") || self.login.user.is_empty() {
                return Err("SSH username and SHA256 host fingerprint required".into());
            }
            let mut ssh = russh::client::connect_stream(
                Default::default(),
                sock,
                Pin(self.login.pin.clone()),
            )
            .await?;
            if !ssh
                .authenticate_password(self.login.user.clone(), self.login.password.clone())
                .await?
                .success()
            {
                return Err("SSH authentication rejected".into());
            }
            let ch = ssh
                .channel_open_direct_tcpip(host, u32::from(port), "127.0.0.1", 0)
                .await?;
            return Ok(Box::new(ch.into_stream()));
        }
        let mut s: Stream = Box::new(sock);
        match self.kind.as_str() {
            "http" | "https" => {
                if self.kind == "https" {
                    s = tls(s, &self.host, &self.login.pin).await?;
                }
                let mut req = format!("CONNECT {dest} HTTP/1.1\r\nHost: {dest}\r\n");
                if !self.login.user.is_empty() {
                    let token =
                        STANDARD.encode(format!("{}:{}", self.login.user, self.login.password));
                    req.push_str(&format!("Proxy-Authorization: Basic {token}\r\n"));
                }
                req.push_str("\r\n");
                s.write_all(req.as_bytes()).await?;
                let header = headers(&mut s).await?;
                if !header
                    .lines()
                    .next()
                    .and_then(|l| l.split_whitespace().nth(1))
                    .is_some_and(|v| v == "200")
                {
                    return Err("HTTP CONNECT rejected".into());
                }
            }
            "socks4" => {
                let mut req = vec![4, 1];
                req.extend(port.to_be_bytes());
                let ip = host.parse::<Ipv4Addr>().ok();
                req.extend(ip.map(|x| x.octets()).unwrap_or([0, 0, 0, 1]));
                req.extend(self.login.user.as_bytes());
                req.push(0);
                if ip.is_none() {
                    req.extend(host.as_bytes());
                    req.push(0);
                }
                s.write_all(&req).await?;
                let mut reply = [0; 8];
                s.read_exact(&mut reply).await?;
                if reply[..2] != [0, 90] {
                    return Err("SOCKS4 CONNECT rejected".into());
                }
            }
            "socks5" => {
                let method = if self.login.user.is_empty() { 0 } else { 2 };
                s.write_all(&[5, 1, method]).await?;
                let mut reply = [0; 2];
                s.read_exact(&mut reply).await?;
                if reply != [5, method] {
                    return Err("SOCKS5 method rejected".into());
                }
                if method == 2 {
                    let mut auth = vec![1, self.login.user.len() as u8];
                    auth.extend(self.login.user.as_bytes());
                    auth.push(self.login.password.len() as u8);
                    auth.extend(self.login.password.as_bytes());
                    s.write_all(&auth).await?;
                    s.read_exact(&mut reply).await?;
                    if reply != [1, 0] {
                        return Err("SOCKS5 authentication rejected".into());
                    }
                }
                let mut req = vec![5, 1, 0];
                match host.parse::<std::net::IpAddr>() {
                    Ok(std::net::IpAddr::V4(ip)) => {
                        req.push(1);
                        req.extend(ip.octets())
                    }
                    Ok(std::net::IpAddr::V6(ip)) => {
                        req.push(4);
                        req.extend(ip.octets())
                    }
                    Err(_) => {
                        req.extend([3, host.len() as u8]);
                        req.extend(host.as_bytes())
                    }
                }
                req.extend(port.to_be_bytes());
                s.write_all(&req).await?;
                let mut head = [0; 4];
                s.read_exact(&mut head).await?;
                if head[..3] != [5, 0, 0] {
                    return Err("SOCKS5 CONNECT rejected".into());
                }
                let n = match head[3] {
                    1 => 4,
                    4 => 16,
                    3 => usize::from(s.read_u8().await?),
                    _ => return Err("invalid SOCKS5 reply".into()),
                };
                s.read_exact(&mut vec![0; n + 2]).await?;
            }
            _ => return Err("unsupported transport".into()),
        }
        Ok(s)
    }
    pub fn open(&self, dest: &str) -> Result<Opened, StateError> {
        runtime().block_on(async {
            Ok(Opened(Some(
                tokio::time::timeout(Duration::from_secs(5), self.stream(dest)).await??,
            )))
        })
    }
    pub fn test(&self) -> Result<(), StateError> {
        runtime().block_on(async{
            tokio::time::timeout(Duration::from_secs(9),async{
                let s=self.stream("www.gstatic.com:443").await?;
                let mut verified=tls(s,"www.gstatic.com","").await?;
                verified.write_all(b"GET /generate_204 HTTP/1.1\r\nHost: www.gstatic.com\r\nConnection: close\r\n\r\n").await?;
                if headers(&mut verified).await?.lines().next().and_then(|s|s.split_whitespace().nth(1))!=Some("204"){return Err("unexpected probe response".into())}
                Ok::<_,StateError>(())
            }).await?
        })
    }
}
struct Pin(String);
impl russh::client::Handler for Pin {
    type Error = russh::Error;
    async fn check_server_key(
        &mut self,
        key: &russh::keys::PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        Ok(match key {
            russh::keys::PublicKeyOrCertificate::PublicKey { key: k, .. } => {
                k.fingerprint(russh::keys::HashAlg::Sha256).to_string() == self.0
            }
            _ => false,
        })
    }
}
async fn tls(s: Stream, host: &str, ca_der: &str) -> Result<Stream, StateError> {
    static CFG: OnceLock<Arc<rustls::ClientConfig>> = OnceLock::new();
    let config = |roots| {
        Arc::new(
            rustls::ClientConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_safe_default_protocol_versions()
            .expect("TLS versions")
            .with_root_certificates(roots)
            .with_no_client_auth(),
        )
    };
    let cfg = if ca_der.is_empty() {
        CFG.get_or_init(|| {
            config(rustls::RootCertStore::from_iter(
                webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
            ))
        })
        .clone()
    } else {
        let mut roots = rustls::RootCertStore::empty();
        roots.add(STANDARD.decode(ca_der)?.into())?;
        config(roots)
    };
    Ok(Box::new(
        tokio_rustls::TlsConnector::from(cfg.clone())
            .connect(host.to_owned().try_into()?, s)
            .await?,
    ))
}
async fn headers(s: &mut Stream) -> Result<String, StateError> {
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() >= 8192 {
            return Err("proxy headers too long".into());
        }
        head.push(s.read_u8().await?);
    }
    Ok(String::from_utf8_lossy(&head).into_owned())
}
pub(super) struct Opened(Option<Stream>);
impl Drop for Opened {
    fn drop(&mut self) {
        // SSH channel cleanup schedules an async close, including on early relay errors.
        let _runtime = runtime().enter();
        drop(self.0.take());
    }
}
impl Opened {
    pub fn relay(mut self, local: &TcpStream, proxy: &Proxy) -> Result<(), StateError> {
        let sock = local.try_clone()?;
        sock.set_nonblocking(true)?;
        runtime().block_on(async{
            let mut local=Socket::from_std(sock)?;
            let (mut lr,mut lw)=local.split();let (mut rr,mut rw)=tokio::io::split(self.0.as_mut().expect("open stream"));
            async fn pump(mut r:impl AsyncRead+Unpin,mut w:impl AsyncWrite+Unpin,count:&std::sync::atomic::AtomicU64)->io::Result<()>{
                let mut buf=[0;16384];loop{let n=r.read(&mut buf).await?;if n==0{return w.shutdown().await}
                    w.write_all(&buf[..n]).await?;count.fetch_add(n as u64,Ordering::Relaxed);}
            }
            tokio::select!{
                result=async{tokio::try_join!(pump(&mut lr,&mut rw,&proxy.traffic.tx),pump(&mut rr,&mut lw,&proxy.traffic.rx))}=>{result?;},
                _=async{while !proxy.stop.load(Ordering::Acquire){tokio::time::sleep(Duration::from_millis(250)).await;}}=>{}
            };Ok(())
        })
    }
}
