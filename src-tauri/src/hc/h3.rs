use anyhow::Result;
use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use reqwest::Client;

use crate::rendezvouser;

static H3_CLIENT: OnceLock<Arc<Client>> = OnceLock::new();
static H3_ADDR: OnceLock<Arc<String>> = OnceLock::new();

pub async fn get_client(addr: &str) -> Result<(Arc<Client>, Arc<String>)> {
    if let Some(client) = H3_CLIENT.get() {
        let client = client.clone().to_owned();
        if let Some(addr) = H3_ADDR.get() {
            return Ok((client, addr.clone()));
        }
    }

    let signal_cli = rendezvouser::SignalClient::new(&format!("{}/api/v4/p2p/signal", addr), None);
    let (local_addr, _, remote_addr) = signal_cli.get().await?;

    let _ = rendezvouser::udp_hole_punching(local_addr, remote_addr).await;

    let buider = reqwest::ClientBuilder::new()
        .local_address(Some(local_addr.ip()))
        .quic_local_port(local_addr.port())
        .http3_prior_knowledge()
        .quic_keep_alive_interval(Duration::from_secs(15))
        .tls_early_data(true)
        .http3_max_idle_timeout(Duration::from_secs(3600))
        .danger_accept_invalid_certs(true);
    let client = Arc::new(buider.build()?);

    let remote_addr = format!("https://{}", remote_addr.to_string());
    let client = H3_CLIENT.get_or_init(|| return client);
    let remote_addr = H3_ADDR.get_or_init(|| return Arc::new(remote_addr));

    Ok((client.clone(), remote_addr.clone()))
}
