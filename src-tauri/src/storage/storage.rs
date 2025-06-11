use std::time;

use reqwest::{Client, Method};

use crate::proto::{
    self,
    storage::{
        BatchUrisReq, BatchUrlsAck, CreateFileReq, FileDetailsInfo, FileInfo, GetCapacityAck,
        GetFileInfoReq, GetThumbURLAck, UploadSessionAck, UploadSessionReq,
    },
    Site,
};

use crate::proto::Result;

pub async fn get_capacity(site: &Site) -> Result<GetCapacityAck> {
    let ack = site
        .build(Method::GET, "/user/capacity")
        .send()
        .await?
        .json::<proto::Ack<GetCapacityAck>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn batch_urls(site: &Site, urls: Vec<String>) -> Result<BatchUrlsAck> {
    let req: BatchUrisReq = BatchUrisReq { uris: urls };
    let ack = site
        .build(Method::POST, "/file/url")
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<BatchUrlsAck>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn get_thumb_url(site: &Site, uri: String) -> Result<String> {
    let ack = site
        .build(Method::GET, &format!("/file/thumb?uri={}", uri))
        .send()
        .await?
        .json::<proto::Ack<GetThumbURLAck>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap().url);
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn create_folder(site: &Site, uri: &str) -> Result<FileInfo> {
    let req = CreateFileReq {
        file_type: "folder".to_string(),
        err_on_conflict: true,
        uri: uri.to_string(),
    };
    let ack = site
        .build(Method::POST, "/file/create")
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<FileInfo>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn get_file_info(site: &Site, uri: &str) -> Result<FileDetailsInfo> {
    let req = GetFileInfoReq {
        uri: uri.to_string(),
        extended: true,
    };
    let ack = site
        .build_query(Method::GET, "/file/info", req)
        .send()
        .await?
        .json::<proto::Ack<FileDetailsInfo>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn upload_file_session(
    site: &Site,
    mime_type: &str,
    uri: &str,
    size: u64,
    policy_id: &str,
) -> Result<UploadSessionAck> {
    let now = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let req = UploadSessionReq {
        uri: uri.to_string(),
        size: size,
        policy_id: policy_id.to_string(),
        last_modified: now,
        mime_type: mime_type.to_string(),
    };

    let response = site
        .build(Method::PUT, "/file/upload")
        .json(&req)
        .send()
        .await?;
    let ack = response.json::<proto::Ack<UploadSessionAck>>().await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    }
    return Err(proto::AppError::Message(ack.code, ack.msg));
}
