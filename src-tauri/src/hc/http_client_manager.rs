use anyhow::{anyhow, Result};
use enum_dispatch::enum_dispatch;
use iroh::{Endpoint, EndpointAddr};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::hc::iroh_client::{self, IrohClient};
use crate::hc::std::ReqwestClient;
use crate::hc::{HttpClient, Request, Response};

// 客户端类型枚举
#[derive(Debug, Clone, Copy)]
pub enum ClientType {
    Iroh,
    Reqwest,
}

// 使用enum_dispatch宏使枚举能够分发到具体实现
#[enum_dispatch]
pub trait HttpClientWrapper: Send + Sync {
    async fn get<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: serde::de::DeserializeOwned;

    async fn head<R>(&self, req: Request<R>) -> Result<Response<()>>
    where
        R: serde::Serialize;

    async fn post<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned;

    async fn put<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned;

    async fn delete<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: serde::de::DeserializeOwned;
}

// 为IrohClient实现包装器
#[derive(Clone)]
pub struct IrohClientWrapper(Arc<Mutex<IrohClient>>);

impl HttpClientWrapper for IrohClientWrapper {
    async fn get<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        // 获取锁并克隆IrohClient，然后调用其get方法
        let client = self.0.lock().await.clone();
        client.get(req).await
    }

    async fn head<R>(&self, req: Request<R>) -> Result<Response<()>>
    where
        R: serde::Serialize,
    {
        // 获取锁并克隆IrohClient，然后调用其head方法
        let client = self.0.lock().await.clone();
        client.head(req).await
    }

    async fn post<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        // 获取锁并克隆IrohClient，然后调用其post方法
        let client = self.0.lock().await.clone();
        client.post(req).await
    }

    async fn put<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        // 获取锁并克隆IrohClient，然后调用其put方法
        let client = self.0.lock().await.clone();
        client.put(req).await
    }

    async fn delete<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        // 获取锁并克隆IrohClient，然后调用其delete方法
        let client = self.0.lock().await.clone();
        client.delete(req).await
    }
}

// 为ReqwestClient实现包装器
#[derive(Clone)]
pub struct ReqwestClientWrapper(ReqwestClient);

impl HttpClientWrapper for ReqwestClientWrapper {
    async fn get<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let client = self.0.clone();
        client.get(req).await
    }

    async fn head<R>(&self, req: Request<R>) -> Result<Response<()>>
    where
        R: serde::Serialize,
    {
        let client = self.0.clone();
        client.head(req).await
    }

    async fn post<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let client = self.0.clone();
        client.post(req).await
    }

    async fn put<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        let client = self.0.clone();
        client.put(req).await
    }

    async fn delete<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let client = self.0.clone();
        client.delete(req).await
    }
}

// 分发枚举，用于存储不同类型的客户端
#[enum_dispatch(HttpClientWrapper)]
#[derive(Clone)]
pub enum HttpClientDispatcher {
    Iroh(IrohClientWrapper),
    Reqwest(ReqwestClientWrapper),
}

// 客户端管理器，负责创建和管理不同类型的HTTP客户端
#[derive(Clone)]
pub struct HttpClientManager {
    iroh_client: Option<IrohClient>,
}

impl HttpClientManager {
    // 创建一个新的客户端管理器
    pub fn new() -> Self {
        Self { iroh_client: None }
    }

    // 初始化Iroh端点（程序启动时调用）
    pub fn init_iroh_endpoint(&mut self, addr: EndpointAddr) -> Result<()> {
        self.iroh_client = Some(iroh_client::IrohClient::new(addr));
        Ok(())
    }

    // 获取指定类型的客户端
    pub async fn get_client(&self, client_type: ClientType) -> Result<HttpClientDispatcher> {
        match client_type {
            ClientType::Iroh => {
                if let Some(iroh_client) = self.iroh_client.clone() {
                    let wrapper = IrohClientWrapper(Arc::new(Mutex::new(iroh_client)));
                    return Ok(HttpClientDispatcher::Iroh(wrapper));
                }
                return Err(anyhow!("Iroh client not initialized"));
            }
            ClientType::Reqwest => {
                // 获取Reqwest客户端
                let reqwest_client = ReqwestClient::new()?;
                let wrapper = ReqwestClientWrapper(reqwest_client);
                Ok(HttpClientDispatcher::Reqwest(wrapper))
            }
        }
    }
}
