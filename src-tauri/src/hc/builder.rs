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
        if self.version == Version::HTTP_3 {
            match h3::get_client(&self.addr).await {
                Ok(client) => {
                    let url = url::get_api_url(&self.addr, path);
                    let builder = client
                        .request(method, url)
                        .version(Version::HTTP_3)
                        .header("authorization", &self.token);
                    return Ok(builder);
                }
                Err(e) => {
                    println!("build h3 client fail: {}, fall back to http", e);
                }
            }
        }

        let url = url::get_api_url(&self.addr, path);
        let builder = Client::new()
            .request(method, url)
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
        if self.version == Version::HTTP_3 {
            match h3::get_client(&self.addr).await {
                Ok((client, addr)) => {
                    let url = url::get_api_query_url(&addr, path, Some(req));
                    let builder = client
                        .request(method, url)
                        .version(Version::HTTP_3)
                        .header("authorization", &self.token);
                    return Ok(builder);
                }
                Err(e) => {
                    println!("build h3 client fail: {}, fall back to http", e);
                }
            }
        }

        let url = url::get_api_query_url(&self.addr, path, Some(req));
        let builder = Client::new()
            .request(method, url)
            .header("authorization", &self.token);
        Ok(builder)
    }
}
