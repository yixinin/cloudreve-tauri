use std::str::FromStr;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use bytes::Buf;
use http::Request as HttpRequest;
use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;
use iroh::endpoint::TransportConfig;
use iroh::{Endpoint, EndpointAddr, SecretKey};
use serde::de::DeserializeOwned;
use tokio::time::timeout;

use crate::hc::{quinn_endpoint, HttpClient, Request, Response};
// 全局Endpoint缓存，所有IrohClient共享同一个Endpoint
static ENDPOINT: tokio::sync::OnceCell<Endpoint> = tokio::sync::OnceCell::const_new();

pub struct IrohClient {
    addr: EndpointAddr,
}

// 添加IrohClient的构造函数
impl Clone for IrohClient {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
        }
    }
}

impl IrohClient {
    pub fn new(addr: EndpointAddr) -> Self {
        Self { addr }
    }
    async fn send_request<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned,
    {
        // 构建请求
        // 对于Iroh来说，URI应该只包含路径部分，而不是完整的URL
        // 提取URL中的路径部分
        let uri_path = if req.url.starts_with("iroh://") {
            // 如果是Iroh URL，提取API路径部分
            // 实际的API路径应该从"/api/v4"开始
            if let Some(api_path_start) = req.url.find("/api/v4") {
                // 从"/api/v4"开始提取路径
                req.url[api_path_start..].to_string()
            } else {
                // 如果没有找到"/api/v4"，使用空路径
                "/".to_string()
            }
        } else {
            // 否则使用完整URL
            req.url.clone()
        };

        let mut request_builder = HttpRequest::builder()
            .uri(&uri_path)
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

        let start = Instant::now();

        eprintln!(
            "[IrohClient] Sending request: {} {}, cost: {:?}",
            request.method(),
            request.uri(),
            start.elapsed()
        );
        let endpoint = ENDPOINT
            .get_or_init(|| async {
                let secret = get_or_create_secret();
                let transport_config = TransportConfig::default();
                Endpoint::builder()
                    .secret_key(secret.clone())
                    .transport_config(transport_config)
                    .bind()
                    .await
                    .expect("Failed to bind Iroh endpoint, check if the address is correct")
            })
            .await;
        eprintln!("[IrohClient] get endpoint cost: {:?}", start.elapsed());
        let conn = endpoint.connect(self.addr.clone(), dumbpipe::ALPN).await?;

        eprintln!("[IrohClient] connected, cost: {:?}", start.elapsed());
        // 打开双向流
        let (mut send, recv) = conn.open_bi().await?;
        eprintln!(
            "[IrohClient] Bidirectional stream opened, cost: {:?}",
            start.elapsed()
        );
        send.write_all(&dumbpipe::HANDSHAKE).await?;
        eprintln!("[IrohClient] Handshake sent, cost: {:?}", start.elapsed());
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
        eprintln!(
            "[IrohClient] HTTP handshake completed, cost: {:?}",
            start.elapsed()
        );
        // 在后台处理连接
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("[IrohClient] wait Connection error: {}", err);
            }
        });

        // 发送请求并获取响应
        let response = sender
            .send_request(request)
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        eprintln!(
            "[IrohClient] Received response status: {:?}, cost: {:?}",
            response.status(),
            start.elapsed()
        );

        let status = response.status();
        let headers = response.headers().clone();

        let body = response.collect().await?.aggregate();
        if status.is_success() {
            let data: T = serde_json::from_reader(body.reader())?;
            eprintln!(
                "[IrohClient] Response body parsed successfully, cost: {:?}",
                start.elapsed()
            );
            return Ok(Response {
                status,
                headers,
                data,
            });
        } else {
            return Err(anyhow!("Request failed: {:?}", status));
        }
    }
}

impl HttpClient for IrohClient {
    async fn get<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        // 对于GET请求，我们可以忽略body，直接调用send_request
        // 但需要将Request<()>转换为Request<()>
        self.send_request(req).await
    }

    async fn head<R>(&self, req: Request<R>) -> Result<Response<()>>
    where
        R: serde::Serialize,
    {
        self.send_request(req).await
    }

    async fn post<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn put<R, T>(&self, req: Request<R>) -> Result<Response<T>>
    where
        R: serde::Serialize,
        T: DeserializeOwned,
    {
        self.send_request(req).await
    }

    async fn delete<T>(&self, req: Request<()>) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        // 对于DELETE请求，我们可以忽略body，直接调用send_request
        self.send_request(req).await
    }
}

// 缓存密钥，避免每次创建Endpoint时都生成新的密钥
static SECRET_KEY: std::sync::OnceLock<SecretKey> = std::sync::OnceLock::new();

fn get_or_create_secret() -> &'static SecretKey {
    SECRET_KEY.get_or_init(|| {
        let mut rng = rand::thread_rng();
        let key = SecretKey::generate(&mut rng);
        let key_str = hex::encode(key.to_bytes());
        eprintln!("Generated secret key: {key_str}");
        key
    })
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
