use anyhow::{anyhow, Result};
use http::Method;
use reqwest::Proxy;
use serde::de::DeserializeOwned;

use crate::hc::{HttpClient, Request, Response};

pub struct ReqwestClient(reqwest::Client);

impl ReqwestClient {
    pub fn new() -> Result<Self> {
        Self::with_proxy(None, None, None)
    }

    pub fn with_proxy(
        proxy_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Self> {
        let mut builder = reqwest::Client::builder();

        if let Some(url) = proxy_url {
            let mut proxy =
                reqwest::Proxy::all(url).map_err(|e| anyhow!("Failed to create proxy: {}", e))?;

            if let (Some(user), Some(pass)) = (username, password) {
                proxy = proxy.basic_auth(user, pass);
            }

            builder = builder.proxy(proxy);
        }

        let client = builder
            .build()
            .map_err(|e| anyhow!("Failed to create reqwest client: {}", e))?;
        Ok(ReqwestClient(client))
    }
}

impl Clone for ReqwestClient {
    fn clone(&self) -> Self {
        ReqwestClient(self.0.clone())
    }
}

impl HttpClient for ReqwestClient {
    async fn get<T>(self, req: Request<()>) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn head<R>(self, req: Request<R>) -> Result<Response<()>>
    where
        R: serde::Serialize,
    {
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

    async fn post<R, T>(self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn put<R, T>(self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn delete<T>(self, req: Request<()>) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }
}

impl ReqwestClient {
    // 辅助方法，用于发送带或不带请求体的请求
    async fn send_request<R, T>(self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
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
            req_builder = req_builder.json(&body);
        } else if req.method != Method::GET && req.method != Method::DELETE {
            // 对于非GET/DELETE请求，设置空body
            req_builder = req_builder.json(&serde_json::json!({}));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        let status = response.status();
        let headers = response.headers().clone();

        // 解析响应体
        let data = response
            .json::<T>()
            .await
            .map_err(|e| anyhow!("Failed to parse response body: {}", e))?;

        Ok(Response::new(status, headers, data))
    }
}
