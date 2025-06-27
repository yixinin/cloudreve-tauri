use arc_swap::ArcSwap;
use reqwest::Client;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

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
    pub fn get(&self, addr: &str, p2p: bool) -> Option<Client> {
        let mut pool = self.pool.lock().unwrap();
        if let Some(client) = pool.pop_front() {
            Some(client)
        } else {
            Self::create_client(addr, p2p)
        }
    }

    /// 归还 Client 到池中
    pub fn put(&self, client: Client) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(client);
        }
        // 若池满，Client 会被自动丢弃（触发连接关闭）
    }

    /// 创建新 Client（复用配置）
    fn create_client(addr: &str, p2p: bool) -> Option<Client> {
        if !p2p {
            return Client::builder()
                .pool_max_idle_per_host(20) // 优化连接复用[1,6](@ref)
                .timeout(std::time::Duration::from_secs(10))
                .tcp_keepalive(std::time::Duration::from_secs(60))
                .build()
                .ok();
        }
        return h3::get_client(addr).ok();
    }
}

/// 全局静态对象池（线程安全）
lazy_static::lazy_static! {
    static ref CLIENT_POOL: ArcSwap<ClientPool> =
        ArcSwap::from(Arc::new(ClientPool::new(20)));  // 默认池大小20
}

/// 从全局池获取 Client
pub fn acquire_client(addr: &str, p2p: bool) -> Option<Client> {
    CLIENT_POOL.load().get(addr, p2p)
}

/// 归还 Client 到全局池
pub fn release_client(client: Client) {
    CLIENT_POOL.load().put(client);
}
