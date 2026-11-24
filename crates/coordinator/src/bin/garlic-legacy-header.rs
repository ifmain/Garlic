use serde::Deserialize;
use std::{io, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};

#[derive(Deserialize)]
struct Config {
    listen: SocketAddr,
    target: SocketAddr,
    paths: Vec<String>,
    response: String,
}

const HEADER_END: &[u8] = b"\r\n\r\n";
const HEADER_LIMIT: usize = 8192;
const HEADER_TIMEOUT: Duration = Duration::from_secs(8);

async fn read_preface(stream: &mut TcpStream) -> io::Result<Option<(Vec<u8>, usize)>> {
    let mut input = Vec::new();
    let mut chunk = [0; 1024];
    let mut offset = 0;
    loop {
        let read = stream.read(&mut chunk).await?;
        if read == 0 {
            return Ok(None);
        }
        input.extend_from_slice(&chunk[..read]);
        if let Some(end) = input[offset..]
            .windows(HEADER_END.len())
            .position(|part| part == HEADER_END)
        {
            let payload = offset + end + HEADER_END.len();
            return Ok((payload <= HEADER_LIMIT).then_some((input, payload)));
        }
        if input.len() >= HEADER_LIMIT {
            return Ok(None);
        }
        offset = input.len().saturating_sub(HEADER_END.len() - 1);
    }
}

fn allowed(header: &[u8], paths: &[String]) -> bool {
    let Some(line_end) = header.windows(2).position(|part| part == b"\r\n") else {
        return false;
    };
    let request = &header[..line_end];
    let Some(path) = request
        .strip_prefix(b"GET ")
        .and_then(|request| request.strip_suffix(b" HTTP/1.1"))
    else {
        return false;
    };
    paths.iter().any(|allowed| path == allowed.as_bytes())
}

async fn relay(mut stream: TcpStream, config: &Config) -> io::Result<()> {
    let Ok(preface) = timeout(HEADER_TIMEOUT, read_preface(&mut stream)).await else {
        return Ok(());
    };
    let Some((input, payload)) = preface? else {
        return Ok(());
    };
    if !allowed(&input[..payload], &config.paths) {
        return Ok(());
    }
    let mut upstream = TcpStream::connect(config.target).await?;
    stream.write_all(config.response.as_bytes()).await?;
    upstream.write_all(&input[payload..]).await?;
    tokio::io::copy_bidirectional(&mut stream, &mut upstream).await?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: garlic-legacy-header FILE.json")?;
    let config: Arc<Config> = Arc::new(serde_json::from_slice(&std::fs::read(path)?)?);
    let listener = TcpListener::bind(config.listen).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let config = config.clone();
        tokio::spawn(async move {
            let _ = relay(stream, &config).await;
        });
    }
}
