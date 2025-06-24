use hyper_util::client::legacy::connect::dns::GaiResolver as HyperGaiResolver;
use tower_service::Service;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::error::BoxError;

use crate::rendezvouser;

#[derive(Debug)]
pub struct P2PResolver {
    schema: String,
    uri: String,
    local_port: u16,
}

impl P2PResolver {
    pub fn new(schema: &str, uri: &str, local_port: u16) -> Self {
        Self {
            schema: schema.to_string(),
            uri: uri.to_string(),
            local_port,
        }
    }
}

impl Resolve for P2PResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            // {https://}{A.B.C}{/api/v4/p2p/signal}
            let url = format!("{}{}{}", self.schema, name.as_str(), self.uri);
            let client = rendezvouser::SignalClient::new(&url, stun_addrs);
            let (local_addr, pub_addr, remote_addr) =
                client.get_remote_addr(Some(self.local_port)).await?;
            rendezvouser::simple_udp_hole_punching(local_addr, remote_addr).await;
            Ok((vec![remote_addr]))
        })
    }
}
