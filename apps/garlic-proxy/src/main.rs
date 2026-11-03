//! Authenticated HTTP/2 transport. TLS is terminated by the supplied Caddy deployment.
use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{HeaderMap, StatusCode, Version},
    routing::post,
    Router,
};
use futures_util::StreamExt;
use std::{
    env,
    net::{IpAddr, SocketAddr},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Semaphore,
};
use tokio_util::io::ReaderStream;

type Error = Box<dyn std::error::Error + Send + Sync>;
type ApiError = (StatusCode, &'static str);
#[derive(Clone)]
struct App {
    http: reqwest::Client,
    key: String,
    exits: Arc<Vec<(String, String)>>,
    next_exit: Arc<AtomicUsize>,
    doh: String,
    doh_fallback: String,
    slots: Arc<Semaphore>,
}
fn required(name: &str) -> Result<String, Error> {
    let s = env::var(name).map_err(|_| format!("{name} is required"))?;
    if s.len() < 32 {
        return Err(format!("{name} must have at least 32 bytes").into());
    }
    Ok(s)
}
fn https_url(raw: &str) -> Result<String, Error> {
    let u = reqwest::Url::parse(raw)?;
    if u.scheme() != "https"
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
        || u.fragment().is_some()
        || u.query().is_some()
    {
        return Err("HTTPS URL without credentials, query or fragment required".into());
    }
    Ok(raw.trim_end_matches('/').into())
}
fn http_client() -> Result<reqwest::Client, Error> {
    // HTTPS + HTTP/2 prior knowledge advertises h2 only through ALPN; HTTP/1 responses are rejected too.
    let mut b = reqwest::Client::builder()
        .https_only(true)
        .http2_prior_knowledge()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10));
    if let Ok(cert) = env::var("GARLIC_CA_CERT") {
        b = b.add_root_certificate(reqwest::Certificate::from_pem(&std::fs::read(cert)?)?);
    }
    if let (Ok(host), Ok(ip)) = (
        env::var("GARLIC_BOOTSTRAP_HOST"),
        env::var("GARLIC_BOOTSTRAP_IP"),
    ) {
        // Keep domain in URL for SNI and certificate verification, use IP only for connection.
        b = b.resolve(&host, SocketAddr::new(ip.parse()?, 443));
    }
    Ok(b.build()?)
}
fn check(headers: &HeaderMap, version: Version, key: &str) -> Result<(), ApiError> {
    if version != Version::HTTP_2 {
        return Err((StatusCode::HTTP_VERSION_NOT_SUPPORTED, "HTTP/2 required"));
    }
    let provided = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let wanted = format!("Bearer {key}");
    // Fixed-length comparison avoids early exit on the first differing secret byte.
    if provided.len() != wanted.len()
        || provided
            .bytes()
            .zip(wanted.bytes())
            .fold(0u8, |a, (b, c)| a | (b ^ c))
            != 0
    {
        return Err((StatusCode::UNAUTHORIZED, "unauthorized"));
    }
    Ok(())
}
fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            let [a, b, _, _] = v.octets();
            !(v.is_private()
                || v.is_loopback()
                || v.is_link_local()
                || v.is_broadcast()
                || v.is_unspecified()
                || v.is_multicast()
                || a == 0
                || a >= 240
                || (a == 100 && (64..=127).contains(&b))
                || (a == 198 && (b == 18 || b == 19))
                || v.is_documentation())
        }
        // Reject transition/mapped addresses; only globally allocated IPv6 unicast.
        IpAddr::V6(v) => {
            let s = v.segments();
            (s[0] & 0xe000) == 0x2000
                && s[0] != 0x2002
                && !(s[0] == 0x2001 && (s[1] < 0x0200 || s[1] == 0x0db8))
        }
    }
}
async fn target(raw: &str) -> Result<TcpStream, ApiError> {
    // CONNECT authority only. No URL, credentials, paths or arbitrary destination ports.
    let u = reqwest::Url::parse(&format!("https://{raw}"))
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid target"))?;
    if u.path() != "/"
        || u.query().is_some()
        || u.fragment().is_some()
        || !u.username().is_empty()
        || u.password().is_some()
    {
        return Err((StatusCode::BAD_REQUEST, "authority required"));
    }
    let host = u
        .host_str()
        .ok_or((StatusCode::BAD_REQUEST, "host required"))?
        .trim_matches(['[', ']']);
    let port = u.port_or_known_default().unwrap_or(443);
    if port != 443 {
        return Err((StatusCode::FORBIDDEN, "only TCP 443 permitted"));
    }
    let addresses: Vec<_> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| (StatusCode::BAD_GATEWAY, "DNS failed"))?
        .collect();
    if addresses.is_empty() || addresses.iter().any(|a| !public_ip(a.ip())) {
        return Err((StatusCode::FORBIDDEN, "non-public target blocked"));
    }
    // Connect to the checked IP, never re-resolve after validation.
    for addr in addresses {
        if let Ok(Ok(s)) =
            tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(addr)).await
        {
            return Ok(s);
        }
    }
    Err((StatusCode::BAD_GATEWAY, "connect failed"))
}
async fn tunnel(
    State(s): State<App>,
    request: Request,
) -> Result<axum::response::Response, ApiError> {
    check(request.headers(), request.version(), &s.key)?;
    let permit = s
        .slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| (StatusCode::TOO_MANY_REQUESTS, "capacity exhausted"))?;
    let authority = request
        .headers()
        .get("x-garlic-target")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::BAD_REQUEST, "target required"))?
        .to_owned();
    if authority.len() > 255 {
        return Err((StatusCode::BAD_REQUEST, "target too long"));
    }
    let selected = if s.exits.is_empty() {
        None
    } else {
        Some(&s.exits[s.next_exit.fetch_add(1, Ordering::Relaxed) % s.exits.len()])
    };
    let body = if let Some((exit, key)) = selected {
        let r = s
            .http
            .post(format!("{exit}/v1/tunnel"))
            .bearer_auth(key)
            .header("x-garlic-target", authority)
            .body(reqwest::Body::wrap_stream(
                request.into_body().into_data_stream(),
            ))
            .send()
            .await
            .map_err(|_| (StatusCode::BAD_GATEWAY, "exit unavailable"))?;
        if r.version() != Version::HTTP_2 || !r.status().is_success() {
            return Err((StatusCode::BAD_GATEWAY, "exit rejected tunnel"));
        }
        let stream = r.bytes_stream().map(move |chunk| {
            let _ = &permit;
            chunk
        });
        Body::from_stream(stream)
    } else {
        let socket = target(&authority).await?;
        let (read, mut write) = socket.into_split();
        let mut incoming = request.into_body().into_data_stream();
        let upload = tokio::spawn(async move {
            while let Some(Ok(chunk)) = incoming.next().await {
                if write.write_all(&chunk).await.is_err() {
                    break;
                }
            }
            let _ = write.shutdown().await;
        });
        // Abort the upload task when the downstream response stream is dropped.
        struct Upload(tokio::task::JoinHandle<()>);
        impl Drop for Upload {
            fn drop(&mut self) {
                self.0.abort();
            }
        }
        let guard = Upload(upload);
        Body::from_stream(ReaderStream::new(read).map(move |chunk| {
            let _ = (&permit, &guard);
            chunk
        }))
    };
    Ok(axum::response::Response::new(body))
}
async fn dns(State(s): State<App>, request: Request) -> Result<axum::response::Response, ApiError> {
    check(request.headers(), request.version(), &s.key)?;
    let _permit = s
        .slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| (StatusCode::TOO_MANY_REQUESTS, "capacity exhausted"))?;
    if request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        != Some("application/dns-message")
    {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "DNS wire format required",
        ));
    }
    let data = to_bytes(request.into_body(), 65535)
        .await
        .map_err(|_| (StatusCode::PAYLOAD_TOO_LARGE, "DNS message too large"))?;
    if data.len() < 12 {
        return Err((StatusCode::BAD_REQUEST, "invalid DNS message"));
    }
    let mut urls: Vec<(String, Option<String>)> = if !s.exits.is_empty() {
        s.exits
            .iter()
            .map(|(url, key)| (format!("{url}/v1/dns"), Some(key.clone())))
            .collect()
    } else {
        vec![(s.doh.clone(), None), (s.doh_fallback.clone(), None)]
    };
    urls.dedup();
    let mut result = None;
    for (url, key) in urls {
        let mut req = s
            .http
            .post(url)
            .header("content-type", "application/dns-message")
            .header("accept", "application/dns-message")
            .body(data.clone())
            .timeout(Duration::from_secs(5));
        if let Some(key) = key {
            req = req.bearer_auth(key);
        }
        let Ok(mut response) = req.send().await else {
            continue;
        };
        if response.version() != Version::HTTP_2
            || !response.status().is_success()
            || response
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                != Some("application/dns-message")
        {
            continue;
        }
        let mut body = Vec::new();
        let mut valid = true;
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) if chunk.len() <= 65535 - body.len() => {
                    body.extend_from_slice(&chunk)
                }
                Ok(None) => break,
                _ => {
                    valid = false;
                    break;
                }
            }
        }
        if valid
            && body.len() >= 12
            && body[..2] == data[..2]
            && body[2] & 0x80 != 0
            && body[3] & 0x0f != 2
        {
            result = Some(body);
            break;
        }
    }
    let body = result.ok_or((StatusCode::BAD_GATEWAY, "DNS resolvers unavailable"))?;
    let mut r = axum::response::Response::new(Body::from(body));
    r.headers_mut()
        .insert("content-type", "application/dns-message".parse().unwrap());
    Ok(r)
}
async fn local_connection(
    mut socket: TcpStream,
    http: reqwest::Client,
    url: String,
    key: String,
) -> Result<(), Error> {
    // Local HTTP CONNECT adapter. Header is capped and read without consuming tunnel bytes.
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() >= 8192 {
            return Err("CONNECT header too large".into());
        }
        header.push(socket.read_u8().await?);
    }
    let header = std::str::from_utf8(&header)?;
    let mut line = header.lines().next().unwrap_or("").split_whitespace();
    if line.next() != Some("CONNECT") {
        socket
            .write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n")
            .await?;
        return Ok(());
    }
    let authority = line.next().ok_or("missing authority")?.to_owned();
    if line.next() != Some("HTTP/1.1") {
        return Err("HTTP/1.1 CONNECT required".into());
    }
    let (read, mut write) = socket.into_split();
    // Wait for HTTP/2 response headers before admitting browser tunnel bytes.
    let r = http
        .post(format!("{url}/v1/tunnel"))
        .bearer_auth(key)
        .header("x-garlic-target", authority)
        .body(reqwest::Body::wrap_stream(ReaderStream::new(read)))
        .send()
        .await?;
    if r.version() != Version::HTTP_2 || !r.status().is_success() {
        write
            .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
            .await?;
        return Err("tunnel rejected".into());
    }
    write
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await?;
    let mut stream = r.bytes_stream();
    while let Some(chunk) = stream.next().await {
        write.write_all(&chunk?).await?;
    }
    write.shutdown().await?;
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let mode = env::args().nth(1).unwrap_or_else(|| "server".into());
    let http = http_client()?;
    let key = required("GARLIC_PROXY_KEY")?;
    if mode == "client" {
        let url = https_url(&env::var("GARLIC_ENTRY_URL")?)?;
        let bind: SocketAddr = env::var("GARLIC_LOCAL_BIND")
            .unwrap_or_else(|_| "127.0.0.1:1080".into())
            .parse()?;
        if !bind.ip().is_loopback() {
            return Err("local proxy must bind loopback".into());
        }
        let dns_bind: SocketAddr = env::var("GARLIC_LOCAL_DNS_BIND")
            .unwrap_or_else(|_| "127.0.0.1:1053".into())
            .parse()?;
        if !dns_bind.ip().is_loopback() {
            return Err("local DNS must bind loopback".into());
        }
        let udp = Arc::new(UdpSocket::bind(dns_bind).await?);
        let (dns_http, dns_url, dns_key) = (http.clone(), url.clone(), key.clone());
        tokio::spawn(async move {
            let slots = Arc::new(Semaphore::new(64));
            let mut buffer = vec![0u8; 65535];
            loop {
                let Ok((n, peer)) = udp.recv_from(&mut buffer).await else {
                    break;
                };
                if n < 12 {
                    continue;
                }
                let Ok(permit) = slots.clone().try_acquire_owned() else {
                    continue;
                };
                let (udp, http, url, key, query) = (
                    udp.clone(),
                    dns_http.clone(),
                    dns_url.clone(),
                    dns_key.clone(),
                    buffer[..n].to_vec(),
                );
                tokio::spawn(async move {
                    let _permit = permit;
                    let operation = async {
                        let mut response = http
                            .post(format!("{url}/v1/dns"))
                            .bearer_auth(key)
                            .header("content-type", "application/dns-message")
                            .body(query.clone())
                            .send()
                            .await
                            .ok()?;
                        if response.version() != Version::HTTP_2 || !response.status().is_success()
                        {
                            return None;
                        }
                        let mut result = Vec::new();
                        while let Some(chunk) = response.chunk().await.ok()? {
                            if chunk.len() > 65535 - result.len() {
                                return None;
                            }
                            result.extend_from_slice(&chunk);
                        }
                        if result.len() < 12 || result[..2] != query[..2] || result[2] & 0x80 == 0 {
                            return None;
                        }
                        Some(result)
                    };
                    let result = tokio::time::timeout(Duration::from_secs(12), operation)
                        .await
                        .ok()
                        .flatten();
                    let result = result.unwrap_or_else(|| {
                        let mut failure = query;
                        failure[2] |= 0x80;
                        failure[3] = (failure[3] & 0xf0) | 2;
                        failure[6..12].fill(0);
                        failure
                    });
                    let _ = udp.send_to(&result, peer).await;
                });
            }
        });
        println!("Local DNS stub on {dns_bind} (UDP)");
        let listener = TcpListener::bind(bind).await?;
        let slots = Arc::new(Semaphore::new(128));
        println!("Local HTTP CONNECT proxy on {bind}");
        loop {
            let (socket, _) = listener.accept().await?;
            let Ok(permit) = slots.clone().try_acquire_owned() else {
                continue;
            };
            let (http, url, key) = (http.clone(), url.clone(), key.clone());
            tokio::spawn(async move {
                let _permit = permit;
                let _ = tokio::time::timeout(
                    Duration::from_secs(3600),
                    local_connection(socket, http, url, key),
                )
                .await;
            });
        }
    } else if mode == "server" {
        let raw_exits = env::var("GARLIC_EXIT_URLS")
            .or_else(|_| env::var("GARLIC_EXIT_URL"))
            .unwrap_or_default();
        let mut exits = Vec::new();
        for raw in raw_exits
            .split(',')
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            exits.push((https_url(raw)?, required("GARLIC_EXIT_KEY")?));
        }
        let doh = https_url(
            &env::var("GARLIC_DOH_URL")
                .unwrap_or_else(|_| "https://cloudflare-dns.com/dns-query".into()),
        )?;
        let doh_fallback = https_url(
            &env::var("GARLIC_DOH_FALLBACK_URL")
                .unwrap_or_else(|_| "https://dns.google/dns-query".into()),
        )?;
        let bind: SocketAddr = env::var("GARLIC_PROXY_BIND")
            .unwrap_or_else(|_| "127.0.0.1:8070".into())
            .parse()?;
        if !bind.ip().is_loopback() {
            return Err("backend must bind loopback behind TLS proxy".into());
        }
        let app = App {
            http,
            key,
            exits: Arc::new(exits),
            next_exit: Arc::new(AtomicUsize::new(0)),
            doh,
            doh_fallback,
            slots: Arc::new(Semaphore::new(128)),
        };
        let router = Router::new()
            .route("/v1/tunnel", post(tunnel))
            .route("/v1/dns", post(dns))
            .with_state(app);
        axum::serve(TcpListener::bind(bind).await?, router)
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await?;
    } else {
        return Err("usage: garlic-proxy [server|client]".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_targets_blocked() {
        for raw in [
            "127.0.0.1",
            "10.0.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "::1",
            "::ffff:127.0.0.1",
            "2001:db8::1",
        ] {
            assert!(!public_ip(raw.parse().unwrap()), "{raw}");
        }
        assert!(public_ip("1.1.1.1".parse().unwrap()));
    }
    #[test]
    fn secure_endpoints_only() {
        assert!(https_url("http://example.org").is_err());
        assert!(https_url("https://user:pass@example.org").is_err());
        assert!(https_url("https://example.org").is_ok());
    }
    #[test]
    fn h2_and_auth_required() {
        let mut h = HeaderMap::new();
        h.insert("authorization", "Bearer test".parse().unwrap());
        assert!(check(&h, Version::HTTP_11, "test").is_err());
        assert!(check(&h, Version::HTTP_2, "other").is_err());
        assert!(check(&h, Version::HTTP_2, "test").is_ok());
    }
}
