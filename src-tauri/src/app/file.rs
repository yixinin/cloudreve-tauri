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
        let req = Request::new(Method::PUT, &self.base_url, "/file/source").with_body(());
        let resp = self
            .get_client()
            .await?
            .put::<(), proto::Ack<Vec<FileSrouce>>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            for v in ack.data.unwrap() {
                return Ok(v);
            }
        }
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }

    pub async fn delete_lock(&self, tokens: Vec<String>) -> Result<bool> {
        let req = Request::new(Method::DELETE, &self.base_url, "/file/token").with_body(());
        let resp = self
            .get_client()
            .await?
            .delete::<proto::Ack<String>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn restore_file(&self, uris: Vec<String>) -> Result<bool> {
        let req = BatchUrisReq { uris };
        let req = Request::new(Method::POST, &self.base_url, "/file/restore").with_body(req);
        let resp = self
            .get_client()
            .await?
            .post::<_, proto::Ack<String>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_files(&self, req: GetFilesReq) -> Result<GetFilesAck> {
        let req = Request::new(Method::GET, &self.base_url, "/file").with_body(());
        let resp = self
            .get_client()
            .await?
            .get::<proto::Ack<GetFilesAck>>(req)
            .await?;
        let ack = resp.into_data();
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
        let req = Request::new(Method::POST, &self.base_url, "/file/move").with_body(req);
        let resp = self
            .get_client()
            .await?
            .post::<_, proto::Ack<String>>(req)
            .await?;
        let ack = resp.into_data();
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
        let req = Request::new(
            Method::DELETE,
            &self.base_url,
            &format!(
                "/file?unlink={}&skip_soft_delete={}&uris={:?}",
                unlink, !soft_delete, uris
            ),
        )
        .with_body(());
        let resp = self
            .get_client()
            .await?
            .delete::<proto::Ack<DeleteFileAck>>(req)
            .await?;
        let ack = resp.into_data();
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
        let req = Request::new(Method::POST, &self.base_url, "/file/rename").with_body(req);
        let resp = self
            .get_client()
            .await?
            .post::<_, proto::Ack<FileDetailsInfo>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }
    pub async fn batch_urls(&self, urls: Vec<String>) -> Result<BatchUrlsAck> {
        let req: BatchUrisReq = BatchUrisReq { uris: urls };
        let req = Request::new(Method::POST, &self.base_url, "/file/url").with_body(req);
        let resp = self
            .get_client()
            .await?
            .post::<_, proto::Ack<BatchUrlsAck>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            if let Some(mut data) = ack.data {
                let settings = self.get_addr()?;
                if settings.mode == NetworkMode::P2P {
                    let mut urls = Vec::with_capacity(data.urls.len());
                    for (_, url) in data.urls.iter().enumerate() {
                        if let Ok(uri) = Uri::from_str(&url.url) {
                            let url = format!(
                                "{}{}",
                                settings.get_addr(),
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
        let req = Request::new(
            Method::GET,
            &self.base_url,
            &format!("/file/thumb?uri={}", uri),
        )
        .with_body(());
        let resp = self
            .get_client()
            .await?
            .get::<proto::Ack<GetThumbURLAck>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            if let Some(data) = ack.data {
                if let Ok(uri) = Uri::from_str(&data.url) {
                    let settings = self.get_addr()?;
                    if settings.mode == NetworkMode::P2P {
                        let url = format!(
                            "{}{}",
                            settings.get_addr(),
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
        let req = Request::new(Method::POST, &self.base_url, "/file/create").with_body(req);
        let resp = self
            .get_client()
            .await?
            .post::<_, proto::Ack<FileInfo>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_file_info(&self, uri: &str) -> Result<FileDetailsInfo> {
        let req = Request::new(
            Method::GET,
            &self.base_url,
            &format!("/file/info?uri={}&extended=true", uri),
        )
        .with_body(());
        let resp = self
            .get_client()
            .await?
            .get::<proto::Ack<FileDetailsInfo>>(req)
            .await?;
        let ack = resp.into_data();
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

        let req = Request::new(Method::PUT, &self.base_url, "/file/upload").with_body(req);
        let response = self
            .get_client()
            .await?
            .put::<_, proto::Ack<UploadSessionAck>>(req)
            .await?;
        let ack = response.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        }
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}
