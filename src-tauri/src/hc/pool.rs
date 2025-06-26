use anyhow::Result;
use reqwest::Client;
use reqwest::Version;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

/// HTTP客户端对象池
#[derive(Clone)]
pub struct HttpClientPool {
    addr: String,
    clients: Arc<Mutex<Vec<Client>>>,
    semaphore: Arc<Semaphore>, // 控制最大并发数
}

impl HttpClientPool {
    /// 创建新对象池
    /// - `max_size`: 池中最大客户端数量
    /// - `idle_connections`: 每个主机最大空闲连接数
    pub fn new(addr: &str, max_size: usize, idle_connections: usize) -> Self {
        HttpClientPool {
            addr: addr.to_string(),
            clients: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            semaphore: Arc::new(Semaphore::new(max_size)),
        }
    }

    /// 从池中获取客户端 (非阻塞)
    pub async fn get_client(&self, version: Version) -> Result<HttpClientGuard<'_>> {
        let permit = self.semaphore.acquire().await.expect("Semaphore closed");

        // 尝试从池中获取空闲客户端
        if let Some(client) = self.clients.lock().unwrap().pop() {
            return Ok(HttpClientGuard {
                client: Some(client),
                pool: &self.clone(),
                _permit: permit,
            });
        }
        let client = match version {
            Version::HTTP_3 => super::h3::get_client(&self.addr).await?,
            _ => Client::builder().build()?,
        };
        // 池为空时创建新客户端
        Ok(HttpClientGuard {
            client: Some(client),
            pool: &self.clone(),
            _permit: permit,
        })
    }

    // 内部方法：归还客户端到池
    fn return_client(&self, client: Client) {
        let mut pool = self.clients.lock().unwrap();
        if pool.len() < self.semaphore.available_permits() {
            pool.push(client);
        }
        // 若池已满则丢弃客户端
    }
}

/// 客户端守护对象，自动归还机制
pub struct HttpClientGuard<'a> {
    client: Option<Client>,
    pool: &'a HttpClientPool,
    _permit: tokio::sync::SemaphorePermit<'a>,
}

impl HttpClientGuard<'_> {
    /// 获取内部Client引用
    pub fn client(&self) -> &Client {
        self.client.as_ref().unwrap()
    }
}

// 自动归还客户端到池
impl Drop for HttpClientGuard<'_> {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            self.pool.return_client(client);
        }
    }
}
