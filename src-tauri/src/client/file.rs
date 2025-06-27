use crate::hc::pool;
use crate::proto::login::Token;
use anyhow::Result;
use reqwest::Client;
use reqwest::Method;

pub struct FileClient {
    token: Token,
}

impl FileClient {
    pub fn new(token: Token) -> Self {
        Self { token }
    }

    pub async fn request(&self, method: Method, url: &str) -> Result<String> {
        let client = pool::acquire_client(addr, p2p)
    }
}
