use std::str::FromStr;

use http::Uri;
use reqwest::Method;

use crate::{
    hc::{http_client_manager::HttpClientWrapper, Request},
    proto::{
        self,
        file::{DeleteFileAck, DeleteTokenReq},
        settings::NetworkMode,
        storage::{
            BatchUrisReq, BatchUrlsAck, CreateFileReq, DeleteFileReq, FileDetailsInfo, FileInfo,
            FileSrouce, GetFileInfoReq, GetFileSourceReq, GetFilesAck, GetFilesReq, GetThumbURLAck,
            MoveReq, RenameReq, UploadSessionAck, UploadSessionReq, Url,
        },
    },
};

use crate::proto::Result;

impl super::AppState {
    pub async fn get_file_source(&self, uris: Vec<String>) -> Result<FileSrouce> {
        let req = GetFileSourceReq { uris: uris };
        let req = Request::new(Method::PUT, "/file/source").with_body(());
        let resp = self
            .get_client()
            .await?
            .put::<(), proto::Ack<Vec<FileSrouce>>>(req)
            .await?;
        let ack = resp.json::<proto::Ack<Vec<FileSrouce>>>().await?;
        if ack.code == 0 {
            for v in ack.data.unwrap() {
                return Ok(v);
            }
        }
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }

    pub async fn delete_lock(&self, tokens: Vec<String>) -> Result<bool> {
        let req = DeleteTokenReq { tokens };
        let resp = self
            .request_json(Method::DELETE, "/file/token", req)
            .await?;
        let ack = resp.json::<proto::Ack<String>>().await?;
        if ack.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn restore_file(&self, uris: Vec<String>) -> Result<bool> {
        let req = BatchUrisReq { uris };
        let resp = self
            .request_json(Method::POST, "/file/restore", req)
            .await?;
        let ack = resp.json::<proto::Ack<String>>().await?;
        if ack.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_files(&self, req: GetFilesReq) -> Result<GetFilesAck> {
        let resp = self.request_query(Method::GET, "/file", req).await?;
        let ack = resp.json::<proto::Ack<GetFilesAck>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn move_file(&self, uris: Vec<String>, dst: &str, copy: bool) -> Result<bool> {
        let req = MoveReq {
            copy,
            dst: dst.to_string(),
            uris,
        };
        let resp = self.request_json(Method::POST, "/file/move", req).await?;
        let ack = resp.json::<proto::Ack<String>>().await?;
        if ack.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn delete_file(
        &self,
        unlink: bool,
        soft_delete: bool,
        uris: Vec<String>,
    ) -> Result<Option<DeleteFileAck>> {
        let req = DeleteFileReq {
            unlink: unlink,
            skip_soft_delete: !soft_delete,
            uris: uris,
        };
        let resp = self.request_json(Method::DELETE, "/file", req).await?;
        let ack = resp.json::<proto::Ack<DeleteFileAck>>().await?;
        match ack.code {
            0 => {
                return Ok(None);
            }
            40073 => {
                return Ok(ack.data);
            }
            _ => {
                return Err(proto::AppError::Message(ack.code, ack.msg));
            }
        }
    }

    pub async fn rename(&self, name: String, uri: String) -> Result<FileDetailsInfo> {
        let req = RenameReq {
            new_name: name,
            uri: uri,
        };
        let resp = self.request_json(Method::POST, "/file/rename", req).await?;
        let ack = resp.json::<proto::Ack<FileDetailsInfo>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }
    pub async fn batch_urls(&self, urls: Vec<String>) -> Result<BatchUrlsAck> {
        let req: BatchUrisReq = BatchUrisReq { uris: urls };
        let (resp, addr) = self
            .request_json_addr(Method::POST, "/file/url", req)
            .await?;
        let ack = resp.json::<proto::Ack<BatchUrlsAck>>().await?;
        if ack.code == 0 {
            if let Some(mut data) = ack.data {
                let settings = self.get_addr()?;
                if settings.mode == NetworkMode::P2P {
                    let mut urls = Vec::with_capacity(data.urls.len());
                    for (_, url) in data.urls.iter().enumerate() {
                        if let Ok(uri) = Uri::from_str(&url.url) {
                            let url = format!(
                                "{}{}",
                                addr.clone().unwrap_or(settings.get_addr()),
                                uri.path_and_query().unwrap().as_str()
                            );
                            urls.push(Url { url });
                        }
                    }
                    data.urls = urls;
                }
                return Ok(data);
            }
            return Ok(BatchUrlsAck {
                expires: String::new(),
                urls: Vec::new(),
            });
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_thumb_url(&self, uri: String) -> Result<String> {
        let (resp, addr) = self
            .request_addr(Method::GET, &format!("/file/thumb?uri={}", uri))
            .await?;
        let ack = resp.json::<proto::Ack<GetThumbURLAck>>().await?;
        if ack.code == 0 {
            if let Some(data) = ack.data {
                if let Ok(uri) = Uri::from_str(&data.url) {
                    let settings = self.get_addr()?;
                    if settings.mode == NetworkMode::P2P {
                        let url = format!(
                            "{}{}",
                            addr.unwrap_or(settings.get_addr()),
                            uri.path_and_query().unwrap().as_str()
                        );
                        return Ok(url);
                    }
                }
                return Ok(data.url);
            }

            return Ok(String::new());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn create_folder(&self, uri: &str) -> Result<FileInfo> {
        let req = CreateFileReq {
            file_type: "folder".to_string(),
            err_on_conflict: true,
            uri: uri.to_string(),
        };
        let resp = self.request_json(Method::POST, "/file/create", req).await?;
        let ack = resp.json::<proto::Ack<FileInfo>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_file_info(&self, uri: &str) -> Result<FileDetailsInfo> {
        let req = GetFileInfoReq {
            uri: uri.to_string(),
            extended: true,
        };
        let resp = self.request_json(Method::GET, "/file/info", req).await?;
        let ack = resp.json::<proto::Ack<FileDetailsInfo>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn upload_file_session(
        &self,
        mime_type: &str,
        uri: &str,
        size: u64,
        policy_id: &str,
    ) -> Result<UploadSessionAck> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let req = UploadSessionReq {
            uri: uri.to_string(),
            size: size,
            policy_id: policy_id.to_string(),
            last_modified: now,
            mime_type: mime_type.to_string(),
        };

        let response = self.request_json(Method::PUT, "/file/upload", req).await?;
        let ack = response.json::<proto::Ack<UploadSessionAck>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        }
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}
