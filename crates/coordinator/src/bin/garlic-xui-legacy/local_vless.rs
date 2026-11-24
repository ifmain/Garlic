use super::{socks, Result};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use subtle::ConstantTimeEq;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Route {
    listen: SocketAddr,
    pub garlic_proxy: SocketAddr,
    local_vless_id: Option<Uuid>,
}

async fn destination(peer: &mut TcpStream, route: &Route) -> Result<(Vec<u8>, u16)> {
    if peer.read_u8().await? != 0 {
        return Err("unsupported VLESS version".into());
    }
    let mut identity = [0; 16];
    peer.read_exact(&mut identity).await?;
    let expected = route.local_vless_id.ok_or("VLESS identity is not configured")?;
    if identity.ct_eq(expected.as_bytes()).unwrap_u8() != 1 {
        return Err("VLESS authentication failed".into());
    }
    let addons = usize::from(peer.read_u8().await?);
    peer.read_exact(&mut [0; 255][..addons]).await?;
    if peer.read_u8().await? != 1 {
        return Err("unsupported VLESS command".into());
    }
    let port = peer.read_u16().await?;
    let kind = match peer.read_u8().await? {
        1 => 1,
        2 => 3,
        _ => return Err("unknown VLESS address type".into()),
    };
    Ok((socks::address(peer, kind).await?, port))
}

pub async fn listen(route: Route) -> Result {
    let listener = TcpListener::bind(route.listen).await?;
    let shared = Arc::new(route);
    loop {
        let (mut peer, _) = listener.accept().await?;
        let route = shared.clone();
        tokio::spawn(async move {
            let _ = async {
                peer.set_nodelay(true)?;
                let (address, port) =
                    timeout(Duration::from_secs(10), destination(&mut peer, &route)).await??;
                let mut upstream = timeout(
                    Duration::from_secs(10),
                    socks::connect(route.garlic_proxy, &address, port),
                )
                .await??;
                peer.write_all(&[0, 0]).await?;
                tokio::io::copy_bidirectional(&mut peer, &mut upstream).await?;
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            }
            .await;
        });
    }
}
