//! Loopback-only integration fixture. Interrupt the busiest of two test TLS
//! connections after the proxy client has verified its first payloads.
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

struct Link {
    socket: TcpStream,
    transferred: AtomicU64,
}

fn copy(mut from: &TcpStream, mut to: &TcpStream, link: &Link) {
    let mut buf = [0; 16384];
    while let Ok(n) = from.read(&mut buf) {
        if n == 0 || to.write_all(&buf[..n]).is_err() {
            break;
        }
        link.transferred.fetch_add(n as u64, Ordering::Relaxed);
    }
    let _ = to.shutdown(Shutdown::Write);
}

fn main() -> io::Result<()> {
    let signal = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or_else(|| io::Error::other("signal file required"))?,
    );
    let listener = TcpListener::bind(("127.0.0.1", 18448))?;
    let links = Arc::new(Mutex::new(HashMap::<usize, Arc<Link>>::new()));
    let watch = links.clone();
    thread::spawn(move || {
        while !signal.is_file() {
            thread::sleep(Duration::from_millis(10));
        }
        let sockets = watch.lock().unwrap();
        let busy = sockets
            .values()
            .max_by_key(|link| link.transferred.load(Ordering::Relaxed));
        if let Some(link) = busy {
            if sockets.len() >= 2 && link.transferred.load(Ordering::Relaxed) > 65_536 {
                let _ = link.socket.shutdown(Shutdown::Both);
                println!(
                    "interrupted active relay with {} open TLS connections",
                    sockets.len()
                );
                std::fs::write(signal.with_extension("done"), b"interrupted").unwrap();
            }
        }
    });
    for (id, client) in listener.incoming().enumerate() {
        let client = client?;
        let upstream = TcpStream::connect(("127.0.0.1", 18443))?;
        let link = Arc::new(Link {
            socket: client.try_clone()?,
            transferred: AtomicU64::new(0),
        });
        links.lock().unwrap().insert(id, link.clone());
        let registry = links.clone();
        thread::spawn(move || {
            thread::scope(|scope| {
                scope.spawn(|| copy(&client, &upstream, &link));
                copy(&upstream, &client, &link);
            });
            registry.lock().unwrap().remove(&id);
        });
    }
    Ok(())
}
