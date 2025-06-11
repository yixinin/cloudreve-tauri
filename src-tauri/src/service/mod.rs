use crate::models::Photo;
use android_media_scanner::MediaScanner;
use chrono::Utc;
use std::time::Duration;
use tauri::Manager;
use tokio::time;

pub mod uploader;

pub use uploader::Uploader;

pub struct SyncService {
    app_handle: tauri::AppHandle,
    scan_interval: Duration,
}

impl SyncService {
    pub fn new(app_handle: tauri::AppHandle, scan_interval: Duration) -> Self {
        Self {
            app_handle,
            scan_interval,
        }
    }

    pub async fn start(&self) {
        let mut interval = time::interval(self.scan_interval);

        loop {
            interval.tick().await;
            if let Err(e) = self.scan_and_sync().await {
                log::error!("Sync error: {}", e);
            }
        }
    }

    async fn scan_and_sync(&self) -> Result<(), String> {
        // 使用安卓媒体扫描器获取新照片
        let scanner = MediaScanner::new();
        let photos = scanner.query_images().map_err(|e| e.to_string())?;

        // 过滤出未同步的照片
        let unsynced_photos = photos
            .into_iter()
            .filter(|p| !p.is_synced)
            .collect::<Vec<_>>();

        // 通知前端有新照片需要同步
        self.app_handle
            .emit_all(
                "new_photos",
                json!({
                    "count": unsynced_photos.len(),
                    "photos": unsynced_photos
                }),
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
