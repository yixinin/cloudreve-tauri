use reqwest::Client;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::hc::h3;

/// 线程安全的 Client 对象池
#[derive(Debug)]
pub struct ClientPool {
    pool: Arc<Mutex<VecDeque<Client>>>,
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
    pub fn get(&self, p2p: bool) -> Result<Client> {
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
                    return Ok(client);
                }
            }
        }
        Err(anyhow::anyhow!("cannot create p2p client"))
    }

    /// 归还 Client 到池中
    pub fn put(&self, client: Client) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(client);
        }
    }
}
