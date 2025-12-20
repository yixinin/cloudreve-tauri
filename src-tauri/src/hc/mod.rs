pub mod h3_iroh;
pub mod http_client_manager;
pub mod iroh_client;
pub mod quinn_endpoint;
pub mod std;
pub mod url;

use ::std::{any::TypeId, fmt::format, mem, str::FromStr};

use bytes::Bytes;
use http::Method;
use serde::de::DeserializeOwned;

pub struct Response<T> {
    status: http::StatusCode,
    headers: http::HeaderMap,
    data: T,
}

impl<T> Response<T> {
    pub fn new(status: http::StatusCode, headers: http::HeaderMap, data: T) -> Self {
        Self {
            status,
            headers,
            data,
        }
    }

    pub fn status(&self) -> http::StatusCode {
        self.status
    }

    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn into_data(self) -> T {
        self.data
    }
}

/// 为Response<T>实现的扩展trait，提供into_bytes方法
pub trait ResponseExt<T> {
    /// 尝试将响应数据转换为Vec<u8>
    /// 如果数据本身就是Vec<u8>，直接返回
    /// 否则尝试将其序列化为JSON，然后返回JSON字节
    fn into_bytes(self) -> Vec<u8>;
}

impl<T> ResponseExt<T> for Response<T>
where
    T: serde::Serialize + 'static,
{
    fn into_bytes(self) -> Vec<u8> {
        // 直接尝试序列化为JSON
        if let Ok(json_bytes) = serde_json::to_vec(&self.data) {
            json_bytes
        } else {
            // 如果序列化失败，返回空向量
            Vec::new()
        }
    }
}

pub struct Request {
    method: Method,
    url: String,
    headers: http::HeaderMap,
    body: Option<Bytes>,
}

impl Request {
    pub fn new(method: Method, base_url: &str, path: &str) -> Self {
        Self {
            method,
            url: format!("{}{}", base_url, path),
            headers: http::HeaderMap::new(),
            body: None,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        if let Ok(header_value) = http::HeaderValue::from_str(value) {
            if let Ok(header_name) = http::HeaderName::from_str(key) {
                self.headers.insert(header_name, header_value);
            }
        }
        self
    }

    pub fn json<T>(mut self, body: T) -> anyhow::Result<Self>
    where
        T: serde::Serialize,
    {
        let body = serde_json::to_vec(&body)?;
        let body = Bytes::copy_from_slice(&body);
        self.body = Some(body);
        Ok(self)
    }

    pub fn form_data<T>(mut self, body: T) -> anyhow::Result<Self>
    where
        T: serde::Serialize,
    {
        let body = serde_urlencoded::to_string(body)?;
        let body = Bytes::copy_from_slice(body.as_bytes());
        self.body = Some(body);
        Ok(self)
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    pub fn body(self) -> Option<Bytes> {
        match self.body {
            Some(body) => Some(body),
            None => None,
        }
    }
}

pub trait HttpClient {
    async fn get<T>(&self, req: Request) -> anyhow::Result<Response<T>>
    where
        T: DeserializeOwned;

    async fn get_bytes(&self, req: Request) -> anyhow::Result<Response<Bytes>>;

    async fn head(&self, req: Request) -> anyhow::Result<Response<()>>;

    async fn post<T>(&self, req: Request) -> anyhow::Result<Response<T>>
    where
        T: DeserializeOwned;
    async fn put<T>(&self, req: Request) -> anyhow::Result<Response<T>>
    where
        T: DeserializeOwned;
    async fn delete<T>(&self, req: Request) -> anyhow::Result<Response<T>>
    where
        T: DeserializeOwned;
}
