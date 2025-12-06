use std::collections::VecDeque;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use std::vec::Vec;

use anyhow::{anyhow, Result};
use bytes::Buf;
use http::Request as HttpRequest;
use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;
use iroh::endpoint::Connection;
use iroh::{EndpointAddr, SecretKey};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

use crate::hc::{quinn_endpoint, HttpClient, Request, Response};
/// global iroh endpoint
struct ConnectionPool {
    connections: Mutex<VecDeque<(Connection, Instant)>>,
    addr: EndpointAddr,
    max_idle_time: Duration,
}

impl ConnectionPool {
    fn new(addr: EndpointAddr) -> Self {
        Self {
            connections: Mutex::new(VecDeque::new()),
            addr,
            max_idle_time: Duration::from_secs(30),
        }
    }

    async fn get_connection(&self) -> Result<Connection> {
        let mut connections = self.connections.lock().await;

        // 移除过期连接
        let now = Instant::now();
        connections.retain(|(_, created)| now.duration_since(*created) < self.max_idle_time);

        // 如果有可用连接则复用
        if let Some((conn, _)) = connections.pop_front() {
            if conn.close_reason().is_none() {
                return Ok(conn);
            }
        }

        // 释放锁后再进行异步操作
        drop(connections); // 释放锁后再进行异步连接

        let endpoint = iroh::Endpoint::builder()
            .secret_key(get_or_create_secret())
            .bind()
            .await?;

        let conn = endpoint.connect(self.addr.clone(), dumbpipe::ALPN).await?;
        Ok(conn)
    }

    async fn release_connection(&self, conn: Connection) {
        if conn.close_reason().is_none() {
            let mut connections = self.connections.lock().await;
            if connections.len() < 5 {
                connections.push_back((conn, Instant::now()));
            }
        }
    }
}

pub struct IrohClient {
    connection_pool: Arc<ConnectionPool>,
}

// 添加IrohClient的构造函数
impl Clone for IrohClient {
    fn clone(&self) -> Self {
        Self {
            connection_pool: self.connection_pool.clone(),
        }
    }
}

impl IrohClient {
    pub fn new(addr: EndpointAddr) -> Self {
        Self {
            connection_pool: Arc::new(ConnectionPool::new(addr)),
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

        println!(
            "send request to: {}, body: {:#?}",
            request.uri(),
            request.body(),
        );

        // 从连接池获取连接
        let pool_conn = self.connection_pool.get_connection().await?;
        let addr = self.connection_pool.addr.clone();

        // 打开双向流
        let (mut send, recv) = pool_conn.open_bi().await?;
        send.write_all(&dumbpipe::HANDSHAKE).await?;
        // 创建QuinnEndpoint包装器
        let stream = quinn_endpoint::QuinnEndpoint { send, recv };
        let io = TokioIo::new(stream);

        // 使用HTTP/1.1
        let (mut sender, conn) = hyper::client::conn::http1::Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(io)
            .await
            .map_err(|e| anyhow!("HTTP handshake failed: {}", e))?;

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
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        // 请求成功，将连接释放回连接池
        self.connection_pool.release_connection(pool_conn).await;

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
        self.send_request(req).await
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

fn get_or_create_secret() -> SecretKey {
    let key = SecretKey::generate(&mut rand::rng());
    let key_str = hex::encode(key.to_bytes());
    eprintln!("using secret key {key_str}");
    key
}
pub fn parse_subdomain(subdomain: &str) -> anyhow::Result<iroh::EndpointAddr> {
    // first try to parse as a endpoint id
    if let Ok(endpoint_id) = iroh::EndpointId::from_str(subdomain) {
        return Ok(iroh::EndpointAddr::new(endpoint_id));
    }
    // then try to parse as a endpoint ticket
    if let Ok(ticket) = dumbpipe::EndpointTicket::from_str(subdomain) {
        return Ok(ticket.endpoint_addr().clone());
    }
    Err(anyhow::anyhow!("invalid subdomain"))
}
