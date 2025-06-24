use anyhow::Result;
use socket2::SockAddr;
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

pub fn simple_udp_hole_punching(
    local_addr: Option<SocketAddr>,
    remote_addr: SocketAddr,
) -> Result<()> {
    let socket = socket2::Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    let local_addr = local_addr.unwrap_or("0.0.0.0:0".parse()?);
    socket.bind(&SockAddr::from(local_addr))?;
    let _ = socket.send_to(b"PUNCH", &SockAddr::from(remote_addr))?;
    // println!("send punch to remote: {}", remote_addr);
    Ok(())
}
