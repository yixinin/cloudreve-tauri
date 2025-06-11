use std::{net::TcpStream, time::Duration};

pub fn has_ipv6_connectivity() -> bool {
    let test_targets = [
        // 全球性公共服务的IPv6地址
        "[2606:4700:4700::1111]:80", // Cloudflare DNS
        "[2001:4860:4860::8888]:80", // Google DNS
    ];

    for addr in &test_targets {
        if TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_secs(3), // 3秒超时
        )
        .is_ok()
        {
            return true;
        }
    }
    false
}
