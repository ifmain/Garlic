use super::{socks, sync, Result};
use axum::http::{HeaderMap, Method, Request, Response, StatusCode};
use bytes::{Bytes, BytesMut};
use h2::{server::SendResponse, RecvStream, SendStream};
use serde::{Deserialize, Serialize};
use std::{
    future::poll_fn,
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use subtle::ConstantTimeEq;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};

const MESSAGE_LIMIT: usize = 4 * 1024 * 1024;
const ACCOUNT_LIMIT: u64 = 1024 * 1024;

#[derive(Deserialize)]
pub struct Settings {
    listen: SocketAddr,
    accounts_file: PathBuf,
    #[serde(default)]
    service_name: String,
    sync: Option<sync::Source>,
}

#[derive(Deserialize, Serialize)]
struct Account {
    sha224: String,
    expiry_ms: u64,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Grant {
    accounts: Vec<Account>,
    pub valid_until_ms: u64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Directory {
    Grant(Grant),
    File(Vec<Account>),
}

enum Accounts {
    Local(PathBuf),
    Synchronized(sync::Cache),
}

async fn read_directory(path: &Path) -> Result<Directory> {
    let mut input = Vec::new();
    tokio::fs::File::open(path)
        .await?
        .take(ACCOUNT_LIMIT + 1)
        .read_to_end(&mut input)
        .await?;
    if input.len() as u64 > ACCOUNT_LIMIT {
        return Err("account directory is too large".into());
    }
    Ok(serde_json::from_slice(&input)?)
}

fn has_account(entries: &[Account], digest: &[u8], now: u128) -> bool {
    entries.iter().any(|entry| {
        (entry.expiry_ms == 0 || u128::from(entry.expiry_ms) > now)
            && entry.sha224.as_bytes().ct_eq(digest).unwrap_u8() == 1
    })
}

impl Accounts {
    async fn authorized(&self, digest: &[u8]) -> Result<bool> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let grant_matches = |grant: &Grant| {
            u128::from(grant.valid_until_ms) > now && has_account(&grant.accounts, digest, now)
        };
        Ok(match self {
            Self::Local(path) => match read_directory(path).await? {
                Directory::File(entries) => has_account(&entries, digest, now),
                Directory::Grant(grant) => grant_matches(&grant),
            },
            Self::Synchronized(cache) => cache
                .read()
                .map_err(|_| "legacy account cache unavailable")?
                .as_ref()
                .is_some_and(grant_matches),
        })
    }
}

fn frame(data: &[u8]) -> Bytes {
    let mut message = vec![0; 5];
    message.push(10);
    let mut size = data.len();
    while size >= 128 {
        message.push((size as u8 & 127) | 128);
        size >>= 7;
    }
    message.push(size as u8);
    message.extend(data);
    let size = (message.len() - 5) as u32;
    message[1..5].copy_from_slice(&size.to_be_bytes());
    message.into()
}

fn payload(message: Bytes) -> Result<Bytes> {
    if message.is_empty() {
        return Ok(message);
    }
    if message[0] != 10 {
        return Err("expected a gRPC Hunk".into());
    }
    let mut length = 0;
    for (offset, byte) in message[1..].iter().copied().take(4).enumerate() {
        length |= usize::from(byte & 127) << (offset * 7);
        if byte & 128 == 0 {
            let data = offset + 2;
            if message.len() - data == length {
                return Ok(message.slice(data..));
            }
            break;
        }
    }
    Err("invalid gRPC Hunk length".into())
}

struct Hunks {
    stream: RecvStream,
    buffer: BytesMut,
}

impl Hunks {
    async fn next(&mut self) -> Result<Option<Bytes>> {
        loop {
            if self.buffer.len() >= 5 {
                let length = u32::from_be_bytes(self.buffer[1..5].try_into()?) as usize;
                if self.buffer[0] != 0 || length > MESSAGE_LIMIT {
                    return Err("unsupported gRPC message".into());
                }
                if self.buffer.len() >= 5 + length {
                    return Ok(Some(payload(
                        self.buffer.split_to(5 + length).freeze().slice(5..),
                    )?));
                }
            }
            match self.stream.data().await.transpose()? {
                Some(data) => {
                    self.buffer.extend_from_slice(&data);
                    self.stream.flow_control().release_capacity(data.len())?;
                }
                None if self.buffer.is_empty() => return Ok(None),
                None => return Err("incomplete gRPC message".into()),
            }
        }
    }
}

struct Destination {
    address: Vec<u8>,
    port: u16,
    payload: usize,
}

async fn destination(input: &[u8]) -> io::Result<Destination> {
    if input.len() < 60 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    if &input[56..59] != b"\r\n\x01" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported Trojan request",
        ));
    }
    let mut tail = &input[60..];
    let address = socks::address(&mut tail, input[59]).await?;
    let port = tail.read_u16().await?;
    let mut delimiter = [0; 2];
    tail.read_exact(&mut delimiter).await?;
    if delimiter != *b"\r\n" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid Trojan request trailer",
        ));
    }
    Ok(Destination {
        address,
        port,
        payload: input.len() - tail.len(),
    })
}

async fn handshake(hunks: &mut Hunks, accounts: &Accounts) -> Result<(Destination, Vec<u8>)> {
    let mut request = Vec::new();
    loop {
        match destination(&request).await {
            Ok(destination) => {
                if !accounts.authorized(&request[..56]).await? {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "invalid Trojan credential",
                    )
                    .into());
                }
                return Ok((destination, request));
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {}
            Err(e) => return Err(e.into()),
        }
        let data = hunks.next().await?.ok_or("incomplete Trojan request")?;
        request.extend(data);
    }
}

async fn send(stream: &mut SendStream<Bytes>, mut data: Bytes) -> Result {
    stream.reserve_capacity(data.len());
    while !data.is_empty() {
        let available = if stream.capacity() == 0 {
            poll_fn(|cx| stream.poll_capacity(cx))
                .await
                .ok_or("HTTP/2 stream closed")??
        } else {
            stream.capacity()
        };
        let count = available.min(data.len());
        if count > 0 {
            stream.send_data(data.split_to(count), false)?;
        }
    }
    Ok(())
}

async fn transfer(
    mut hunks: Hunks,
    accounts: &Accounts,
    proxy: SocketAddr,
    reply: &mut SendStream<Bytes>,
) -> Result {
    let (target, initial) =
        timeout(Duration::from_secs(12), handshake(&mut hunks, accounts)).await??;
    let upstream = timeout(
        Duration::from_secs(12),
        socks::connect(proxy, &target.address, target.port),
    )
    .await??;
    let (mut reader, mut writer) = upstream.into_split();
    let send_out = async {
        writer.write_all(&initial[target.payload..]).await?;
        while let Some(data) = hunks.next().await? {
            writer.write_all(&data).await?;
        }
        writer.shutdown().await?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    };
    let send_back = async {
        let mut data = [0; 16 * 1024];
        loop {
            let n = reader.read(&mut data).await?;
            if n == 0 {
                break;
            }
            send(reply, frame(&data[..n])).await?;
        }
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    };
    tokio::try_join!(send_out, send_back)?;
    Ok(())
}

async fn session(
    request: Request<RecvStream>,
    mut response: SendResponse<Bytes>,
    accounts: &Accounts,
    proxy: SocketAddr,
    path: &str,
) -> Result {
    if request.method() != Method::POST || request.uri().path() != path {
        response.send_response(
            Response::builder().status(StatusCode::NOT_FOUND).body(())?,
            true,
        )?;
        return Ok(());
    }
    let mut reply = response.send_response(
        Response::builder()
            .header("content-type", "application/grpc")
            .body(())?,
        false,
    )?;
    let request = Hunks {
        stream: request.into_body(),
        buffer: BytesMut::new(),
    };
    let result = transfer(request, accounts, proxy, &mut reply).await;
    if let Err(error) = &result {
        if std::env::var_os("GARLIC_GATEWAY_DIAGNOSTICS").is_some() {
            eprintln!("Trojan gRPC stream ended: {error}");
        }
    }
    let mut trailers = HeaderMap::new();
    trailers.insert(
        "grpc-status",
        if result.is_ok() { "0" } else { "13" }.parse()?,
    );
    reply.send_trailers(trailers)?;
    Ok(())
}

pub async fn listen(settings: Option<Settings>, proxy: SocketAddr) -> Result {
    let Some(settings) = settings else {
        return std::future::pending().await;
    };
    let path: Arc<str> = format!("/{}/Tun", settings.service_name).into();
    let listener = TcpListener::bind(settings.listen).await?;
    let accounts = if let Some(source) = settings.sync {
        let previous = match read_directory(&settings.accounts_file).await {
            Ok(Directory::Grant(grant)) => Some(grant),
            _ => None,
        };
        let cache = Arc::new(RwLock::new(previous));
        tokio::spawn(sync::run(source, settings.accounts_file, cache.clone()));
        Accounts::Synchronized(cache)
    } else {
        Accounts::Local(settings.accounts_file)
    };
    let accounts = Arc::new(accounts);
    loop {
        let (peer, _) = listener.accept().await?;
        let path = path.clone();
        let accounts = accounts.clone();
        tokio::spawn(async move {
            let _ = async {
                peer.set_nodelay(true)?;
                let mut connection = h2::server::handshake(peer).await?;
                while let Some(request) = connection.accept().await {
                    let (request, response) = request?;
                    let path = path.clone();
                    let accounts = accounts.clone();
                    tokio::spawn(async move {
                        let _ = session(request, response, &accounts, proxy, &path).await;
                    });
                }
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            }
            .await;
        });
    }
}
