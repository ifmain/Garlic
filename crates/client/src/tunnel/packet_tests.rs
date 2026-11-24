use super::*;
use etherparse::PacketBuilder;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::{TcpListener, UdpSocket};

/// A TUN read returns one IP datagram. Stream sockets can merge successive
/// writes, which would incorrectly drop a packet at the IP-stack boundary.
struct PacketDevice(UdpSocket);
impl AsyncRead for PacketDevice {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.0.poll_recv(cx, buf)
    }
}
impl AsyncWrite for PacketDevice {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.0.poll_send(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
async fn next_packet(wire: &UdpSocket, step: &str) -> Vec<u8> {
    let mut data = vec![0; 1500];
    let size = tokio::time::timeout(std::time::Duration::from_secs(5), wire.recv(&mut data))
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {step}"))
        .unwrap();
    data.truncate(size);
    assert!(size >= 20 && data[0] >> 4 == 4);
    assert_eq!(u16::from_be_bytes([data[2], data[3]]) as usize, size);
    data
}
async fn emit_tcp(
    wire: &UdpSocket,
    server: [u8; 4],
    seq: u32,
    ack: Option<u32>,
    syn: bool,
    fin: bool,
    data: &[u8],
) {
    let mut builder =
        PacketBuilder::ipv4([10, 254, 254, 1], server, 64).tcp(49000, 443, seq, 65535);
    if syn {
        builder = builder.syn();
    }
    if let Some(a) = ack {
        builder = builder.ack(a);
    }
    if fin {
        builder = builder.fin();
    }
    if !data.is_empty() {
        builder = builder.psh();
    }
    let mut pkt = Vec::new();
    builder.write(&mut pkt, data).unwrap();
    assert_eq!(wire.send(&pkt).await.unwrap(), pkt.len());
}
async fn socks_echo(listener: TcpListener) {
    let (mut tcp, _) = listener.accept().await.unwrap();
    assert_eq!(tcp.read_u8().await.unwrap(), 5);
    let n = tcp.read_u8().await.unwrap();
    let mut methods = vec![0; n as usize];
    tcp.read_exact(&mut methods).await.unwrap();
    assert!(methods.contains(&0));
    tcp.write_all(&[5, 0]).await.unwrap();
    let mut prefix = [0; 4];
    tcp.read_exact(&mut prefix).await.unwrap();
    assert_eq!(prefix, [5, 1, 0, 3]);
    let n = tcp.read_u8().await.unwrap();
    let mut host = vec![0; n as usize];
    tcp.read_exact(&mut host).await.unwrap();
    assert_eq!(host, b"garlic.example");
    assert_eq!(tcp.read_u16().await.unwrap(), 443);
    tcp.write_all(&[5, 0, 0, 1, 127, 0, 0, 1, 1, 1])
        .await
        .unwrap();
    let (mut r, mut w) = tcp.into_split();
    tokio::io::copy(&mut r, &mut w).await.unwrap();
    w.shutdown().await.unwrap();
}
#[test]
fn virtual_dns_and_tcp_use_the_loopback_socks_server() {
    runtime().unwrap().block_on(async{
        let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();
        let opts=options(listener.local_addr().unwrap()).unwrap();
        let socks=tokio::spawn(socks_echo(listener));
        let device=UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let wire=UdpSocket::bind("127.0.0.1:0").await.unwrap();
        device.connect(wire.local_addr().unwrap()).await.unwrap();
        wire.connect(device.local_addr().unwrap()).await.unwrap();
        let device=PacketDevice(device);
        let stop=CancellationToken::new();
        let runner=tokio::spawn(tun2proxy::run(device,1500,opts,stop.clone()));
        let exercise=async{
            let mut query=Vec::new();
            PacketBuilder::ipv4([10,254,254,1],[1,1,1,1],64).udp(49001,53)
                .write(&mut query,b"\x26\x12\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x06garlic\x07example\x00\x00\x01\x00\x01").unwrap();
            assert_eq!(wire.send(&query).await.unwrap(),query.len());
            let answer=next_packet(&wire,"DNS answer").await;
            assert_eq!(answer[9],17);assert_eq!(&answer[28..30],&[0x26,0x12]);
            assert!(answer[30]&0x80!=0);assert_eq!(answer[31]&15,0);assert_eq!(&answer[34..36],&[0,1]);
            let server:[u8;4]=answer[answer.len()-4..].try_into().unwrap();
            let seq=1000;
            emit_tcp(&wire,server,seq,None,true,false,&[]).await;
            let handshake=next_packet(&wire,"SYN-ACK").await;assert_eq!(handshake[9],6);
            assert_eq!(handshake[33]&0x12,0x12);
            let mut ack=u32::from_be_bytes(handshake[24..28].try_into().unwrap()).wrapping_add(1);
            let message=b"Garlic virtual interface test";
            emit_tcp(&wire,server,seq+1,Some(ack),false,false,&[]).await;
            emit_tcp(&wire,server,seq+1,Some(ack),false,false,message).await;
            let mut received=Vec::new();let end=seq+1+message.len() as u32;
            while received.len()<message.len(){
                let pkt=next_packet(&wire,"echo").await;let body=20+(pkt[32]>>4) as usize*4;
                if pkt.len()>body{
                    assert_eq!(u32::from_be_bytes(pkt[24..28].try_into().unwrap()),ack);
                    received.extend_from_slice(&pkt[body..]);ack=ack.wrapping_add((pkt.len()-body) as u32);
                    emit_tcp(&wire,server,end,Some(ack),false,false,&[]).await;
                }
            }
            assert_eq!(received,message);
            emit_tcp(&wire,server,end,Some(ack),false,true,&[]).await;
            loop{
                let pkt=next_packet(&wire,"TCP FIN").await;
                if pkt[33]&1!=0{emit_tcp(&wire,server,end+1,Some(ack.wrapping_add(1)),false,false,&[]).await;break;}
            }
        };
        let result=tokio::time::timeout(std::time::Duration::from_secs(15),exercise).await;
        stop.cancel();runner.await.unwrap().unwrap();result.unwrap();socks.await.unwrap();
    });
}
