pub mod error;
pub mod file;
pub mod login;
pub mod share;
pub mod storage;

pub use error::*;

use reqwest::{Client, Method, Version};
use serde::{Deserialize, Serialize};

use crate::rendezvouser;
#[derive(Debug, Serialize, Deserialize)]
pub struct Ack<T> {
    pub code: i64,
    pub data: Option<T>,
    pub msg: String,
    pub correlation_id: Option<String>,
}

pub fn get_api_query_url<T>(addr: &str, path: &str, query: Option<T>) -> String
where
    T: Serialize,
{
    if let Some(query) = query {
        let query_str = serde_urlencoded::to_string(query).unwrap_or_default();
        return format!("{}/api/v4{}?{}", addr, path, query_str);
    }
    return format!("{}/api/v4{}", addr, path);
}

pub fn get_api_url(addr: &str, path: &str) -> String {
    return get_api_query_url::<String>(addr, path, None);
}

pub struct Site {
    pub token: String,
    pub addr: String,
}

impl Site {
    pub fn build(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        let client = Client::new();
        let url = get_api_url(&self.addr, path);
        return client
            .request(method, url)
            .header("authorization", &self.token);
    }

    pub async fn h3_build(&self, method: Method, path: &str) -> Result<reqwest::RequestBuilder> {
        let (client, addr) = get_h3_client(&self.addr).await?;
        let url = get_api_url(&addr, path);
        let builder = client
            .request(method, url)
            .version(Version::HTTP_3)
            .header("authorization", &self.token);
        Ok(builder)
    }

    pub fn build_query<T>(&self, method: Method, path: &str, req: T) -> reqwest::RequestBuilder
    where
        T: Serialize,
    {
        let client = Client::new();
        let url = get_api_query_url(&self.addr, path, Some(req));
        return client
            .request(method, url)
            .header("authorization", &self.token);
    }
}

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
    Ok((client, format!("https://{}", remote_addr.to_string())))
}
