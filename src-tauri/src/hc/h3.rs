use anyhow::Result;
use std::{
    collections::HashSet,
    net::UdpSocket,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tauri::utils::acl::schema;

use reqwest::Client;

pub fn get_client(addr: &str) -> Result<Client> {
    let uri: http::Uri;
    if !addr.starts_with("http") {
        uri = format!("https://{}", addr).parse()?;
    } else {
        uri = addr.parse()?;
    }

    let port = uri.port_u16().unwrap_or_default();
    let host = uri.host().unwrap_or_default();
    let schema = uri.scheme_str().unwrap_or("https");
    let signal_url: String;
    if port > 0 && port != 80 && port != 443 {
        signal_url = format!("{}://{}:{}/api/v4/p2p/signal", schema, host, port)
    } else {
        signal_url = format!("{}://{}/api/v4/p2p/signal", schema, host)
    }

    let local_port = 5212;

    let dns_resolver = super::p2p_dns::P2PResolver::new(&signal_url, local_port, None);
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
