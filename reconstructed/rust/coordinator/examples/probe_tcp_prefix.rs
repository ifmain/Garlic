use std::{
    env,
    io::{self, Write},
    net::{Shutdown, TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let [listen, target, mode] = args.as_slice() else {
        return Err(io::ErrorKind::InvalidInput.into());
    };
    let header: &[u8] = match mode.as_str() {
        "v1" => b"PROXY TCP4 192.0.2.31 127.0.0.1 57098 18722\r\n",
        "v1-6" => b"PROXY TCP6 2001:db8::31 ::1 57098 18722\r\n",
        "unknown" => b"PROXY UNKNOWN\r\n",
        "malformed" => b"PROXY TCP4 invalid\r\n",
        "v2" => b"\r\n\r\n\0\r\nQUIT\n\x21\x11\0\x0c\xc0\0\x02\x1f\x7f\0\0\x01\xdf\x0a\x49\x22",
        _ => return Err(io::ErrorKind::InvalidInput.into()),
    };
    let listener = TcpListener::bind(format!("127.0.0.1:{listen}"))?;
    println!("test relay listening");
    let (mut front, _) = listener.accept()?;
    let mut back = TcpStream::connect(format!("127.0.0.1:{target}"))?;
    front.set_read_timeout(Some(Duration::from_secs(10)))?;
    back.set_read_timeout(Some(Duration::from_secs(10)))?;
    back.write_all(header)?;
    let mut from_front = front.try_clone()?;
    let mut to_back = back.try_clone()?;
    let up = thread::spawn(move || {
        let _ = io::copy(&mut from_front, &mut to_back);
        let _ = to_back.shutdown(Shutdown::Write);
    });
    let _ = io::copy(&mut back, &mut front);
    let _ = front.shutdown(Shutdown::Write);
    let _ = up.join();
    Ok(())
}
