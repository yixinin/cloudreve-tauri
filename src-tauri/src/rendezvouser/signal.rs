use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, ToSocketAddrs as _};

#[derive(Debug, Serialize, Deserialize)]
pub struct AddrReq {
    pub addr: String,
}

pub struct SignalClient {
    stun_addrs: Vec<String>,
    signal_url: String,
}

impl SignalClient {
    pub fn new(signal_url: &str, stun_addrs: Option<Vec<String>>) -> Self {
        if let Some(stun_addrs) = stun_addrs {
            return SignalClient {
                signal_url: signal_url.to_string(),
                stun_addrs,
            };
        }
        let stun_addrs = vec![
            "stun.miwifi.com:3478".to_string(),
            "stun.chat.bilibili.com:3478".to_string(),
            "turn.cloudflare.com:3478".to_string(),
            "fwa.lifesizecloud.com:3478".to_string(),
            "stun.isp.net.au:3478".to_string(),
            "stun.voipbusterpro.com:3478".to_string(),
            "stun.freeswitch.org:3478".to_string(),
            "stun.nextcloud.com:3478".to_string(),
            "stun.l.google.com:19302".to_string(),
            "stun.sipnet.com:3478".to_string(),
        ];
        SignalClient {
            signal_url: signal_url.to_string(),
            stun_addrs,
        }
    }

    pub async fn get(&self) -> Result<(SocketAddr, SocketAddr, SocketAddr)> {
        for stun_addr in &self.stun_addrs {
            match self.get_pub_addr(stun_addr).await {
                Ok((local_addr, pub_addr)) => {
                    let client = reqwest::Client::new();

                    let req = AddrReq {
                        addr: pub_addr.to_string(),
                    };
                    let response: String = client
                        .post(&self.signal_url)
                        .json(&req)
                        .send()
                        .await?
                        .text()
                        .await?;

                    match response.parse() {
                        Ok(remote_addr) => {
                            return Ok((local_addr, pub_addr, remote_addr));
                        }
                        Err(e) => {
                            return Err(anyhow::format_err!("resp: {}, err: {}", response, e));
                        }
                    }
                }
                Err(e) => {
                    println!("get pub addr by {} fail: {}", stun_addr, e);
                }
            }
        }
        return Err(anyhow::format_err!("get all pub addr fail"));
    }

    pub async fn get_pub_addr(&self, stun_addr: &str) -> Result<(SocketAddr, SocketAddr)> {
        let laddr: SocketAddr = "0.0.0.0:0".parse()?;
        let socket = tokio::net::UdpSocket::bind(laddr).await?;
        let local_addr = socket.local_addr()?;
        let raddr = stun_addr.to_socket_addrs()?.next();
        if let Some(raddr) = raddr {
            let cli = stunclient::StunClient::new(raddr);
            let pub_addr = cli.query_external_address_async(&socket).await?;

            return Ok((local_addr, pub_addr));
        }
        return Err(anyhow::format_err!("invalid stun addr"));
    }
}
