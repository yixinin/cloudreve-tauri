use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use crate::rendezvouser;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct P2PResolver {
    schema: String,
    uri: String,
    local_port: u16,
    stun_addrs: Option<Vec<String>>,
}

impl P2PResolver {
    pub fn new(schema: &str, uri: &str, local_port: u16, stun_addrs: Option<Vec<String>>) -> Self {
        Self {
            schema: schema.to_string(),
            uri: uri.to_string(),
            local_port,
            stun_addrs,
        }
    }
}

impl Resolve for P2PResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let local_port = self.local_port;
        let stun_addrs = self.stun_addrs.clone();
        let schema = self.schema.clone();
        let uri = self.uri.clone();
        Box::pin(async move {
            // {https://}{A.B.C}{/api/v4/p2p/signal}
            let url = format!("{}{}{}", schema, name.as_str(), uri);
            let client = rendezvouser::SignalClient::new(&url, stun_addrs);
            let (local_addr, _, remote_addr) = client.get_remote_addr(Some(local_port)).await?;
            rendezvouser::simple_udp_hole_punching(local_addr, remote_addr).await;
            let addrs: Addrs = Box::new(SocketAddrs {
                iter: vec![remote_addr].into_iter(),
            });
            Ok(addrs)
        })
    }
}

pub struct SocketAddrs {
    iter: std::vec::IntoIter<SocketAddr>,
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
