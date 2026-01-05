use crate::app::AppState;
use base64::{self, engine::general_purpose::STANDARD, Engine};
use bytes::Bytes;
use tauri::Manager;
use tokio::sync::Mutex;

pub fn handle_iroh_protocol<R>(
    app_context: tauri::UriSchemeContext<'_, R>,
    request: http::Request<Vec<u8>>,
) -> http::Response<Vec<u8>>
where
    R: tauri::Runtime,
{
    let app_handle = app_context.app_handle();
    let uri = request.uri().to_string();
    let uri = uri.trim_start_matches("iroh://localhost/");
    // 解码URI
    let decoded_uri = match decode_btoa_encoded_uri(&uri) {
        Ok(decoded) => decoded.to_string(),
        Err(e) => {
            eprintln!("iroh:// Failed to decode URI: {}", e);
            return http::Response::builder()
                .status(400)
                .body(Vec::new())
                .unwrap();
        }
    };
    // 解析URI以获取路径和参数
    let parsed_uri = match url::Url::parse(&decoded_uri) {
        Ok(uri) => uri,
        Err(e) => {
            eprintln!("iroh:// Failed to parse URI: {}", e);
            return http::Response::builder()
                .status(400)
                .body(Vec::new())
                .unwrap();
        }
    };
    let url = format!("{}?{}", parsed_uri.path(), parsed_uri.query().unwrap_or(""));

    eprintln!(
        "iroh:// Request, method: {:?}, url: {:?}",
        request.method(),
        url
    );

    // 使用block_on来执行异步操作
    tauri::async_runtime::block_on(async move {
        // 获取应用状态
        let state = app_handle.state::<Mutex<AppState>>();
        let app = state.lock().await;
        let mut req_builder = match app.request(request.method().clone(), &url) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to build request: {}", e);
                return http::Response::builder()
                    .status(500)
                    .body(Vec::new())
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
                let data = match response.bytes().await {
                    Ok(bytes) => bytes.to_vec(),
                    Err(e) => {
                        eprintln!("Failed to read response bytes: {}", e);
                        return http::Response::builder()
                            .status(500)
                            .body(Vec::new())
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

                // 如果是成功响应，设置Content-Type
                if status.is_success() {
                    http_response_builder =
                        http_response_builder.header("Content-Type", "application/octet-stream");
                    http_response_builder =
                        http_response_builder.header("Content-Length", data.len().to_string());
                }

                // 返回响应
                if let Ok(resp) = http_response_builder.body(data) {
                    eprintln!(
                        "outgoing iroh response,  status: {:?}, body size:{}",
                        resp.status(),
                        resp.body().len()
                    );
                    return resp;
                } else {
                    return http::Response::builder()
                        .status(status)
                        .body(Vec::new())
                        .unwrap();
                }
            }
            Err(e) => {
                eprintln!("iroh:// Request failed: {}", e);
                http::Response::builder()
                    .status(500)
                    .body(Vec::new())
                    .unwrap()
            }
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
