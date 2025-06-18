use std::sync::OnceLock;

use reqwest::Client;
use tokio::runtime::Runtime;

use crate::rendezvouser;

static H3_CLIENT: OnceLock<Arc<Client>> = OnceLock::new();
static H3_ADDR: OnceLock<Arc<String>> = OnceLock::new();

pub async fn get_h3_client(addr: &str) -> Result<(Client, String)> {
    let ice = rendezvouser::SignalClient::new(&format!("{}/api/v1/ice", addr), None);

    let (local_addr, pub_addr, remote_addr) = ice.get().await?;
    println!("get server addr: {}", remote_addr);
    let mut buider = reqwest::ClientBuilder::new()
        .local_address(Some(local_addr.ip()))
        .local_port(local_addr.port())
        .http3_prior_knowledge()
        .danger_accept_invalid_certs(true);
    let client = buider.build()?;
    H3_CLIENT.set(client);
    H3_ADDR.set(remote_addr.to_string());
    Ok((client, format!("https://{}", remote_addr.to_string())))
}
