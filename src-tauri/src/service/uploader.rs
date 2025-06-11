use crate::models::{Photo, UploadSession};
use reqwest::Client;
use std::path::Path;
use tauri::Manager;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct Uploader {
    client: Client,
    app_handle: tauri::AppHandle,
}

impl Uploader {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            client: Client::new(),
            app_handle,
        }
    }

    pub async fn upload_photo(&self, photo: &Photo, session: &UploadSession) -> Result<(), String> {
        let file = File::open(&photo.uri).await.map_err(|e| e.to_string())?;
        let file_size = file.metadata().await.map_err(|e| e.to_string())?.len();

        // 分块上传逻辑
        let chunk_size = session.chunk_size as usize;
        let mut uploaded = 0;
        let mut buffer = vec![0; chunk_size];
        let mut reader = tokio::io::BufReader::new(file);

        while uploaded < file_size {
            let read_size = reader.read(&mut buffer).await.map_err(|e| e.to_string())?;
            if read_size == 0 {
                break;
            }

            // 实际上传逻辑
            let response = self
                .client
                .post(&session.uri)
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Range",
                    format!(
                        "bytes {}-{}/{}",
                        uploaded,
                        uploaded + read_size - 1,
                        file_size
                    ),
                )
                .body(buffer[..read_size].to_vec())
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !response.status().is_success() {
                return Err(format!("Upload failed with status: {}", response.status()));
            }

            uploaded += read_size as u64;

            // 通知前端进度
            self.app_handle
                .emit_all(
                    "upload_progress",
                    json!({
                        "photo_id": photo.id,
                        "progress": (uploaded as f64 / file_size as f64) * 100.0
                    }),
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}
