pub mod error;
pub mod file;
pub mod login;
pub mod share;
pub mod storage;

pub use error::*;

use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
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
