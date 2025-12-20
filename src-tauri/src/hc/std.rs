use anyhow::{anyhow, Result};
use bytes::Bytes;
use http::Method;
use serde::de::DeserializeOwned;
use std::any::TypeId;
use std::mem;

use crate::hc::{HttpClient, Request, Response};

pub struct ReqwestClient(reqwest::Client);

impl ReqwestClient {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(ReqwestClient(client))
    }
}

impl Clone for ReqwestClient {
    fn clone(&self) -> Self {
        ReqwestClient(self.0.clone())
    }
}

impl HttpClient for ReqwestClient {
    async fn get<T>(&self, req: Request) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn get_bytes(&self, req: Request) -> Result<Response<Bytes>> {
        let mut req_builder = self.0.request(req.method.clone(), &req.url);

        // 添加headers
        for (key, value) in &req.headers {
            if let Some(value_str) = value.to_str().ok() {
                req_builder = req_builder.header(key.as_str(), value_str);
            }
        }

        // 添加请求体
        if let Some(body) = req.body {
            req_builder = req_builder.body(body);
        } else if req.method != Method::GET && req.method != Method::DELETE {
            // 对于非GET/DELETE请求，设置空body
            req_builder = req_builder.body(Bytes::new());
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        let status = response.status();
        let headers = response.headers().clone();

        // 获取原始字节
        let bytes = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read response body as bytes: {}", e))?;

        Ok(Response::new(status, headers, bytes))
    }

    async fn head(&self, req: Request) -> Result<Response<()>> {
        // 对于HEAD请求，我们不关心响应体，所以可以使用泛型参数并忽略它
        let mut req_builder = self.0.request(Method::HEAD, &req.url);
        // 添加headers
        for (key, value) in &req.headers {
            if let Some(value_str) = value.to_str().ok() {
                req_builder = req_builder.header(key.as_str(), value_str);
            }
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send HEAD request: {}", e))?;

        let status = response.status();
        let headers = response.headers().clone();

        Ok(Response::new(status, headers, ()))
    }

    async fn post<T>(&self, req: Request) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn put<T>(&self, req: Request) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn delete<T>(&self, req: Request) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }
}

impl ReqwestClient {
    // 辅助方法，用于发送带或不带请求体的请求
    async fn send_request<T>(&self, req: Request) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        let mut req_builder = self.0.request(req.method.clone(), &req.url);

        // 添加headers
        for (key, value) in &req.headers {
            if let Some(value_str) = value.to_str().ok() {
                req_builder = req_builder.header(key.as_str(), value_str);
            }
        }

        // 添加请求体
        if let Some(body) = req.body {
            req_builder = req_builder.body(body);
        } else if req.method != Method::GET && req.method != Method::DELETE {
            // 对于非GET/DELETE请求，设置空body
            req_builder = req_builder.body(Bytes::new());
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        let status = response.status();
        let headers = response.headers().clone();

        // 将响应解析为JSON
        let data = response
            .json::<T>()
            .await
            .map_err(|e| anyhow!("Failed to parse response body: {}", e))?;

        Ok(Response::new(status, headers, data))
    }
}
