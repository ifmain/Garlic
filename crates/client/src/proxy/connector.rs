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
    pub slot: String,
    pub kind: String,
    pub address: String,
    pub host: String,
    pub port: u16,
    pub login: Login,
    pub wire: Option<(release_cache::guest::Guest, String)>,
    pub gate: Option<std::sync::Arc<SlotGate>>,
}
impl Peer {
    pub fn new(slot: &str, kind: &str, address: &str, login: Login) -> Result<Self, StateError> {
        let u = url::Url::parse(address)?;
        if kind.len() > 32
            || !kind
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
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
            slot: slot.into(),
            kind: kind.into(),
            address: address.into(),
            host,
            port,
            login,
            wire: None,
            gate: None,
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
        if !self.enabled() {
            return Err("plugin disabled".into());
        }
        let sock = Socket::connect((self.host.as_str(), self.port)).await?;
        sock.set_nodelay(true)?;
        let layer = self.wire.as_ref().map(|(_, layer)| layer.as_str());
        if layer == Some("ssh") {
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
            return self.wrap(Box::new(ch.into_stream()), dest).await;
        }
        let mut s: Stream = Box::new(sock);
        if let Some(layer) = layer {
            if layer == "tls" {
                s = tls(s, &self.host, &self.login.pin).await?;
            }
            return self.wrap(s, dest).await;
        }
        Err("missing network guest".into())
    }
    async fn wrap(&self, mut remote: Stream, dest: &str) -> Result<Stream, StateError> {
        let Some((guest, layer)) = &self.wire else {
            return Ok(remote);
        };
        let mut vm = release_cache::wire::Wire::new(guest)?;
        // Passwords consumed by the SSH layer stay outside the guest.
        let password = if layer == "ssh" {
            ""
        } else {
            self.login.password.as_str()
        };
        let context =
            [self.kind.as_str(), dest, self.login.user.as_str(), password].join("\0") + "\0";
        let (mut ready, hello) = vm.event(0, context.as_bytes())?;
        remote.write_all(&hello).await?;
        for _ in 0..8192 {
            if ready {
                break;
            }
            let byte = remote.read_u8().await?;
            let (done, reply) = vm.event(1, &[byte])?;
            if !reply.is_empty() {
                remote.write_all(&reply).await?;
            }
            ready = done;
        }
        if !ready {
            return Err("plugin handshake limit exceeded".into());
        }
        let (app, channel) = tokio::io::duplex(32768);
        let gate = self.gate.clone();
        let task = tokio::spawn(async move {
            let (mut input, mut output) = tokio::io::split(channel);
            let (mut rx, mut tx) = tokio::io::split(remote);
            let mut a = [0; 4096];
            let mut b = [0; 4096];
            let (mut local_open, mut peer_open) = (true, true);
            let work = async {
                while local_open || peer_open {
                    tokio::select! {
                        n=input.read(&mut a),if local_open=>{
                            let n=n?;let (_,bytes)=vm.event(if n==0{4}else{2},&a[..n])?;
                            tx.write_all(&bytes).await?;
                            if n==0{local_open=false;tx.shutdown().await?;}
                        }
                        n=rx.read(&mut b),if peer_open=>{
                            let n=n?;let (_,bytes)=vm.event(if n==0{5}else{3},&b[..n])?;
                            output.write_all(&bytes).await?;
                            if n==0{peer_open=false;output.shutdown().await?;}
                        }
                    }
                }
                Ok::<(), StateError>(())
            };
            let revoked = async {
                loop {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    if !gate.as_ref().is_some_and(|g| g.enabled()) {
                        break;
                    }
                }
            };
            tokio::select! { _=work=>{}, _=revoked=>{} }
        });
        Ok(Box::new(GuestStream(app, task.abort_handle())))
    }
    pub fn enabled(&self) -> bool {
        self.gate.as_ref().is_some_and(|g| g.enabled())
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

// A dropped stream must also release its per-connection VM and upstream socket.
struct GuestStream(tokio::io::DuplexStream, tokio::task::AbortHandle);
impl Drop for GuestStream {
    fn drop(&mut self) {
        self.1.abort();
    }
}
impl AsyncRead for GuestStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        b: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_read(cx, b)
    }
}
impl AsyncWrite for GuestStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        b: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        std::pin::Pin::new(&mut self.0).poll_write(cx, b)
    }
    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_shutdown(cx)
    }
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_flush(cx)
    }
}
pub(super) struct SlotGate {
    path: std::path::PathBuf,
    cached: std::sync::Mutex<Option<(Instant, bool)>>,
}
impl SlotGate {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            cached: Default::default(),
        }
    }
    pub(super) fn enabled(&self) -> bool {
        let mut cache = self.cached.lock().unwrap();
        if let Some((at, on)) = *cache {
            if at.elapsed() < Duration::from_millis(200) {
                return on;
            }
        }
        let on = std::fs::File::open(&self.path)
            .ok()
            .and_then(|f| serde_json::from_reader::<_, Value>(std::io::Read::take(f, 16385)).ok())
            .is_some_and(|v| v["enabled"] == true);
        *cache = Some((Instant::now(), on));
        on
    }
}
