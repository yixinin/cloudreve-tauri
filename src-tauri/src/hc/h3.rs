use anyhow::Result;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};

use reqwest::Client;

static H3_ADDR: OnceLock<Arc<HashSet<String>>> = OnceLock::new();

pub async fn get_client(addr: &str) -> Result<(Client, String)> {
    let builder = reqwest::ClientBuilder::new()
        .http3_prior_knowledge()
        .http3_keep_alive_interval(Duration::from_secs(15))
        .tls_early_data(true)
        .http3_max_idle_timeout(Duration::from_secs(3600))
        .danger_accept_invalid_certs(true);
    let connector = super::quic::p2p_quinn::H3QuinnConnector::new(
        &format!("{}/api/v4/p2p/signal", addr),
        None,
        tls,
        tp,
        c_cfg,
    )?;
    let client = builder.build_h3_with_connector(connector)?;
    Ok((client, format!("https://{}", remote_addr.to_string())))
}
