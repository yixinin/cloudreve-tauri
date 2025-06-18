use anyhow::Result;
use std::sync::{Arc, OnceLock};

use reqwest::Client;

use crate::rendezvouser;

static H3_CLIENT: OnceLock<Arc<Client>> = OnceLock::new();
static H3_ADDR: OnceLock<Arc<String>> = OnceLock::new();

pub async fn get_client(addr: &str) -> Result<(Arc<Client>, String)> {
    if let Some(client) = H3_CLIENT.get() {
        let client = client.clone().to_owned();
        let addr = H3_ADDR
            .get()
            .clone()
            .unwrap()
            .to_owned()
            .as_ref()
            .to_string();
        return Ok((client, addr));
    }

    let ice = rendezvouser::SignalClient::new(&format!("{}/api/v1/ice", addr), None);
    let (local_addr, _, remote_addr) = ice.get().await?;

    let buider = reqwest::ClientBuilder::new()
        .local_address(Some(local_addr.ip()))
        .local_port(local_addr.port())
        .http3_prior_knowledge()
        .danger_accept_invalid_certs(true);
    let client = Arc::new(buider.build()?);

    H3_CLIENT.set(client.clone());
    H3_ADDR.set(Arc::new(remote_addr.to_string()));

    Ok((client, format!("https://{}", remote_addr.to_string())))
}
