use crate::hc::http_client_manager::{ClientType, HttpClientWrapper};
use crate::hc::Request;
use base64::{self, engine::general_purpose::STANDARD, Engine};
use bytes::Bytes;
use http::{HeaderMap, Method, Version};
use std::{
    collections::{HashMap, HashSet},
    path::{self},
};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::Mutex;
use url::Url;
use urlencoding;

use crate::{
    app::AppState,
    proto::{
        login::LoginReq,
        settings::{NetworkMode, NetworkSettings},
        storage::GetCapacityAck,
        Result,
    },
};

use proto::{
    file::DeleteFileAck,
    share::{GetSharesAck, ShareInfo},
    storage::{Download, FileDetailsInfo, FileInfo, FileTag, GetFilesAck, GetFilesReq, Upload},
    AppError, JsonResult,
};
use serde::Serialize;

pub mod app;
pub mod media;
pub mod net;
pub mod proto;
pub mod transfer;

fn decode_btoa_encoded_uri(encoded_str: &str) -> anyhow::Result<String> {
    // 1. Base64解码
    let decoded_bytes = STANDARD.decode(encoded_str.trim())?; // trim() 用于去除可能的空白字符

    // 2. 将解码后的字节转换为字符串。这步得到的是百分号编码的URL。
    let percent_encoded_url = String::from_utf8(decoded_bytes)?;

    // 3. URL解码（百分号解码）
    let final_url = urlencoding::decode(&percent_encoded_url)?.into_owned();

    Ok(final_url)
}
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .register_uri_scheme_protocol("iroh", |app_context, request| {
            let app_handle = app_context.app_handle();
            let uri = request.uri().to_string();
            let uri = uri.trim_start_matches("iroh://localhost/");

            // 使用block_on来执行异步操作
            tauri::async_runtime::block_on(async move {
                // 获取应用状态
                let state = app_handle.state::<Mutex<AppState>>();
                let app = state.lock().await;

                // 解码URI
                let decoded_uri = match decode_btoa_encoded_uri(&uri) {
                    Ok(decoded) => decoded.to_string(),
                    Err(e) => {
                        eprintln!("Failed to decode URI: {}", e);
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
                        eprintln!("Failed to parse URI: {}", e);
                        return http::Response::builder()
                            .status(400)
                            .body(Vec::new())
                            .unwrap();
                    }
                };
                let path = parsed_uri.path();

                // 获取网络设置和基础URL
                let settings = match app.get_network_settings() {
                    Ok(settings) => settings,
                    Err(e) => {
                        eprintln!("Failed to get network settings: {}", e);
                        return http::Response::builder()
                            .status(500)
                            .body(Vec::new())
                            .unwrap();
                    }
                };
                let base_url = settings.get_addr();

                // 构建完整的请求URL
                let request_url = format!("{}{}", base_url, path);

                // 根据网络设置选择客户端类型
                let client_type = if settings.mode == NetworkMode::P2P {
                    ClientType::Iroh
                } else {
                    ClientType::Reqwest
                };

                // 获取客户端
                let client = match app.hcm.get_client(client_type).await {
                    Ok(client) => client,
                    Err(e) => {
                        eprintln!("Failed to get client: {}", e);
                        return http::Response::builder()
                            .status(500)
                            .body(Vec::new())
                            .unwrap();
                    }
                };

                let method = request.method();

                // 构建请求对象
                let mut req = Request::new(method.to_owned(), &request_url, "");
                req = req.with_headers(request.headers().clone());
                req = req.with_body(Bytes::copy_from_slice(request.body()));
                // 发送请求并获取响应
                match client.get_bytes(req).await {
                    Ok(response) => {
                        let status = response.status();
                        let headers = response.headers().clone();
                        let data = response.into_data().to_vec();

                        // 构建响应
                        let mut http_response_builder = http::Response::builder().status(status);

                        for (key, value) in headers {
                            eprintln!("h3 response header: {:?} {:?}", key, value);
                            if let Some(key) = key {
                                http_response_builder =
                                    http_response_builder.header(key, value.to_owned());
                            }
                        }

                        // 如果是成功响应，设置Content-Type
                        if status.is_success() {
                            http_response_builder = http_response_builder
                                .header("Content-Type", "application/octet-stream");
                            http_response_builder = http_response_builder.header("Content-Length", data.len().to_string());
                        }

                        // 返回响应
                        if let Ok(resp) = http_response_builder.body(data) {
                            eprintln!("outgoing iroh response, method: {:?}, url: {:?}, status: {:?}, body size:{}", 
                                &method,
                                &request_url,
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
                        eprintln!("Request failed: {}", e);
                        http::Response::builder()
                            .status(500)
                            .body(Vec::new())
                            .unwrap()
                    }
                }
            })
        })
        .setup(|app| {
            let app_handle = app.handle();
            let mut app_state = app::AppState::new(&app_handle)?;

            let settings = app_state.get_network_settings()?;
            tauri::async_runtime::block_on(async move {
                if let Err(e) = app_state.init_base_url(&settings).await {
                    eprintln!("Error initializing base URL: {:?}", e);
                }
                // app_state.init_base_url(&settings).await?;
                if settings.mode == NetworkMode::P2P {
                    // 只有当有实际的Iroh端点地址时，才使用Iroh客户端
                    if settings.get_addr() != "iroh://p2p" {
                        app_state.ct = ClientType::Iroh;
                        println!("Using Iroh client for P2P mode");
                    } else {
                        // 如果是默认的P2P标记，使用Reqwest客户端
                        app_state.ct = ClientType::Reqwest;
                        println!("Using Reqwest client for default P2P mode");
                    }
                }
                app.manage(Mutex::new(app_state));

                #[cfg(dev)]
                {
                    let window = app.get_webview_window("main").unwrap();
                    window.open_devtools();
                }
                #[cfg(mobile)]
                {
                    let app_handle = app.handle();
                    app_handle.plugin(tauri_plugin_app_events::init())?;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_fullscreen,
            get_network_settings,
            set_network_settings,
            prepare,
            login,
            logout,
            get_storage_info,
            get_files,
            search_files,
            get_url,
            get_thumb_url,
            rename_file,
            delete_file,
            create_folder,
            get_file_info,
            share_file,
            get_file_source,
            pre_upload,
            pre_download,
            upload,
            download,
            get_shares,
            delete_lock,
            restore_file,
            move_file,
            delete_share,
            update_share,
            get_share_info,
            get_sync_status,
            get_sync_progress,
            trigger_sync,
            get_download_history,
            proxy_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn toggle_fullscreen(window: tauri::Window) -> crate::proto::Result<()> {
    let is_fullscreen = window.is_fullscreen().unwrap_or(false);

    #[cfg(target_os = "android")]
    {
        // Android doesn't support programmatic fullscreen toggle via this API
        return Ok(());
    }

    #[cfg(not(target_os = "android"))]
    {
        window.set_fullscreen(!is_fullscreen).unwrap_or(());
    }

    Ok(())
}

#[tauri::command]
async fn set_network_settings(
    state: tauri::State<'_, Mutex<AppState>>,
    addr: Option<String>,
    addr6: Option<String>,
    iroh_endpoint: Option<String>,
    mode: NetworkMode,
) -> JsonResult<NetworkSettings> {
    let mut app = state.lock().await;
    app.set_addr(addr, addr6, iroh_endpoint, mode).await?;
    let settings = app.get_network_settings()?;
    let addr = settings.get_addr();

    // 初始化基础URL
    app.init_base_url(&settings).await?;

    // 根据网络模式设置客户端类型
    match settings.mode {
        NetworkMode::P2P => {
            // 只有当addr不是默认的"iroh://p2p"时，才使用Iroh客户端
            // 这样可以确保只有在提供了实际的Iroh端点地址时才使用P2P模式
            if addr != "iroh://p2p" {
                app.ct = hc::http_client_manager::ClientType::Iroh;
            } else {
                // 如果是默认地址，回退到Reqwest客户端
                app.ct = hc::http_client_manager::ClientType::Reqwest;
            }
        }
        _ => {
            app.ct = hc::http_client_manager::ClientType::Reqwest;
        }
    }

    return Ok(settings);
}
#[tauri::command]
async fn get_network_settings(
    state: tauri::State<'_, Mutex<AppState>>,
) -> JsonResult<NetworkSettings> {
    eprintln!("get_network_settings watting lock");
    let app = state.lock().await;
    eprintln!("get_network_settings get lock");
    let settings = app.get_network_settings()?;
    return Ok(settings);
}

#[tauri::command]
async fn prepare(
    state: tauri::State<'_, Mutex<AppState>>,
    username: String,
) -> JsonResult<proto::login::PrepareAck> {
    let app = state.lock().await;
    let ack = app.prepare(&username).await?;
    Ok(ack)
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, Mutex<AppState>>,
    username: String,
    password: String,
) -> JsonResult<proto::login::User> {
    let req = LoginReq {
        email: username,
        password,
    };
    let app = state.lock().await;
    let ack = app.login(&req.email, &req.password).await?;
    return Ok(ack);
}

#[tauri::command]
async fn logout(state: tauri::State<'_, Mutex<AppState>>) -> JsonResult<bool> {
    let app = state.lock().await;
    app.logout().await?;
    Ok(true)
}

#[tauri::command]
async fn get_storage_info(state: tauri::State<'_, Mutex<AppState>>) -> JsonResult<GetCapacityAck> {
    let app = state.lock().await;
    app.get_capacity().await
}

#[tauri::command]
async fn get_files(
    state: tauri::State<'_, Mutex<AppState>>,
    path: String,
    category: String,
    page: u64,
    page_size: u64,
    order_by: String,
    order: String,
) -> JsonResult<GetFilesAck> {
    let app = state.lock().await;
    let uri: String;
    if !category.is_empty() {
        uri = format!("{}?category={}", path, category);
    } else {
        uri = path;
    }
    let req = GetFilesReq {
        page: page,
        page_size: page_size,
        uri: uri,
        order_by: order_by,
        order_direction: order,
    };
    let mut ack = app.get_files(req).await?;
    let mut tags = HashSet::new();
    let mut filetags = Vec::new();
    for (_, file) in ack.files.iter().enumerate() {
        let mut file_tags = Vec::new();
        if let Some(metadata) = &file.metadata {
            for (k, v) in metadata {
                if k.starts_with("tag:") {
                    tags.insert(k.clone());
                    file_tags.push(FileTag {
                        key: k.clone()[4..].to_string(),
                        color: v.clone(),
                    })
                }
            }
        }
        filetags.push(file_tags)
    }
    ack.file_tags = Some(filetags);
    let tag_vec = tags.into_iter().collect();
    ack.tags = Some(tag_vec);
    Ok(ack)
}

#[tauri::command]
async fn search_files(
    state: tauri::State<'_, Mutex<AppState>>,
    name: String,
    path: String,
    page: u64,
    page_size: u64,
    order_by: String,
    order: String,
) -> JsonResult<GetFilesAck> {
    let app = state.lock().await;
    let uri = format!("{}?name={}&name_op_or=&case_folding=", path, name);

    let req = GetFilesReq {
        page: page,
        page_size: page_size,
        uri: uri,
        order_by: order_by,
        order_direction: order,
    };
    app.get_files(req).await
}

#[tauri::command]
async fn get_url(state: tauri::State<'_, Mutex<AppState>>, uri: String) -> JsonResult<String> {
    let app = state.lock().await;
    // 确保始终使用Iroh客户端获取URL
    let client = app
        .hcm
        .get_client(hc::http_client_manager::ClientType::Iroh)
        .await?;
    let ack = app.batch_urls_with_client(vec![uri.clone()], client).await;
    match ack {
        Ok(ack) => {
            for v in ack.urls {
                return Ok(v.url);
            }
            return Err(AppError::NoData);
        }
        Err(e) => return Err(e),
    }
}

#[tauri::command]
async fn get_thumb_url(
    state: tauri::State<'_, Mutex<AppState>>,
    uri: String,
) -> JsonResult<String> {
    let app = state.lock().await;
    // 使用Iroh客户端获取缩略图URL
    let client = app
        .hcm
        .get_client(hc::http_client_manager::ClientType::Iroh)
        .await?;
    app.get_thumb_url_with_client(uri, client).await
}

#[derive(Clone, Serialize)]
pub struct SessionChaged {
    logined: bool,
}

#[tauri::command]
async fn rename_file(
    state: tauri::State<'_, Mutex<AppState>>,
    new_name: String,
    uri: String,
) -> JsonResult<FileDetailsInfo> {
    let app = state.lock().await;
    let ack = app.rename(new_name, uri).await;
    match ack {
        Ok(ack) => {
            return Ok(ack);
        }
        Err(e) => return Err(e),
    }
}

#[tauri::command]
async fn delete_file(
    state: tauri::State<'_, Mutex<AppState>>,
    unlink: bool,
    soft_delete: bool,
    uri: String,
) -> JsonResult<Option<DeleteFileAck>> {
    let app = state.lock().await;
    app.delete_file(unlink, soft_delete, vec![uri]).await
}

#[tauri::command]
async fn create_folder(
    state: tauri::State<'_, Mutex<AppState>>,
    uri: String,
) -> JsonResult<FileInfo> {
    let app = state.lock().await;
    app.create_folder(&uri).await
}

#[tauri::command]
async fn get_file_info(
    state: tauri::State<'_, Mutex<AppState>>,
    uri: String,
) -> JsonResult<FileDetailsInfo> {
    let app = state.lock().await;
    app.get_file_info(&uri).await
}

#[tauri::command]
async fn share_file(
    state: tauri::State<'_, Mutex<AppState>>,
    downloads: u64,
    expire: u64,
    is_private: bool,
    uri: String,
) -> JsonResult<String> {
    let app = state.lock().await;
    app.share_file(downloads, expire, is_private, &uri).await
}

#[tauri::command]
async fn get_file_source(
    state: tauri::State<'_, Mutex<AppState>>,
    uri: String,
) -> JsonResult<String> {
    let app = state.lock().await;
    let ack = app.get_file_source(vec![uri]).await?;
    return Ok(ack.link);
}

#[cfg(target_os = "windows")]
fn get_downloads_dir(app: &AppHandle) -> Result<String> {
    match app.path().download_dir() {
        Ok(dir) => {
            return Ok(dir.to_string_lossy().to_string());
        }
        Err(err) => {
            return Err(AppError::Anyhow(err.into()));
        }
    }
}

#[cfg(target_os = "macos")]
fn get_downloads_dir(app: &AppHandle) -> Result<String> {
    match app.path().download_dir() {
        Ok(dir) => {
            return Ok(dir.to_string_lossy().to_string());
        }
        Err(err) => {
            return Err(AppError::Anyhow(err.into()));
        }
    }
}

#[cfg(target_os = "android")]
fn get_downloads_dir(app: &AppHandle) -> Result<String> {
    match app.path().home_dir() {
        Ok(dir) => {
            return Ok(format!("{}/Download", dir.to_string_lossy().to_string()));
        }
        Err(err) => {
            return Err(AppError::Anyhow(err.into()));
        }
    }
}

#[tauri::command]
async fn download(
    app: AppHandle,
    id: u32,
    url: String,
    file_path: String,
    headers: Option<HashMap<String, String>>,
) -> JsonResult<u32> {
    transfer::download(&app, id, &url, &file_path, headers).await
}

#[tauri::command]
async fn upload(
    hd: AppHandle,
    id: u32,
    url: String,
    file_path: String,
    headers: Option<HashMap<String, String>>,
) -> JsonResult<u32> {
    transfer::upload(&hd, id, &url, &file_path, headers).await
}

#[tauri::command]
async fn pre_upload(
    hd: AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    target_path: String,
    policy_id: String,
) -> JsonResult<Upload> {
    // 先进行文件选择，不要持有锁
    let path = match hd.dialog().file().blocking_pick_file() {
        Some(path) => path,
        None => return Err(AppError::NoData),
    };

    let file_path = path.as_path().unwrap();
    let filename = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let uri = format!("{}/{}", target_path, filename);

    let mime_type = proto::storage::get_mime_type(&filename);
    let mut size = 0;
    if let Ok(meta) = std::fs::metadata(&file_path) {
        size = meta.len()
    }

    // 现在获取锁，进行后续操作
    let app = state.lock().await;
    let id = app.gen_id().await?;

    println!(
        "  dir:{}, policy_id:{}, filename:{}, mime:{}",
        target_path, policy_id, filename, &mime_type
    );
    let ack = app
        .upload_file_session(&mime_type, &uri, size, &policy_id)
        .await?;
    let addr = app.get_network_settings()?.get_addr();
    let upload = Upload {
        id: id,
        file_path: file_path.to_string_lossy().to_string(),
        filename: filename.to_string(),
        uri: uri.clone(),
        upload_url: format!("{}{}/{}/0", addr, "/file/upload", ack.session_id),
    };
    return Ok(upload);
}

#[tauri::command]
async fn pre_download(
    hd: AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    uri: String,
    filename: String,
) -> JsonResult<Download> {
    let app = state.lock().await;
    let ack = app.batch_urls(vec![uri.clone()]).await;
    let mut download_url = "".to_string();
    match ack {
        Ok(ack) => {
            for v in ack.urls {
                download_url = v.url;
            }
        }
        Err(e) => return Err(e),
    }
    let saved_dir = get_downloads_dir(&hd)?;
    if download_url.is_empty() || saved_dir.is_empty() {
        return Err(AppError::Anyhow(anyhow::format_err!(
            "fail, dir:{}, url:{}",
            saved_dir,
            download_url
        )));
    }
    let id = app.gen_id().await?;

    Ok(Download {
        id: id,
        file_path: path::Path::new(&saved_dir)
            .join(filename)
            .to_string_lossy()
            .to_string(),
        url: download_url,
    })
}

#[tauri::command]
async fn restore_file(state: tauri::State<'_, Mutex<AppState>>, uri: String) -> JsonResult<bool> {
    let app = state.lock().await;
    app.restore_file(vec![uri.clone()]).await
}
#[tauri::command]
async fn delete_lock(
    state: tauri::State<'_, Mutex<AppState>>,
    tokens: Vec<String>,
) -> JsonResult<bool> {
    let app = state.lock().await;
    app.delete_lock(tokens).await
}
#[tauri::command]
async fn get_shares(
    state: tauri::State<'_, Mutex<AppState>>,
    order_direction: String,
) -> JsonResult<GetSharesAck> {
    let app = state.lock().await;
    app.get_shares(&order_direction).await
}

#[tauri::command]
async fn move_file(
    state: tauri::State<'_, Mutex<AppState>>,
    copy: bool,
    dst: String,
    uri: String,
) -> JsonResult<bool> {
    let app = state.lock().await;
    app.move_file(vec![uri], &dst, copy).await
}
#[tauri::command]
async fn delete_share(state: tauri::State<'_, Mutex<AppState>>, id: String) -> JsonResult<bool> {
    let app = state.lock().await;
    app.delete_share(&id).await
}
#[tauri::command]
async fn update_share(
    state: tauri::State<'_, Mutex<AppState>>,
    id: String,
    downloads: u8,
    expire: u64,
    uri: String,
) -> JsonResult<String> {
    let app = state.lock().await;
    app.update_share(&id, downloads, expire, &uri).await
}
#[tauri::command]
async fn get_share_info(
    state: tauri::State<'_, Mutex<AppState>>,
    id: String,
    owner_extended: bool,
) -> JsonResult<ShareInfo> {
    let app = state.lock().await;
    app.get_share_info(&id, owner_extended).await
}

#[derive(Debug, Serialize)]
struct SyncStatus {
    enabled: bool,
    last_sync: Option<String>,
    synced_files: Option<u32>,
}

#[derive(Debug, Serialize)]
struct SyncProgress {
    percentage: u8,
    synced: u32,
    total: u32,
    status: String,
}

#[tauri::command]
async fn get_sync_status() -> JsonResult<SyncStatus> {
    Ok(SyncStatus {
        enabled: false,
        last_sync: Some("2024-05-20T12:34:56".to_string()),
        synced_files: Some(42),
    })
}

#[tauri::command]
async fn toggle_sync(enabled: bool) -> JsonResult<SyncStatus> {
    Ok(SyncStatus {
        enabled,
        last_sync: None,
        synced_files: None,
    })
}

#[tauri::command]
async fn get_sync_progress() -> JsonResult<SyncProgress> {
    Ok(SyncProgress {
        percentage: 35,
        synced: 7,
        total: 20,
        status: "syncing".to_string(),
    })
}

#[tauri::command]
async fn trigger_sync() -> JsonResult<()> {
    Ok(())
}

#[tauri::command]
async fn proxy_image(state: tauri::State<'_, Mutex<AppState>>, url: String) -> JsonResult<Vec<u8>> {
    let app = state.lock().await;
    // 使用Reqwest客户端获取图片数据
    let client = app
        .hcm
        .get_client(hc::http_client_manager::ClientType::Reqwest)
        .await?;

    // 创建请求
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", "Cloudreve-Tauri/1.0".parse().unwrap());

    // 使用 Request::new 方法创建请求对象，将完整URL作为base_url，path设为空
    let req = Request::new(Method::GET, &url, "").with_header("User-Agent", "Cloudreve-Tauri/1.0");

    // 发送请求并获取响应
    let resp = client.get_bytes(req).await?;
    let data = resp.into_data();

    Ok(data.to_vec())
}

#[derive(Debug, Serialize)]
struct DownloadHistoryItem {
    id: u32,
    file_name: String,
    file_path: String,
    url: String,
    status: String,
    progress: u8,
    size: u64,
    downloaded_at: String,
}

#[tauri::command]
async fn get_download_history(
    state: tauri::State<'_, Mutex<AppState>>,
) -> JsonResult<Vec<DownloadHistoryItem>> {
    let app = state.lock().await;
    // 实际实现应从数据库或存储中获取下载历史
    // 这里返回模拟数据作为示例
    Ok(vec![DownloadHistoryItem {
        id: 1,
        file_name: "example.jpg".to_string(),
        file_path: "/downloads/example.jpg".to_string(),
        url: "https://example.com/example.jpg".to_string(),
        status: "completed".to_string(),
        progress: 100,
        size: 1024 * 1024,
        downloaded_at: "2024-06-18T10:30:00Z".to_string(),
    }])
}
