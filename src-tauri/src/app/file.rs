use reqwest::Method;
use std::sync::Arc;
use url::Url;

use crate::proto::{
    self,
    file::DeleteFileAck,
    storage::{
        BatchUrisReq, BatchUrlsAck, CreateFileReq, DeleteFileReq, FileDetailsInfo, FileInfo,
        FileSrouce, GetFileSourceReq, GetFilesAck, GetFilesReq, GetThumbURLAck, MoveReq, RenameReq,
        UploadSessionAck, UploadSessionReq,
    },
};

use crate::proto::Result;

impl super::AppState {
    pub async fn get_file_source(&self, uris: Vec<String>) -> Result<FileSrouce> {
        let req = GetFileSourceReq { uris: uris };
        let req = self.request_with_query(Method::PUT, "/file/source", req)?;
        let resp = req
            .send()
            .await?
            .json::<proto::Ack<Vec<FileSrouce>>>()
            .await?;
        if resp.code == 0 {
            for v in resp.data.unwrap() {
                return Ok(v);
            }
        }
        return Err(proto::AppError::Message(resp.code, resp.msg));
    }

    pub async fn delete_lock(&self, tokens: Vec<String>) -> Result<bool> {
        let req = self.request_with_query(
            Method::DELETE,
            "/file/token",
            proto::file::DeleteTokenReq { tokens },
        )?;
        let resp = req.send().await?.json::<proto::Ack<String>>().await?;
        if resp.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn restore_file(&self, uris: Vec<String>) -> Result<bool> {
        let req = BatchUrisReq { uris };
        let req = self.request_with_body(Method::POST, "/file/restore", req)?;
        let resp = req.send().await?.json::<proto::Ack<String>>().await?;
        if resp.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn get_files(&self, req: GetFilesReq) -> Result<GetFilesAck> {
        let req = self.request_with_query(Method::GET, "/file", req)?;
        let resp = req.send().await?.json::<proto::Ack<GetFilesAck>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn move_file(&self, uris: Vec<String>, dst: &str, copy: bool) -> Result<bool> {
        let req = MoveReq {
            copy,
            dst: dst.to_string(),
            uris,
        };
        let req = self.request_with_body(Method::POST, "/file/move", req)?;
        let resp = req.send().await?.json::<proto::Ack<String>>().await?;
        if resp.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
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
        let resp = req
            .send()
            .await?
            .json::<proto::Ack<DeleteFileAck>>()
            .await?;
        match resp.code {
            0 => {
                return Ok(None);
            }
            40073 => {
                return Ok(resp.data);
            }
            _ => {
                return Err(proto::AppError::Message(resp.code, resp.msg));
            }
        }
    }

    pub async fn rename(&self, name: String, uri: String) -> Result<FileDetailsInfo> {
        let req = RenameReq {
            new_name: name,
            uri: uri,
        };
        let req = self.request_with_body(Method::POST, "/file/rename", req)?;
        let resp = req
            .send()
            .await?
            .json::<proto::Ack<FileDetailsInfo>>()
            .await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }
    pub async fn batch_urls(&self, urls: Vec<String>) -> Result<BatchUrlsAck> {
        let client = self.get_client().await?;
        self.batch_urls_with_client(urls, client).await
    }

    pub async fn batch_urls_with_client(
        &self,
        urls: Vec<String>,
        client: Arc<reqwest::Client>,
    ) -> Result<BatchUrlsAck> {
        let req: BatchUrisReq = BatchUrisReq { uris: urls };
        let url = Url::parse(&self.base_url)?.join("/file/url")?;
        let mut req_builder = client.request(Method::POST, url);

        if let Ok(tokens) = self.get_token_sync() {
            req_builder = req_builder.header(
                "Authorization",
                format!("Bearer {}", tokens.access_token)
                    .parse::<reqwest::header::HeaderValue>()?,
            );
        }

        let resp = req_builder
            .json(&req)
            .send()
            .await?
            .json::<proto::Ack<BatchUrlsAck>>()
            .await?;
        if resp.code == 0 {
            if let Some(data) = resp.data {
                return Ok(data);
            }
            return Ok(BatchUrlsAck {
                expires: String::new(),
                urls: Vec::new(),
            });
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn get_thumb_url(&self, uri: String) -> Result<String> {
        let client = self.get_client().await?;
        self.get_thumb_url_with_client(uri, client).await
    }

    pub async fn get_thumb_url_with_client(
        &self,
        uri: String,
        client: Arc<reqwest::Client>,
    ) -> Result<String> {
        let query = std::collections::HashMap::from([("uri", uri)]);
        let base_url = Url::parse(&self.base_url)?;
        let path_url = base_url.join("/file/thumb")?;
        let url = format!("{}?{}", path_url, serde_urlencoded::to_string(query)?);
        let mut req_builder = client.request(Method::GET, url);

        if let Ok(tokens) = self.get_token_sync() {
            req_builder = req_builder.header(
                "Authorization",
                format!("Bearer {}", tokens.access_token)
                    .parse::<reqwest::header::HeaderValue>()?,
            );
        }

        let resp = req_builder
            .send()
            .await?
            .json::<proto::Ack<GetThumbURLAck>>()
            .await?;
        if resp.code == 0 {
            if let Some(data) = resp.data {
                return Ok(data.url);
            }

            return Ok(String::new());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn create_folder(&self, uri: &str) -> Result<FileInfo> {
        let req = CreateFileReq {
            file_type: "folder".to_string(),
            err_on_conflict: true,
            uri: uri.to_string(),
        };
        let req = self.request_with_body(Method::POST, "/file/create", req)?;
        let resp = req.send().await?.json::<proto::Ack<FileInfo>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
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
        let resp = req
            .send()
            .await?
            .json::<proto::Ack<FileDetailsInfo>>()
            .await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
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
        let response = req
            .send()
            .await?
            .json::<proto::Ack<UploadSessionAck>>()
            .await?;
        if response.code == 0 {
            return Ok(response.data.unwrap());
        }
        return Err(proto::AppError::Message(response.code, response.msg));
    }
}
