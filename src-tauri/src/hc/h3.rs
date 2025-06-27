use anyhow::Result;
use std::{
    collections::HashSet,
    net::UdpSocket,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use reqwest::Client;

pub fn get_client(addr: &str) -> Result<Client> {
    let uri: http::Uri = addr.parse()?;
    let schema = uri.scheme_str().unwrap_or("https://");
    // let socket = UdpSocket::bind("0.0.0.0:0")?;
    // let local_port = socket.local_addr()?.port();
    // drop(socket);
    let local_port = 5212;

    let dns_resolver =
        super::p2p_dns::P2PResolver::new(schema, "/api/v4/p2p/signal", Some(local_port), None);
    let builder = reqwest::ClientBuilder::new()
        .http3_prior_knowledge()
        .http3_keep_alive_interval(Duration::from_secs(15))
        .tls_early_data(true)
        .http3_max_idle_timeout(Duration::from_secs(3600))
        .http3_local_port(local_port)
        .dns_resolver(Arc::new(dns_resolver))
        .danger_accept_invalid_certs(true);

    Ok(builder.build()?)
}
