use anyhow::Result;
use std::net::SocketAddr;
use tokio::time::Duration;

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
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
) -> Result<()> {
    let socket = tokio::net::UdpSocket::bind(local_addr).await?;

    let _ = socket.send_to(b"PUNCH", remote_addr).await?;
    // println!("send punch to remote: {}", remote_addr);
    Ok(())
}
