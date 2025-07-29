use reqwest::Client;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::hc::h3;

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    addr: Option<String>,
}
impl HttpClient {
    pub fn new(client: reqwest::Client, addr: Option<SocketAddr>) -> Self {
        if let Some(addr) = addr {
            Self {
                client: client,
                addr: Some(addr.to_string()),
            }
        } else {
            Self {
                client: client,
                addr: None,
            }
        }
    }
    pub fn get_addr(&self) -> Option<String> {
        return self.addr.clone();
    }
    pub fn request<U: reqwest::IntoUrl>(
        &self,
        method: reqwest::Method,
        url: U,
    ) -> reqwest::RequestBuilder {
        self.client.request(method, url)
    }
    pub fn post<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::POST, url)
    }
    pub fn get<U: reqwest::IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.request(reqwest::Method::GET, url)
    }
}
#[derive(Debug)]
pub struct ClientPool {
    pool: Arc<Mutex<VecDeque<HttpClient>>>,
    max_size: usize,
}

impl ClientPool {
    /// 创建新对象池
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }
    /// 从池中获取 Client（若池空则新建）
    pub fn get(&self, p2p: bool) -> Result<HttpClient> {
        let mut pool = self.pool.lock().unwrap();
        if let Some(client) = pool.pop_front() {
            return Ok(client);
        } else {
            if !p2p {
                if let Ok(client) = Client::builder()
                    .pool_max_idle_per_host(20) // 优化连接复用[1,6](@ref)
                    .timeout(std::time::Duration::from_secs(10))
                    .tcp_keepalive(std::time::Duration::from_secs(60))
                    .build()
                {
                    return Ok(HttpClient {
                        client: client,
                        addr: None,
                    });
                }
            }
        }
        Err(anyhow::anyhow!("cannot create p2p client"))
    }

    /// 归还 Client 到池中
    pub fn put(&self, client: HttpClient) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(client);
        }
    }
}
