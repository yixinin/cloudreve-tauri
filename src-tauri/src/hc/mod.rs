pub mod http_client_manager;
pub mod iroh_client;
pub mod quinn_endpoint;
pub mod std;
pub mod url;

use ::std::str::FromStr;

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

pub struct Request<T> {
    method: Method,
    url: String,
    headers: http::HeaderMap,
    body: Option<T>,
}

impl<T> Request<T> {
    pub fn new(method: Method, url: String) -> Self {
        Self {
            method,
            url,
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

    pub fn with_body(mut self, body: T) -> Self {
        self.body = Some(body);
        self
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

    pub fn body(&self) -> Option<&T> {
        self.body.as_ref()
    }
}

pub trait HttpClient {
    async fn get<T>(self, req: Request<()>) -> anyhow::Result<Response<T>>
    where
        T: DeserializeOwned;

    async fn head<R>(self, req: Request<R>) -> anyhow::Result<Response<()>>
    where
        R: serde::Serialize;
    async fn post<R, T>(self, req: Request<R>) -> anyhow::Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned;
    async fn put<R, T>(self, req: Request<R>) -> anyhow::Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned;
    async fn delete<T>(self, req: Request<()>) -> anyhow::Result<Response<T>>
    where
        T: DeserializeOwned;
}
