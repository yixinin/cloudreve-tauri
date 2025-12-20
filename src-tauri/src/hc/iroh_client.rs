use std::future;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use bytes::{Buf, Bytes, BytesMut};
use http::Request as HttpRequest;
use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;
use iroh::endpoint::TransportConfig;
use iroh::{Endpoint, EndpointAddr, SecretKey};
use iroh_tickets::endpoint::EndpointTicket;
use serde::de::DeserializeOwned;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use crate::hc::{h3_iroh, quinn_endpoint, HttpClient, Request, Response};
// 全局Endpoint缓存，所有IrohClient共享同一个Endpoint
static ENDPOINT: tokio::sync::OnceCell<Endpoint> = tokio::sync::OnceCell::const_new();

pub struct IrohClient {
    endpoint: iroh::Endpoint,
    drive_task: Option<JoinHandle<Result<()>>>,
    send_request: Arc<tokio::sync::Mutex<h3::client::SendRequest<h3_iroh::OpenStreams, Bytes>>>,
}

impl Clone for IrohClient {
    fn clone(&self) -> Self {
        Self {
            endpoint: self.endpoint.clone(),
            drive_task: None,
            send_request: self.send_request.clone(),
        }
    }
}

impl IrohClient {
    pub async fn new(ticket: &str) -> Result<Self> {
        let ticket = EndpointTicket::from_str(ticket)?;
        let addr: EndpointAddr = ticket.into();

        let ep = iroh::Endpoint::builder().bind().await?;

        let conn = ep.connect(addr, b"iroh+h3").await?;
        eprintln!("success connect to remote: {:?}", conn.remote_id());
        let conn = h3_iroh::Connection::new(conn);

        let (mut driver, send_request) = h3::client::new(conn).await?;
        let send_request = Arc::new(tokio::sync::Mutex::new(send_request));

        // 驱动h3连接的任务
        let drive_task = tokio::spawn(async move {
            let err = future::poll_fn(|cx| driver.poll_close(cx)).await;
            match err {
                h3::error::ConnectionError::Local { ref error, .. } => {
                    if matches!(error, h3::error::LocalError::Closing { .. }) {
                        Ok(())
                    } else {
                        Err(err.into())
                    }
                }
                _ => Err(err.into()),
            }
        });

        Ok(Self {
            endpoint: ep,
            drive_task: Some(drive_task),
            send_request,
        })
    }

    async fn send_request<T>(&self, req: Request) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        let mut send_request = self.send_request.lock().await;

        // 构建请求
        let mut req_builder = http::Request::builder().method(req.method()).uri(req.url());

        eprintln!("method: {:?} url: {:?}", req.method(), req.url());
        for (key, value) in req.headers().iter() {
            eprintln!("header: {:?} {:?}", key, value);
            req_builder = req_builder.header(key, value.to_owned());
        }
        // 设置Content-Length头（如果有body）
        if let Some(ref body) = req.body {
            req_builder = req_builder.header(http::header::CONTENT_LENGTH, body.len());
        }

        let request = req_builder.body(())?;
        let mut stream = send_request.send_request(request).await?;

        if let Some(body) = req.body {
            stream.send_data(body).await?;
        }

        // 完成请求发送
        stream.finish().await?;

        // 接收响应
        let resp = stream.recv_response().await?;

        // 接收响应体
        let mut body_bytes = BytesMut::new();
        while let Some(chunk) = stream.recv_data().await? {
            body_bytes.extend_from_slice(chunk.chunk());
        }
        let body_bytes = body_bytes.freeze();

        let mut headers = http::HeaderMap::new();

        // 复制所有响应头
        for (key, value) in resp.headers() {
            headers.append(key, value.to_owned());
        }
        let data = serde_json::from_slice::<T>(&body_bytes)?;
        let resp = Response {
            status: resp.status(),
            headers: headers,
            data: data,
        };

        Ok(resp)
    }

    async fn send_request_bytes(&self, req: Request) -> Result<Response<Bytes>> {
        let mut send_request = self.send_request.lock().await;

        // 构建请求
        let mut req_builder = http::Request::builder().method(req.method()).uri(req.url());

        eprintln!("method: {:?} url: {:?}", req.method(), req.url());
        for (key, value) in req.headers().iter() {
            eprintln!("header: {:?} {:?}", key, value);
            req_builder = req_builder.header(key, value.to_owned());
        }
        // 设置Content-Length头（如果有body）
        if let Some(ref body) = req.body {
            req_builder = req_builder.header(http::header::CONTENT_LENGTH, body.len());
        }

        let request = req_builder.body(())?;
        let mut stream = send_request.send_request(request).await?;

        if let Some(body) = req.body {
            stream.send_data(body).await?;
        }

        // 完成请求发送
        stream.finish().await?;

        // 接收响应
        let resp = stream.recv_response().await?;

        // 接收响应体
        let mut body_bytes = BytesMut::new();
        while let Some(chunk) = stream.recv_data().await? {
            body_bytes.extend_from_slice(chunk.chunk());
        }
        let body_bytes = body_bytes.freeze();

        let mut headers = http::HeaderMap::new();

        // 复制所有响应头
        for (key, value) in resp.headers() {
            headers.append(key, value.to_owned());
        }
        let resp = Response {
            status: resp.status(),
            headers: headers,
            data: body_bytes,
        };

        Ok(resp)
    }
    pub async fn close(self) -> Result<()> {
        // 等待驱动任务完成
        if let Some(drive_task) = self.drive_task {
            self.endpoint.close().await;
            drive_task.await??;
        }

        Ok(())
    }
}

impl HttpClient for IrohClient {
    async fn get<T>(&self, req: Request) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        // 对于GET请求，我们可以忽略body，直接调用send_request
        // 但需要将Request<()>转换为Request<()>
        self.send_request(req).await
    }

    async fn get_bytes(&self, req: Request) -> Result<Response<Bytes>> {
        // 对于GET请求，我们可以忽略body，直接调用send_request_bytes
        self.send_request_bytes(req).await
    }

    async fn head(&self, req: Request) -> Result<Response<()>> {
        self.send_request(req).await
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
