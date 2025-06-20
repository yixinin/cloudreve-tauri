use reqwest::Client;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

use super::{http3, url};

/// HTTP客户端包装（记录最后活跃时间）
struct ClientWrapper {
    pub client: Client,
    pub addr: String,
    last_active: Instant,
}

impl ClientWrapper {
    pub fn get<T>(&self, path: &str, query: Option<T>) -> reqwest::RequestBuilder {
        let client = self.client.clone();
        let url = url::get_api_query_url(&self.addr, path, query);
        client.get(url)
    }
}

/// 连接池实现
pub struct ClientPool {
    signal_url: String,
    pool: Arc<Mutex<VecDeque<Arc<ClientWrapper>>>>,
    max_idle_time: Duration,
    semaphore: Semaphore, // 控制并发创建数量
}

impl ClientPool {
    /// 创建新连接池
    pub fn new(signal_url: &str, max_size: usize, max_idle_time: Duration) -> Self {
        ClientPool {
            signal_url: signal_url.to_string(),
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_idle_time,
            semaphore: Semaphore::new(max_size),
        }
    }

    /// 获取客户端（自动创建或复用）
    pub async fn get_client(&self) -> Arc<ClientWrapper> {
        // 尝试从池中获取可用连接
        if let Some(client) = self.try_get_valid_client() {
            return client;
        }

        // 无可用连接时创建新客户端
        let permit = self.semaphore.acquire().await.expect("Semaphore closed");
        let (client, addr) = http3::get_client(&self.signal_url).await?;
        let new_client = Arc::new(ClientWrapper {
            addr,
            client,
            last_active: Instant::now(),
        });
        self.pool.lock().unwrap().push_back(new_client.clone());
        permit.forget(); // 连接放回池后释放信号量
        new_client
    }

    /// 放回连接池
    pub fn put_client(&self, client: Arc<ClientWrapper>) {
        let mut pool = self.pool.lock().unwrap();
        // 检查连接是否过期
        if client.last_active.elapsed() < self.max_idle_time {
            pool.push_back(client);
        }
        // 过期连接自动丢弃（触发Drop）
    }

    /// 尝试获取有效连接
    fn try_get_valid_client(&self) -> Option<Arc<ClientWrapper>> {
        let mut pool = self.pool.lock().unwrap();
        while let Some(client) = pool.pop_front() {
            if client.last_active.elapsed() < self.max_idle_time {
                return Some(client);
            }
        }
        None
    }
}

/// 清理后台任务（独立线程运行）
fn start_cleanup_task(pool: Arc<ClientPool>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let mut pool_inner = pool.pool.lock().unwrap();
            // 移除过期连接
            pool_inner.retain(|client| client.last_active.elapsed() < pool.max_idle_time);
        }
    });
}
