use std::{
    collections::{HashMap, HashSet},
    fmt::format,
    path::{self},
    sync::atomic::{AtomicU32, Ordering},
};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::{net::has_ipv6_connectivity, proto::Result};

use chrono::Utc;
use proto::{
    file::DeleteFileAck,
    login::Token,
    share::{GetSharesAck, ShareInfo},
    storage::{Download, FileDetailsInfo, FileInfo, FileTag, GetFilesAck, GetFilesReq, Upload},
    AppError, JsonResult, Site,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri_plugin_store::StoreExt;

pub mod hc;
pub mod login;
pub mod media;
pub mod net;
pub mod proto;
pub mod rendezvouser;
pub mod storage;
pub mod transfer;

static IDGEN: AtomicU32 = AtomicU32::new(0);
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let store = app.store("app_data.json")?;
            let ipv6 = has_ipv6_connectivity();
            store.set("ipv6", serde_json::json!(ipv6));
            let id_store = app.store("id.json")?;
            if let Some(id) = id_store.get("transfer") {
                if let Ok(id) = serde_json::from_value::<u32>(id) {
                    IDGEN.store(id, Ordering::Relaxed);
                }
            }
            #[cfg(mobile)]
            {
                let app_handle = app.handle();
                app_handle.plugin(tauri_plugin_app_events::init())?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_login_data,
            save_user_setting,
            get_user_setting,
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
            get_paths,
            upload,
            download,
            get_shares,
            delete_lock,
            restore_file,
            move_file,
            delete_share,
            update_share,
            get_share_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginData {
    pub addr: String,
    pub ipv6: bool,
    pub addr6: Option<String>,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSetting {
    pub force_ipv4: bool,
    pub force_ipv6: bool,
}
async fn set_login_data(
    app: &AppHandle,
    addr: &str,
    ipv6: bool,
    addr6: &str,
    username: &str,
    password: &str,
) -> JsonResult<bool> {
    if let Ok(store) = app.store("app_data.json") {
        let mut data = LoginData {
            addr: addr.to_string(),
            ipv6: ipv6,
            addr6: None,
            username: username.to_string(),
            password: password.to_string(),
        };
        if !addr6.is_empty() {
            data.addr6 = Some(addr6.to_string())
        }
        store.set("login", serde_json::json!(data));
        return Ok(true);
    }
    Err(AppError::Anyhow(anyhow::format_err!(
        "cannot access app data"
    )))
}

async fn get_address(app: &AppHandle) -> Result<String> {
    if let Ok(store) = app.store("app_data.json") {
        match store.get("login") {
            Some(value) => {
                let data = serde_json::from_value::<LoginData>(value)?;
                if data.ipv6 {
                    if let Some(addr6) = data.addr6 {
                        if let Some(value) = store.get("user.setting") {
                            let settings: UserSetting = serde_json::from_value(value)?;
                            if settings.force_ipv6 {
                                return Ok(gen_addr(&addr6));
                            }
                            if settings.force_ipv4 {
                                return Ok(gen_addr(&data.addr));
                            }
                        }

                        if let Some(value) = store.get("ipv6") {
                            if let Some(ok) = value.as_bool() {
                                if ok {
                                    return Ok(gen_addr(&addr6));
                                }
                            }
                        }
                    }
                }

                return Ok(gen_addr(&data.addr));
            }
            None => {
                return Err(AppError::Anyhow(anyhow::format_err!("address is none")));
            }
        }
    }
    Err(AppError::Anyhow(anyhow::format_err!(
        "cannot access app data"
    )))
}

fn gen_addr(addr: &str) -> String {
    if addr.starts_with("http") {
        return addr.to_string();
    }
    return format!("https://{}", addr);
}

#[tauri::command]
async fn get_login_data(app: AppHandle) -> Result<LoginData> {
    if let Ok(store) = app.store("app_data.json") {
        match store.get("login") {
            Some(value) => {
                let data = serde_json::from_value::<LoginData>(value)?;
                return Ok(data);
            }
            None => {
                return Err(AppError::Anyhow(anyhow::format_err!("address is none")));
            }
        }
    }
    Err(AppError::Anyhow(anyhow::format_err!(
        "cannot access app data"
    )))
}
#[tauri::command]
async fn save_user_setting(
    app: AppHandle,
    force_ipv4: Option<bool>,
    force_ipv6: Option<bool>,
) -> JsonResult<bool> {
    if let Ok(store) = app.store("app_data.json") {
        match store.get("user.setting") {
            Some(value) => {
                let mut settings: UserSetting = serde_json::from_value(value)?;
                if let Some(ok) = force_ipv4 {
                    settings.force_ipv4 = ok;
                }
                if let Some(ok) = force_ipv6 {
                    settings.force_ipv6 = ok
                }
                store.set("user.setting", serde_json::json!(settings));
            }
            None => {
                let settings = UserSetting {
                    force_ipv4: force_ipv4.unwrap_or_default(),
                    force_ipv6: force_ipv6.unwrap_or_default(),
                };
                store.set("user.setting", serde_json::json!(settings));
            }
        }
        Ok(true)
    } else {
        Err(AppError::Anyhow(anyhow::format_err!(
            "cannot access app data"
        )))
    }
}
#[tauri::command]
async fn get_user_setting(app: AppHandle) -> JsonResult<UserSetting> {
    if let Ok(store) = app.store("app_data.json") {
        match store.get("user.setting") {
            Some(value) => {
                let settings: UserSetting = serde_json::from_value(value)?;
                return Ok(settings);
            }
            None => {
                return Ok(UserSetting {
                    force_ipv4: false,
                    force_ipv6: false,
                });
            }
        }
    }
    Err(AppError::Anyhow(anyhow::format_err!(
        "cannot access app data"
    )))
}
#[tauri::command]
async fn prepare(
    app: AppHandle,
    addr: String,
    username: String,
) -> JsonResult<proto::login::PrepareAck> {
    let address: String;
    if !addr.starts_with("http") {
        address = format!("https://{}", &addr);
    } else {
        address = addr;
    }
    let ack = login::prepare(&address, &username).await?;
    Ok(ack)
}

#[tauri::command]
async fn login(
    app: AppHandle,
    addr: String,
    ipv6: bool,
    addr6: String,
    username: String,
    password: String,
) -> JsonResult<proto::login::LoginAck> {
    println!("{}", &addr6);
    set_login_data(&app, &addr, ipv6, &addr6, &username, &password).await?;
    let address = get_address(&app).await?;
    let ack = login::login(&address, &username, &password).await?;
    if let Ok(store) = app.store("app_data.json") {
        let json_token = json!(ack.token);
        store.set("token", json_token.clone());
    }
    return Ok(ack);
}

#[tauri::command]
async fn logout(app: AppHandle) -> JsonResult<bool> {
    if let Ok(store) = app.store("app_data.json") {
        store.delete("token");
        return Ok(true);
    }
    Err(AppError::Anyhow(anyhow::format_err!(
        "cannot access app data"
    )))
}

#[tauri::command]
async fn get_storage_info(app: AppHandle) -> JsonResult<proto::storage::GetCapacityAck> {
    let site = get_token(&app).await?;
    storage::get_capacity(&site).await
}

#[tauri::command]
async fn get_files(
    app: AppHandle,
    path: String,
    category: String,
    page: u64,
    page_size: u64,
    order_by: String,
    order: String,
) -> JsonResult<GetFilesAck> {
    let uri: String;
    if !category.is_empty() {
        uri = format!("{}?category={}", path, category);
    } else {
        uri = path;
    }
    let site = get_token(&app).await?;
    let req = GetFilesReq {
        page: page,
        page_size: page_size,
        uri: uri,
        order_by: order_by,
        order_direction: order,
    };
    let mut ack = storage::get_files(&site, req).await?;
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
    app: AppHandle,
    name: String,
    path: String,
    page: u64,
    page_size: u64,
    order_by: String,
    order: String,
) -> JsonResult<GetFilesAck> {
    let uri = format!("{}?name={}&name_op_or=&case_folding=", path, name);

    let site = get_token(&app).await?;
    let req = GetFilesReq {
        page: page,
        page_size: page_size,
        uri: uri,
        order_by: order_by,
        order_direction: order,
    };
    storage::get_files(&site, req).await
}

#[tauri::command]
async fn get_url(app: AppHandle, uri: String) -> JsonResult<String> {
    let site = get_token(&app).await?;
    let ack = storage::batch_urls(&site, vec![uri.clone()]).await;
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
async fn get_thumb_url(app: AppHandle, uri: String) -> JsonResult<String> {
    let site = get_token(&app).await?;
    storage::get_thumb_url(&site, uri).await
}

pub async fn get_token(app: &AppHandle) -> Result<Site> {
    let now = Utc::now();
    let store = app.store("app_data.json").unwrap();
    let addr = get_address(app).await?;
    if addr.is_empty() {
        return Err(AppError::Unauthorized);
    }
    match store.get("token") {
        Some(value) => {
            match serde_json::from_value::<Token>(value.clone()) {
                Ok(token) => {
                    if token.access_expires > now && !token.access_token.is_empty() {
                        return Ok(Site {
                            token: token.access_token,
                            addr,
                        });
                    }

                    if token.refresh_expires > now {
                        match login::refresh_token(Site {
                            token: token.refresh_token,
                            addr: addr.clone(),
                        })
                        .await
                        {
                            Ok(token) => {
                                let json_token = json!(token);
                                store.set("token", json_token.clone());
                                return Ok(Site {
                                    token: token.access_token,
                                    addr,
                                });
                            }
                            Err(err) => {
                                println!("{}", err);
                                return Err(AppError::Unauthorized);
                            }
                        }
                    }
                }
                Err(err) => {
                    println!("{}", err);
                    return Err(AppError::Unauthorized);
                }
            }
            return Err(AppError::Unauthorized);
        }
        None => {
            return Err(AppError::Unauthorized);
        }
    }
}

#[derive(Clone, Serialize)]
pub struct SessionChaged {
    logined: bool,
}

#[tauri::command]
async fn rename_file(app: AppHandle, new_name: String, uri: String) -> JsonResult<FileDetailsInfo> {
    let site = get_token(&app).await?;
    let ack = storage::rename(&site, new_name, uri).await;
    match ack {
        Ok(ack) => {
            return Ok(ack);
        }
        Err(e) => return Err(e),
    }
}

#[tauri::command]
async fn delete_file(
    app: AppHandle,
    unlink: bool,
    soft_delete: bool,
    uri: String,
) -> JsonResult<Option<DeleteFileAck>> {
    let site = get_token(&app).await?;
    storage::delete_file(&site, unlink, soft_delete, vec![uri]).await
}

#[tauri::command]
async fn create_folder(app: AppHandle, uri: String) -> JsonResult<FileInfo> {
    let site = get_token(&app).await?;
    storage::create_folder(&site, &uri).await
}

#[tauri::command]
async fn get_file_info(app: AppHandle, uri: String) -> JsonResult<FileDetailsInfo> {
    let site = get_token(&app).await?;
    storage::get_file_info(&site, &uri).await
}

#[tauri::command]
async fn share_file(
    app: AppHandle,
    downloads: u64,
    expire: u64,
    is_private: bool,
    uri: String,
) -> JsonResult<String> {
    let site = get_token(&app).await?;
    storage::share_file(&site, downloads, expire, is_private, &uri).await
}

#[tauri::command]
async fn get_file_source(app: AppHandle, uri: String) -> JsonResult<String> {
    let site = get_token(&app).await?;
    let ack = storage::get_file_source(&site, vec![uri]).await?;
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
async fn get_paths(app: AppHandle) -> JsonResult<Vec<String>> {
    let paths = vec![
        app.path()
            .public_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        app.path()
            .video_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        app.path()
            .local_data_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        app.path()
            .home_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    ];
    for v in &paths {
        println!("path: {}", v.to_string());
    }

    Ok(paths)
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
    app: AppHandle,
    id: u32,
    url: String,
    file_path: String,
    headers: Option<HashMap<String, String>>,
) -> JsonResult<u32> {
    transfer::upload(&app, id, &url, &file_path, headers).await
}

#[tauri::command]
async fn pre_upload(app: AppHandle, target_path: String, policy_id: String) -> JsonResult<Upload> {
    let site = get_token(&app).await?;
    let id = gen_id(&app);

    match app.dialog().file().blocking_pick_file() {
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
            let ack =
                storage::upload_file_session(&site, &mime_type, &uri, size, &policy_id).await?;
            let upload = Upload {
                id: id,
                file_path: file_path.to_string_lossy().to_string(),
                filename: filename.to_string(),
                uri: uri.clone(),
                upload_url: format!(
                    "{}/{}/0",
                    proto::get_api_url(&site.addr, "/file/upload"),
                    ack.session_id
                ),
            };
            return Ok(upload);
        }
        None => {
            return Err(AppError::NoData);
        }
    }
}

#[tauri::command]
async fn pre_download(app: AppHandle, uri: String, filename: String) -> JsonResult<Download> {
    let site = get_token(&app).await?;
    let ack = storage::batch_urls(&site, vec![uri.clone()]).await;
    let mut download_url = "".to_string();
    match ack {
        Ok(ack) => {
            for v in ack.urls {
                download_url = v.url;
            }
        }
        Err(e) => return Err(e),
    }
    let saved_dir = get_downloads_dir(&app)?;
    if download_url.is_empty() || saved_dir.is_empty() {
        return Err(AppError::Anyhow(anyhow::format_err!(
            "fail, dir:{}, url:{}",
            saved_dir,
            download_url
        )));
    }
    let id = gen_id(&app);

    Ok(Download {
        id: id,
        file_path: path::Path::new(&saved_dir)
            .join(filename)
            .to_string_lossy()
            .to_string(),
        url: download_url,
    })
}

fn gen_id(app: &AppHandle) -> u32 {
    let id = IDGEN.fetch_add(1, Ordering::SeqCst);

    if let Ok(store) = app.store("id.json") {
        store.set("transfer", serde_json::json!(id));
    }
    return id;
}
#[tauri::command]
async fn restore_file(app: AppHandle, uri: String) -> JsonResult<bool> {
    let site = get_token(&app).await?;
    storage::restore_file(&site, vec![uri.clone()]).await
}
#[tauri::command]
async fn delete_lock(app: AppHandle, tokens: Vec<String>) -> JsonResult<bool> {
    let token = get_token(&app).await?;
    storage::delete_lock(&token, tokens).await
}
#[tauri::command]
async fn get_shares(app: AppHandle, order_direction: String) -> JsonResult<GetSharesAck> {
    let site = get_token(&app).await?;
    storage::get_shares(&site, &order_direction).await
}

#[tauri::command]
async fn move_file(app: AppHandle, copy: bool, dst: String, uri: String) -> JsonResult<bool> {
    let token = get_token(&app).await?;
    storage::move_file(&token, vec![uri], &dst, copy).await
}
#[tauri::command]
async fn delete_share(app: AppHandle, id: String) -> JsonResult<bool> {
    let site = get_token(&app).await?;
    storage::delete_share(&site, &id).await
}
#[tauri::command]
async fn update_share(
    app: AppHandle,
    id: String,
    downloads: u8,
    expire: u64,
    uri: String,
) -> JsonResult<String> {
    let site = get_token(&app).await?;
    storage::update_share(&site, &id, downloads, expire, &uri).await
}
#[tauri::command]
async fn get_share_info(app: AppHandle, id: String, owner_extended: bool) -> JsonResult<ShareInfo> {
    let site = get_token(&app).await?;
    storage::get_share_info(&site, &id, owner_extended).await
}
