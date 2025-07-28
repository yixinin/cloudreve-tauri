use futures_util::lock::Mutex;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use anyhow::Result;

use crate::rendezvouser;
use std::{net::SocketAddr, sync::Arc, vec};

#[derive(Debug, Clone)]
pub struct P2PResolver {
    remote_addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct SignalResolver {
    signal_url: String,
    local_port: u16,
    stun_addrs: Option<Vec<String>>,
    addrs: Option<Vec<SocketAddr>>,
}

impl SignalResolver {
    pub fn new(signal_url: &str, local_port: u16, stun_addrs: Option<Vec<String>>) -> Self {
        Self {
            signal_url: signal_url.to_string(),
            local_port: local_port,
            stun_addrs: stun_addrs,
            addrs: None,
        }
    }

    pub async fn lookup(&self) -> Result<Vec<SocketAddr>> {
        if let Some(addrs) = self.addrs.clone() {
            return Ok(addrs);
        }

        let client = rendezvouser::SignalClient::new(&self.signal_url, self.stun_addrs.clone());
        let (local_addr, pub_addr, remote_addr) =
            client.get_remote_addr(Some(self.local_port)).await?;
        println!(
            "send punch local: {}, pub: {} remote: {}",
            local_addr, pub_addr, remote_addr
        );
        return Ok(vec![remote_addr]);
    }
    pub async fn reset(&mut self) {
        self.addrs = None;
    }
}

impl P2PResolver {
    pub fn new(remote_addr: SocketAddr) -> Self {
        let resolver = Self { remote_addr };

        resolver
    }
}

impl Resolve for P2PResolver {
    fn resolve(&self, _: Name) -> Resolving {
        let resolver = self.clone();
        Box::pin(async move {
            let addrs = vec![resolver.remote_addr.clone()];
            let addrs: Addrs = Box::new(SocketAddrs {
                iter: addrs.clone().into_iter(),
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
