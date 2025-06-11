use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetCapacityAck {
    pub total: u64,
    pub used: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetFilesReq {
    pub page: u64,
    pub page_size: u64,
    pub uri: String,
    pub order_by: String,
    pub order_direction: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GetFilesAck {
    pub files: Vec<FileItem>,
    pub file_tags: Option<Vec<Vec<FileTag>>>,
    pub parent: DirectoryItem,
    pub pagination: Pagination,
    pub props: Properties,
    pub context_hint: String,
    pub mixed_type: bool,
    pub storage_policy: Option<StoragePolicy>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FileItem {
    #[serde(rename = "type")]
    pub item_type: i32,
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub size: u64,
    pub metadata: Option<HashMap<String, String>>,
    pub path: String,
    pub capability: String,
    pub owned: bool,
    pub shared: Option<bool>,
    pub primary_entity: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTag {
    pub key: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DirectoryItem {
    #[serde(rename = "type")]
    pub item_type: i32,
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub size: u64,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub path: Option<String>,
    pub capability: Option<String>,
    pub owned: Option<bool>,
    pub primary_entity: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Pagination {
    pub page: i32,
    pub page_size: i32,
    pub is_cursor: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Properties {
    pub capability: String,
    pub max_page_size: i32,
    pub order_by_options: Option<Vec<String>>,
    pub order_direction_options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StoragePolicy {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub policy_type: String,
    pub max_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUrlsAck {
    pub expires: String,
    pub urls: Vec<Url>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Url {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchUrisReq {
    pub uris: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetThumbURLAck {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFileReq {
    #[serde(rename = "type")]
    pub file_type: String,
    pub err_on_conflict: bool,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    #[serde(rename = "type")]
    pub file_type: String,
    pub id: String,
    pub size: u64,
    pub name: String,
    pub path: String,
    pub owned: bool,
    pub capability: String,
    pub primary_entity: String,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenameReq {
    pub new_name: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetFileInfoReq {
    pub uri: String,
    pub extended: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDetailsInfo {
    #[serde(rename = "type")]
    pub type_field: i32,
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub size: i64,
    pub metadata: serde_json::Value,
    pub path: String,
    pub capability: String,
    pub owned: bool,
    pub primary_entity: String,
    pub extended_info: ExtendedInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtendedInfo {
    pub storage_policy: StoragePolicy,
    pub storage_used: i64,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub size: i64,
    #[serde(rename = "type")]
    pub type_field: i32,

    pub created_at: String,
    pub storage_policy: StoragePolicy,
    pub created_by: CreatedBy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatedBy {
    pub id: String,
    pub nickname: String,

    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFileReq {
    pub unlink: bool,
    pub skip_soft_delete: bool,
    pub uris: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadSessionReq {
    pub uri: String,
    pub size: u64,
    pub policy_id: String,
    pub last_modified: u64, // 时间戳（毫秒）
    pub mime_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadSessionAck {
    pub session_id: String,
    pub upload_id: String,
    pub chunk_size: u64,
    pub expires: u64, // Unix 时间戳（秒）
    pub storage_policy: StoragePolicy,
    pub uri: String,
    pub callback_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveReq {
    pub copy: bool,
    pub uris: Vec<String>,
    pub dst: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FileSrouce {
    pub file_url: String,
    pub link: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ShareReq {
    pub downloads: Option<u64>,
    pub expire: Option<u64>,
    pub is_private: Option<bool>,
    pub uri: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetFileSourceReq {
    pub uris: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct FileProgress {
    pub filename: String,
    pub total: usize,
    pub current: usize,
}

#[derive(Clone, Serialize)]
pub struct Download {
    pub id: u32,
    pub file_path: String,
    pub url: String,
}

#[derive(Clone, Serialize)]
pub struct Upload {
    pub id: u32,
    pub filename: String,
    pub file_path: String,
    pub uri: String,
    pub upload_url: String,
}

use std::path::Path;

pub fn get_mime_type(filename: &str) -> String {
    let mime_map: HashMap<&str, &str> = HashMap::from([
        ("txt", "text/plain"),
        ("jpg", "image/jpeg"),
        ("png", "image/png"),
        ("html", "text/html"),
        ("json", "application/json"),
        ("mp4", "video/mp4"),
        ("mp3", "audio/mpeg"),
        ("wav", "audio/wav"),
    ]);
    if let Some(extension) = Path::new(filename).extension().and_then(|ext| ext.to_str()) {
        let ext = extension.to_lowercase();

        if let Some(s) = mime_map.get(ext.as_str()).copied() {
            return s.to_string();
        }
    }

    return String::from("octet-stream");
}
