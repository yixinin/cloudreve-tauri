use std::sync::Arc;

use crate::hc::{h3, url};
use anyhow::Result;
use reqwest::{Client, Method, Version};
use serde::Serialize;
pub struct Site {
    pub token: String,
    pub addr: String,
    pub version: Version,
}

impl Site {
    pub async fn build(&self, method: Method, path: &str) -> Result<reqwest::RequestBuilder> {
        let (client, addr) = match self.version {
            Version::HTTP_3 => h3::get_client(&self.addr).await?,
            _ => (Arc::new(Client::new()), self.addr.clone()),
        };

        let url = url::get_api_url(&addr, path);
        let builder = client
            .request(method, url)
            .version(self.version)
            .header("authorization", &self.token);
        Ok(builder)
    }

    pub async fn h3_build(&self, method: Method, path: &str) -> Result<reqwest::RequestBuilder> {
        let (client, addr) = h3::get_client(&self.addr).await?;
        let url = url::get_api_url(&addr, path);
        let builder = client
            .request(method, url)
            .version(Version::HTTP_3)
            .header("authorization", &self.token);
        Ok(builder)
    }

    pub async fn build_query<T>(
        &self,
        method: Method,
        path: &str,
        req: T,
    ) -> Result<reqwest::RequestBuilder>
    where
        T: Serialize,
    {
        let (client, addr) = match self.version {
            Version::HTTP_3 => h3::get_client(&self.addr).await?,
            _ => (Arc::new(Client::new()), self.addr.clone()),
        };
        let url = url::get_api_query_url(&addr, path, Some(req));
        let builder = client
            .request(method, url)
            .header("authorization", &self.token);
        Ok(builder)
    }
}
