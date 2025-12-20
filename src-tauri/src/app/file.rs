use std::str::FromStr;

use http::Uri;
use reqwest::Method;

use crate::{
    hc::http_client_manager::HttpClientWrapper,
    proto::{
        self,
        file::DeleteFileAck,
        settings::NetworkMode,
        storage::{
            BatchUrisReq, BatchUrlsAck, CreateFileReq, DeleteFileReq, FileDetailsInfo, FileInfo,
            FileSrouce, GetFileSourceReq, GetFilesAck, GetFilesReq, GetThumbURLAck, MoveReq,
            RenameReq, UploadSessionAck, UploadSessionReq, Url,
        },
    },
};

use crate::proto::Result;

impl super::AppState {
    pub async fn get_file_source(&self, uris: Vec<String>) -> Result<FileSrouce> {
        let req = GetFileSourceReq { uris: uris };
        let req = self.request_with_query(Method::PUT, "/file/source", req)?;
        let resp = self
            .get_client()
            .await?
            .put::<proto::Ack<Vec<FileSrouce>>>(req)
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
        let req = self.request_with_query(
            Method::DELETE,
            "/file/token",
            proto::file::DeleteTokenReq { tokens },
        )?;
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
        let req = self.request_with_body(Method::POST, "/file/restore", req)?;
        let resp = self
            .get_client()
            .await?
            .post::<proto::Ack<String>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_files(&self, req: GetFilesReq) -> Result<GetFilesAck> {
        let req = self.request_with_query(Method::GET, "/file", req)?;
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
        let req = self.request_with_body(Method::POST, "/file/move", req)?;
        let resp = self
            .get_client()
            .await?
            .post::<proto::Ack<String>>(req)
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
        let req = self.request_with_query(
            Method::DELETE,
            "/file",
            DeleteFileReq {
                unlink,
                skip_soft_delete: !soft_delete,
                uris,
            },
        )?;
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
        let req = self.request_with_body(Method::POST, "/file/rename", req)?;
        let resp = self
            .get_client()
            .await?
            .post::<proto::Ack<FileDetailsInfo>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }
    pub async fn batch_urls(&self, urls: Vec<String>) -> Result<BatchUrlsAck> {
        let client = self.get_client().await?;
        self.batch_urls_with_client(urls, client).await
    }

    pub async fn batch_urls_with_client(
        &self,
        urls: Vec<String>,
        client: crate::hc::http_client_manager::HttpClientDispatcher,
    ) -> Result<BatchUrlsAck> {
        let req: BatchUrisReq = BatchUrisReq { uris: urls };
        let req = self.request_with_body(Method::POST, "/file/url", req)?;
        let resp = client.post::<proto::Ack<BatchUrlsAck>>(req).await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            if let Some(data) = ack.data {
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
        let client = self.get_client().await?;
        self.get_thumb_url_with_client(uri, client).await
    }

    pub async fn get_thumb_url_with_client(
        &self,
        uri: String,
        client: crate::hc::http_client_manager::HttpClientDispatcher,
    ) -> Result<String> {
        let req = self.request_with_query(
            Method::GET,
            "/file/thumb",
            std::collections::HashMap::from([("uri", uri)]),
        )?;
        let resp = client.get::<proto::Ack<GetThumbURLAck>>(req).await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            if let Some(data) = ack.data {
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
        let req = self.request_with_body(Method::POST, "/file/create", req)?;
        let resp = self
            .get_client()
            .await?
            .post::<proto::Ack<FileInfo>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_file_info(&self, uri: &str) -> Result<FileDetailsInfo> {
        let req = self.request_with_query(
            Method::GET,
            "/file/info",
            std::collections::HashMap::from([
                ("uri", uri.to_string()),
                ("extended", "true".to_string()),
            ]),
        )?;
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

        let req = self.request_with_body(Method::PUT, "/file/upload", req)?;
        let response = self
            .get_client()
            .await?
            .put::<proto::Ack<UploadSessionAck>>(req)
            .await?;
        let ack = response.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        }
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}
