use anyhow::Result;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use reqwest::Client;

use crate::rendezvouser;

static H3_ADDR: OnceLock<Arc<HashSet<String>>> = OnceLock::new();

pub async fn get_client(addr: &str) -> Result<(Client, String)> {
    // if let Some(remote_addr) = H3_ADDR.get() {
    //     let signal_cli =
    //         rendezvouser::SignalClient::new(&format!("{}/api/v4/p2p/signal", addr), None);
    //     if !remote_addr.is_empty() {
    //         let (local_addr, _) = signal_cli.get_local_pub_addr().await?;
    //         let buider = reqwest::ClientBuilder::new()
    //             .local_address(Some(local_addr.ip()))
    //             .http3_local_port(local_addr.port())
    //             .http3_prior_knowledge()
    //             .http3_keep_alive_interval(Duration::from_secs(15))
    //             .tls_early_data(true)
    //             .http3_max_idle_timeout(Duration::from_secs(3600))
    //             .danger_accept_invalid_certs(true);
    //         let client = buider.build()?;
    //         return Ok((client, format!("https://{}", remote_addr.to_string())));
    //     }
    // }

    let signal_cli = rendezvouser::SignalClient::new(&format!("{}/api/v4/p2p/signal", addr), None);
    let (local_addr, _, remote_addr) = signal_cli.get_remote_addr().await?;

    let _ = rendezvouser::simple_udp_hole_punching(local_addr, remote_addr).await;

    let buider = reqwest::ClientBuilder::new()
        .local_address(Some(local_addr.ip()))
        .http3_local_port(local_addr.port())
        .http3_prior_knowledge()
        .http3_keep_alive_interval(Duration::from_secs(15))
        .tls_early_data(true)
        .http3_max_idle_timeout(Duration::from_secs(3600))
        .danger_accept_invalid_certs(true);
    let client = buider.build()?;

    // let remote_addr = H3_ADDR.get_or_init(|| return Arc::new(remote_addr.to_string()));

    Ok((client, format!("https://{}", remote_addr.to_string())))
}
