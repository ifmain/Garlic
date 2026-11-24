use std::{io, net::SocketAddr};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn address(reader: &mut (impl AsyncRead + Unpin), kind: u8) -> io::Result<Vec<u8>> {
    let mut wire = vec![kind];
    let length = match kind {
        1 => 4,
        3 => {
            let length = reader.read_u8().await?;
            if length == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "empty host name",
                ));
            }
            wire.push(length);
            usize::from(length)
        }
        4 => 16,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown address type",
            ))
        }
    };
    let start = wire.len();
    wire.resize(start + length, 0);
    reader.read_exact(&mut wire[start..]).await?;
    Ok(wire)
}

pub async fn connect(proxy: SocketAddr, target: &[u8], port: u16) -> io::Result<TcpStream> {
    let mut stream = TcpStream::connect(proxy).await?;
    stream.set_nodelay(true)?;
    stream.write_all(&[5, 1, 0]).await?;
    let mut authentication = [0; 2];
    stream.read_exact(&mut authentication).await?;
    if authentication != [5, 0] {
        return Err(io::Error::other("SOCKS authentication failed"));
    }
    let mut request = vec![5, 1, 0];
    request.extend(target);
    request.extend(port.to_be_bytes());
    stream.write_all(&request).await?;
    let mut reply = [0; 4];
    stream.read_exact(&mut reply).await?;
    if reply[..3] != [5, 0, 0] {
        return Err(io::Error::other("SOCKS CONNECT failed"));
    }
    address(&mut stream, reply[3]).await?;
    stream.read_u16().await?;
    Ok(stream)
}
