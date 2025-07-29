use anyhow::Result;
use std::net::IpAddr;
use std::{sync::Arc, time::Duration};

use reqwest::Client;

use crate::hc::pool::HttpClient;
use crate::rendezvouser;

pub async fn get_client(addr: &str) -> Result<HttpClient> {
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

    let signal_cli = rendezvouser::SignalClient::new(&signal_url, None);
    let (local_addr, pub_addr, remote_addr) = signal_cli.get_remote_addr(None).await?;
    println!("{} -> {} -> {}", local_addr, pub_addr, remote_addr);

    let dns_resolver = super::p2p_dns::P2PResolver::new(remote_addr);
    let builder = reqwest::ClientBuilder::new()
        .http3_prior_knowledge()
        .http3_keep_alive_interval(Duration::from_secs(15))
        .http3_max_idle_timeout(Duration::from_secs(3600))
        .local_address(local_addr.ip())
        .http3_local_port(local_addr.port())
        .use_rustls_tls()
        .timeout(Duration::from_secs(10))
        .dns_resolver(Arc::new(dns_resolver))
        .danger_accept_invalid_certs(true);

    match builder.build() {
        Ok(client) => return Ok(HttpClient::new(client, Some(remote_addr))),
        Err(e) => {
            println!("build error: {}", e);
            return Err(anyhow::format_err!("build error: {}", e));
        }
    }
}
