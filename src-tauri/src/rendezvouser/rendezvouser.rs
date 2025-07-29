use anyhow::Result;
use socket2::SockAddr;
use std::net::SocketAddr;
use tokio::{net::UdpSocket, time::Duration};

pub async fn udp_hole_punching(local_addr: SocketAddr, remote_addr: SocketAddr) -> Result<()> {
    let socket = tokio::net::UdpSocket::bind(local_addr).await?;

    let mut buf = [0u8; 1024];
    for _ in 0..5 {
        println!("send punch to remote: {}", remote_addr);
        socket.send_to(b"PUNCH", remote_addr).await?;
        for _ in 0..5 {
            if let Ok((_, raddr)) = socket.try_recv_from(&mut buf) {
                if raddr == remote_addr {
                    return Ok(());
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await
        }
    }

    Err(anyhow::format_err!("timeout"))
}

pub async fn simple_udp_hole_punching(
    local_addr: Option<SocketAddr>,
    remote_addr: SocketAddr,
) -> Result<()> {
    let local_addr = local_addr.unwrap_or("0.0.0.0:0".parse()?);
    let socket = UdpSocket::bind(local_addr).await?;
    println!("send PUNCH {} -> {}", local_addr, remote_addr);
    let _ = socket.send_to(b"PUNCH", remote_addr).await?;
    Ok(())
}
