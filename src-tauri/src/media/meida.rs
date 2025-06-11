// src-tauri/src/main.rs
use chrono::{DateTime, Local};
use image::{EncodableLayout, ImageFormat};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
struct MediaItem {
    id: String,
    uri: String,
    r#type: String, // "image" or "video"
    width: Option<i32>,
    height: Option<i32>,
    date_taken: String, // ISO 8601 format
    size: u64,
    duration: Option<f64>, // video duration in seconds
    mime_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanOptions {
    paths: Vec<String>,       // 要扫描的路径
    max_depth: Option<usize>, // 最大扫描深度
    include_images: bool,
    include_videos: bool,
}

// 获取文件创建/修改时间
fn get_file_time(path: &Path) -> SystemTime {
    path.metadata()
        .and_then(|m| m.modified())
        .unwrap_or_else(|_| SystemTime::now())
}

// 获取图片尺寸
fn get_image_dimensions(path: &Path) -> Option<(i32, i32)> {
    if let Ok(dimensions) = image::image_dimensions(path) {
        Some((dimensions.0 as i32, dimensions.1 as i32))
    } else {
        None
    }
}

// 获取视频信息 (需要安装ffmpeg)
#[cfg(feature = "ffmpeg")]
fn get_video_info(path: &Path) -> Option<(i32, i32, f64)> {
    use ffmpeg_next::format::input;
    use ffmpeg_next::media::Type as MediaType;

    if let Ok(context) = input(path) {
        let stream = context.streams().best(MediaType::Video)?;
        let codec = stream.codec();
        let duration = stream.duration() as f64 * f64::from(stream.time_base());

        Some((codec.width(), codec.height(), duration))
    } else {
        None
    }
}

#[cfg(not(feature = "ffmpeg"))]
fn get_video_info(_path: &Path) -> Option<(i32, i32, f64)> {
    None
}

// 判断是否是图片文件
fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => true,
            _ => false,
        }
    } else {
        false
    }
}

// 判断是否是视频文件
fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        match ext.to_lowercase().as_str() {
            "mp4" | "mkv" | "avi" | "mov" | "webm" | "3gp" => true,
            _ => false,
        }
    } else {
        false
    }
}

// 获取文件MIME类型
fn get_mime_type(path: &Path) -> String {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => "image/jpeg".to_string(),
            "png" => "image/png".to_string(),
            "gif" => "image/gif".to_string(),
            "bmp" => "image/bmp".to_string(),
            "webp" => "image/webp".to_string(),
            "mp4" => "video/mp4".to_string(),
            "mkv" => "video/x-matroska".to_string(),
            "avi" => "video/x-msvideo".to_string(),
            "mov" => "video/quicktime".to_string(),
            "webm" => "video/webm".to_string(),
            "3gp" => "video/3gpp".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    } else {
        "application/octet-stream".to_string()
    }
}

#[tauri::command]
async fn scan_media_files(options: ScanOptions) -> Result<Vec<MediaItem>, String> {
    let mut media_items = Vec::new();
    let max_depth = options.max_depth.unwrap_or(usize::MAX);

    for path_str in options.paths {
        let path = PathBuf::from(&path_str);

        for entry in WalkDir::new(path)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            if entry_path.is_file() {
                let is_image = options.include_images && is_image_file(entry_path);
                let is_video = options.include_videos && is_video_file(entry_path);

                if is_image || is_video {
                    let file_name = entry_path
                        .file_name()
                        .and_then(OsStr::to_str)
                        .unwrap_or("unknown")
                        .to_string();

                    let file_time = get_file_time(entry_path);
                    let datetime: DateTime<Local> = file_time.into();

                    let size = entry_path.metadata().map(|m| m.len()).unwrap_or(0);

                    let mime_type = get_mime_type(entry_path);
                    let file_path = entry_path.to_string_lossy().into_owned();

                    if is_image {
                        let (width, height) = get_image_dimensions(entry_path).unwrap_or((0, 0));

                        media_items.push(MediaItem {
                            id: format!("img-{}", generate_file_id(&file_path)),
                            uri: format!("file://{}", &file_path),
                            r#type: "image".to_string(),
                            width: Some(width),
                            height: Some(height),
                            date_taken: datetime.to_rfc3339(),
                            size,
                            duration: None,
                            mime_type,
                        });
                    } else if is_video {
                        let (width, height, duration) =
                            get_video_info(entry_path).unwrap_or((0, 0, 0.0));

                        media_items.push(MediaItem {
                            id: format!("vid-{}", generate_file_id(&file_path)),
                            uri: format!("file://{}", &file_path),
                            r#type: "video".to_string(),
                            width: Some(width),
                            height: Some(height),
                            date_taken: datetime.to_rfc3339(),
                            size,
                            duration: Some(duration),
                            mime_type,
                        });
                    }
                }
            }
        }
    }

    // 按日期从新到旧排序
    media_items.sort_by(|a, b| b.date_taken.cmp(&a.date_taken));

    Ok(media_items)
}

fn generate_file_id(path: &str) -> String {
    let hash = md5::compute(path);
    format!("{:x}", hash)
}
