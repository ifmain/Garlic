//! A local echo service and a byte-for-byte test of the Garlic CONNECT proxy.
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::time::Duration;

fn echo(socket: TcpStream) -> io::Result<()> {
    let mut writer = socket.try_clone()?;
    io::copy(&mut &socket, &mut writer)?;
    writer.shutdown(Shutdown::Write)
}

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let listen = match args.as_slice() {
        [arg] if arg == "--listen" => Some("127.0.0.1"),
        [arg] if arg == "--listen-device" => Some("192.0.2.40"),
        [arg, address] if arg == "--listen-device" => Some(address.as_str()),
        _ => None,
    };
    if let Some(address) = listen {
        for socket in TcpListener::bind((address, 443))?.incoming() {
            let socket = socket?;
            if args[0] == "--listen-device" {
                let peer = socket.peer_addr()?.ip();
                // The remote test OS can reach this echo only through its relay.
                if !peer.is_loopback() && peer != socket.local_addr()?.ip() {
                    continue;
                }
            }
            std::thread::spawn(move || echo(socket));
        }
        return Ok(());
    }
    let (port, interrupt) = match args.as_slice() {
        [port] => (port, None),
        [port, flag, path] if flag == "--interrupt" => (port, Some(std::path::Path::new(path))),
        _ => {
            return Err(io::Error::other(
                "usage: probe_proxy --listen | --listen-device [FIXTURE_ADDRESS] | LOCAL_PROXY_PORT [--interrupt SIGNAL_FILE]",
            ))
        }
    };
    let port: u16 = port.parse().map_err(io::Error::other)?;
    let mut socket = TcpStream::connect(("127.0.0.1", port))?;
    socket.set_read_timeout(Some(Duration::from_secs(25)))?;
    socket.set_write_timeout(Some(Duration::from_secs(25)))?;
    socket.write_all(b"CONNECT 127.0.0.1:443 HTTP/1.1\r\nHost: 127.0.0.1:443\r\n\r\n")?;
    let mut reader = BufReader::new(socket.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if !line.starts_with("HTTP/1.1 200 ") {
        return Err(io::Error::other(line));
    }
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Err(io::Error::other("truncated headers"));
        }
        if line == "\r\n" {
            break;
        }
    }
    for size in [1, 43, 17_001, 64_000, 250_000] {
        let mut input = vec![0; size];
        getrandom::fill(&mut input).map_err(|_| io::Error::other("fixture RNG failed"))?;
        socket.write_all(&input)?;
        let mut got = vec![0; size];
        reader.read_exact(&mut got)?;
        if got != input {
            return Err(io::Error::other("echo contents changed"));
        }
        println!("echo matched {size} bytes");
        if size == 64_000 {
            if let Some(signal) = interrupt {
                std::fs::write(signal, b"interrupt active relay")?;
                let deadline = std::time::Instant::now() + Duration::from_secs(5);
                while !signal.with_extension("done").is_file() {
                    if std::time::Instant::now() >= deadline {
                        return Err(io::Error::other("fixture failed to interrupt relay"));
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                println!("continuing the same client TCP connection after relay interruption");
            }
        }
    }
    socket.shutdown(Shutdown::Write)?;
    let mut tail = Vec::new();
    reader.read_to_end(&mut tail)?;
    if !tail.is_empty() {
        return Err(io::Error::other("unexpected data after half-close"));
    }
    println!("half-close completed");
    Ok(())
}
