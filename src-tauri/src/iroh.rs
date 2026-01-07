use crate::app::AppState;
use base64::{self, engine::general_purpose::STANDARD, Engine};
use bytes::{Bytes, BytesMut};
use futures::{ready, Stream};
use http_body::{Body, Frame};
use http_body_util::BodyExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tauri::{Manager, UriSchemeResponder};
use tokio::sync::Mutex;

// 流式响应体结构体
struct StreamingBody {
    data: Option<Bytes>,
}

impl StreamingBody {
    async fn new(response: reqwest::Response) -> Self {
        // 由于 reqwest 配置的限制，我们先获取整个响应体，然后再处理
        // 在实际应用中，当 reqwest 启用 stream 特性后，可以改为流式处理
        let data = response.bytes().await.ok();
        Self { data }
    }
}

impl Body for StreamingBody {
    type Data = Bytes;
    type Error = std::io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if let Some(data) = self.data.take() {
            Poll::Ready(Some(Ok(Frame::data(data))))
        } else {
            Poll::Ready(None)
        }
    }
}

// 异步协议处理函数
pub async fn handle_iroh_protocol_async<R>(
    app_context: tauri::UriSchemeContext<'_, R>,
    request: http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) where
    R: tauri::Runtime,
{
    let app_handle = app_context.app_handle();
    let uri = request.uri().to_string();
    let uri = uri.trim_start_matches("iroh://localhost/");

    // 解码URI
    let decoded_uri = match decode_btoa_encoded_uri(&uri) {
        Ok(decoded) => decoded.to_string(),
        Err(_) => {
            let response = http::Response::builder()
                .status(400)
                .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                .unwrap();
            responder.respond(response);
            return;
        }
    };

    // 解析URI以获取路径和参数
    let parsed_uri = match url::Url::parse(&decoded_uri) {
        Ok(uri) => uri,
        Err(_) => {
            let response = http::Response::builder()
                .status(400)
                .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                .unwrap();
            responder.respond(response);
            return;
        }
    };

    let url = format!("{}?{}", parsed_uri.path(), parsed_uri.query().unwrap_or(""));

    // 获取应用状态
    let state = app_handle.state::<Mutex<AppState>>();
    let app = state.lock().await;
    let mut req_builder = match app.request(request.method().clone(), &url) {
        Ok(req) => req,
        Err(_) => {
            let response = http::Response::builder()
                .status(500)
                .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                .unwrap();
            responder.respond(response);
            return;
        }
    };

    req_builder = req_builder.headers(request.headers().clone());
    req_builder = req_builder.body(Bytes::copy_from_slice(request.body()));

    // 发送请求并获取响应
    match req_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();

            // 构建响应
            let mut http_response_builder = http::Response::builder().status(status);

            for (key, value) in headers {
                if let Some(key) = key {
                    http_response_builder = http_response_builder.header(key, value.to_owned());
                }
            }

            // 使用流式响应体
            let streaming_body = StreamingBody::new(response).await;

            // 由于 Tauri 的 UriSchemeResponder 要求响应体实现 Into<Cow<'static, [u8]>>，
            // 我们需要将流式响应转换为内存中的响应体
            let body = match streaming_body.data {
                Some(data) => data.to_vec(),
                None => {
                    let response = http::Response::builder()
                        .status(204)
                        .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                        .unwrap();
                    responder.respond(response);
                    return;
                }
            };

            let response = http_response_builder
                .body(std::borrow::Cow::Owned(body))
                .unwrap();
            responder.respond(response);
        }
        Err(_) => {
            let response = http::Response::builder()
                .status(500)
                .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                .unwrap();
            responder.respond(response);
        }
    }
}

// 为了兼容旧的调用方式，保留原函数
pub fn handle_iroh_protocol<R>(
    app_context: tauri::UriSchemeContext<'_, R>,
    request: http::Request<Vec<u8>>,
) -> http::Response<std::borrow::Cow<'static, [u8]>>
where
    R: tauri::Runtime,
{
    let app_handle = app_context.app_handle();
    let uri = request.uri().to_string();
    let uri = uri.trim_start_matches("iroh://localhost/");
    // 解码URI
    let decoded_uri = match decode_btoa_encoded_uri(&uri) {
        Ok(decoded) => decoded.to_string(),
        Err(_) => {
            return http::Response::builder()
                .status(400)
                .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                .unwrap();
        }
    };
    // 解析URI以获取路径和参数
    let parsed_uri = match url::Url::parse(&decoded_uri) {
        Ok(uri) => uri,
        Err(_) => {
            return http::Response::builder()
                .status(400)
                .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                .unwrap();
        }
    };
    let url = format!("{}?{}", parsed_uri.path(), parsed_uri.query().unwrap_or(""));

    // 使用block_on来执行异步操作
    tauri::async_runtime::block_on(async move {
        // 获取应用状态
        let state = app_handle.state::<Mutex<AppState>>();
        let app = state.lock().await;
        let mut req_builder = match app.request(request.method().clone(), &url) {
            Ok(req) => req,
            Err(_) => {
                return http::Response::builder()
                    .status(500)
                    .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                    .unwrap();
            }
        };
        req_builder = req_builder.headers(request.headers().clone());
        req_builder = req_builder.body(Bytes::copy_from_slice(request.body()));

        // 发送请求并获取响应
        match req_builder.send().await {
            Ok(response) => {
                let status = response.status();
                let headers = response.headers().clone();

                // 读取响应体
                let body = match response.bytes().await {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        return http::Response::builder()
                            .status(500)
                            .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                            .unwrap();
                    }
                };

                // 构建响应
                let mut http_response_builder = http::Response::builder().status(status);

                for (key, value) in headers {
                    if let Some(key) = key {
                        http_response_builder = http_response_builder.header(key, value.to_owned());
                    }
                }

                // 返回响应
                if let Ok(resp) = http_response_builder.body(std::borrow::Cow::Owned(body.to_vec()))
                {
                    return resp;
                } else {
                    return http::Response::builder()
                        .status(status)
                        .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                        .unwrap();
                }
            }
            Err(_) => http::Response::builder()
                .status(500)
                .body(std::borrow::Cow::Borrowed(&[] as &[u8]))
                .unwrap(),
        }
    })
}

fn decode_btoa_encoded_uri(encoded_str: &str) -> anyhow::Result<String> {
    // 1. Base64解码
    let decoded_bytes = STANDARD.decode(encoded_str.trim())?; // trim() 用于去除可能的空白字符

    // 2. 将解码后的字节转换为字符串。这步得到的是百分号编码的URL。
    let percent_encoded_url = String::from_utf8(decoded_bytes)?;

    // 3. URL解码（百分号解码）
    let final_url = urlencoding::decode(&percent_encoded_url)?.into_owned();

    Ok(final_url)
}
