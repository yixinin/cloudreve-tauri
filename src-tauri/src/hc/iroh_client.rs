use std::sync::OnceLock;
use std::vec::Vec;
use std::{str::FromStr, sync::Arc};

use anyhow::{anyhow, Result};
use bytes::Buf;
use http::Request as HttpRequest;
use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;
use iroh::{Endpoint, EndpointAddr};
use serde::de::DeserializeOwned;
use tokio::{self, task};

use crate::hc::{quinn_endpoint, HttpClient, Request, Response};
/// global iroh endpoint
static ENDPOINT: OnceLock<Endpoint> = OnceLock::new();
fn endpoint() -> &'static Endpoint {
    ENDPOINT.get().unwrap()
}
pub struct IrohClient {
    endpoint: Arc<Endpoint>,
    addr: EndpointAddr,
}

// 添加IrohClient的构造函数
impl Clone for IrohClient {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            addr: self.addr.clone(),
        }
    }
}

impl IrohClient {
    pub fn new(endpoint: Endpoint, addr: EndpointAddr) -> Self {
        Self {
            endpoint: Arc::new(endpoint),
            addr,
        }
    }

    async fn send_request<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned,
    {
        // 构建请求
        let mut request_builder = HttpRequest::builder()
            .uri(&req.url)
            .method(req.method.clone());

        // 添加请求头
        for (key, value) in &req.headers {
            request_builder = request_builder.header(key, value.clone());
        }

        let request = if let Some(body_data) = req.body {
            let body_bytes = serde_json::to_vec(&body_data)
                .map_err(|e| anyhow!("Failed to serialize request body: {}", e))?;

            // 只在没有设置content-type时添加默认值
            if !req.headers.contains_key("content-type") {
                request_builder = request_builder.header("content-type", "application/json");
            }

            // 转换Vec<u8>为Bytes以满足Buf trait要求
            let body_bytes = bytes::Bytes::from(body_bytes);
            request_builder.body(Full::new(body_bytes))?
        } else {
            // 使用空Bytes而不是Vec来满足Buf trait要求
            request_builder.body(Full::new(bytes::Bytes::new()))?
        };

        // 克隆必要的字段，避免移动整个self
        let endpoint = self.endpoint.clone();

        let addr = self.addr.clone();

        eprintln!("try to connect to: {:?}", addr);
        // 连接到远程端点，使用HTTP/1.1
        let conn = endpoint.connect(addr.clone(), dumbpipe::ALPN).await?;
        eprintln!("iroh client connected, start open bi");
        // 打开双向流
        let (mut send, recv) = conn.open_bi().await?;
        send.write_all(&dumbpipe::HANDSHAKE).await?;
        // 创建QuinnEndpoint包装器
        let stream = quinn_endpoint::QuinnEndpoint { send, recv };
        let io = TokioIo::new(stream);

        // 使用HTTP/1.1
        let (mut sender, conn) = hyper::client::conn::http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;

        // 在后台处理连接
        task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection to {:?} error: {}", addr, err);
            }
        });

        println!(
            "send request to: {}, body: {:#?}",
            request.uri(),
            request.body(),
        );

        // 发送请求并获取响应
        let response = sender
            .send_request(request)
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        let status = response.status();
        let headers = response.headers().clone();

        // 读取响应体
        let mut body_bytes = Vec::new();
        let mut body = response.into_body();

        // 手动读取响应体内容
        while let Some(frame) = body
            .frame()
            .await
            .transpose()
            .map_err(|e| anyhow!("Failed to read response chunk: {}", e))?
        {
            // 检查是否是数据帧
            if let Some(chunk) = frame.data_ref() {
                // 使用bytes库的Buf trait方法正确处理数据
                if chunk.has_remaining() {
                    // 使用chunk()方法获取不可变引用而不是copy_to_slice
                    body_bytes.extend_from_slice(chunk.chunk());
                    // 不需要手动advance，因为chunk()返回的是不可变引用，且我们已使用完数据
                }
            }
        }
        eprintln!("response body: {:?}", String::from_utf8_lossy(&body_bytes));
        // 反序列化响应数据
        let data = if !body_bytes.is_empty() {
            serde_json::from_slice(&body_bytes)
                .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?
        } else {
            // 对于空响应，返回适当的错误
            return Err(anyhow!("Empty response body: cannot deserialize to type T"));
        };

        Ok(Response {
            status,
            headers,
            data,
        })
    }
}

impl HttpClient for IrohClient {
    async fn get<T>(self, req: Request<()>) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        // 对于GET请求，我们可以忽略body，直接调用send_request
        // 但需要将Request<()>转换为Request<()>
        self.send_request(req).await
    }

    async fn head<R>(self, req: Request<R>) -> Result<Response<()>>
    where
        R: serde::Serialize,
    {
        let mut req_builder = http::Request::builder()
            .uri(&req.url)
            .method(req.method.clone());

        // 添加请求头
        for (key, value) in &req.headers {
            req_builder = req_builder.header(key, value.clone());
        }

        let request = req_builder.body(http_body_util::Full::new(bytes::Bytes::new()))?;

        // 克隆必要的字段，避免移动整个self
        let endpoint = self.endpoint.clone();
        let addr = self.addr.clone();
        // 连接到远程端点，使用HTTP/1.1
        let conn = endpoint.connect(addr.clone(), dumbpipe::ALPN).await?;
        // 打开双向流
        let (mut send, recv) = conn.open_bi().await?;
        send.write_all(&dumbpipe::HANDSHAKE).await?;

        // 创建QuinnEndpoint包装器
        let stream = quinn_endpoint::QuinnEndpoint { send, recv };
        let io = TokioIo::new(stream);

        // 使用HTTP/1.1
        let (mut sender, conn) = hyper::client::conn::http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await?;

        // 在后台处理连接
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection to {:?} error: {}", addr, err);
            }
        });

        // 发送请求并获取响应

        let response = sender
            .send_request(request)
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
        // 对于DELETE请求，我们可以忽略body，直接调用send_request
        self.send_request(req).await
    }
}
