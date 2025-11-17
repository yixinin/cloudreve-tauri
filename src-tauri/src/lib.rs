use http::Version;
use std::{
    collections::{HashMap, HashSet},
    path::{self},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::Mutex;

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
pub mod hc;
pub mod media;
pub mod net;
pub mod proto;
pub mod rendezvouser;
pub mod transfer;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = app::AppState::new()?;
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
            toggle_sync,
            get_download_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn toggle_fullscreen(window: tauri::Window) {
    let is_fullscreen = window.is_fullscreen().unwrap_or(false);
    window.set_fullscreen(!is_fullscreen).unwrap();
}

#[tauri::command]
async fn set_network_settings(
    state: tauri::State<'_, Mutex<AppState>>,
    addr: Option<String>,
    addr6: Option<String>,
    mode: NetworkMode,
) -> JsonResult<NetworkSettings> {
    let app = state.lock().await;
    app.set_addr(addr, addr6, mode)?;
    let settings = app.get_addr()?;
    if settings.mode == NetworkMode::P2P {
        let client = app.get_client(&settings.addr, true).await?;
        let url = settings.get_url(Some(settings.addr.clone()), "/site/config/basic");
        let resp = client.get(url).version(Version::HTTP_3).send().await?;
        app.hc_pool.put(client);
        println!("get basic info: {}", resp.text().await?);
    }
    return Ok(settings);
}
#[tauri::command]
async fn get_network_settings(
    state: tauri::State<'_, Mutex<AppState>>,
) -> JsonResult<NetworkSettings> {
    let app = state.lock().await;
    let settings = app.get_addr()?;
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
    let ack = app.batch_urls(vec![uri.clone()]).await;
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
    app.get_thumb_url(uri).await
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
    let app = state.lock().await;
    let id = app.gen_id()?;

    match hd.dialog().file().blocking_pick_file() {
        Some(path) => {
            let file_path = path.as_path().unwrap();
            let filename = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let uri = format!("{}/{}", target_path, filename);

            let mime_type = proto::storage::get_mime_type(&filename);
            let mut size = 0;
            if let Ok(meta) = std::fs::metadata(&file_path) {
                size = meta.len()
            }
            println!(
                "  dir:{}, policy_id:{}, filename:{}, mime:{}",
                target_path, policy_id, filename, &mime_type
            );
            let ack = app
                .upload_file_session(&mime_type, &uri, size, &policy_id)
                .await?;
            let addr = app.get_addr()?.get_addr();
            let upload = Upload {
                id: id,
                file_path: file_path.to_string_lossy().to_string(),
                filename: filename.to_string(),
                uri: uri.clone(),
                upload_url: format!("{}{}/{}/0", addr, "/file/upload", ack.session_id),
            };
            return Ok(upload);
        }
        None => {
            return Err(AppError::NoData);
        }
    }
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
    let id = app.gen_id()?;

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
        file_name: "example.jpg",
        file_path: "/downloads/example.jpg",
        url: "https://example.com/example.jpg",
        status: "completed",
        progress: 100,
        size: 1024 * 1024,
        downloaded_at: "2024-06-18T10:30:00Z",
    }])
}
