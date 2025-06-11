use reqwest::{Client, Method};

use crate::proto::{
    self,
    file::{DeleteFileAck, DeleteTokenReq},
    storage::{
        BatchUrisReq, DeleteFileReq, FileDetailsInfo, FileSrouce, GetFileSourceReq, GetFilesAck,
        GetFilesReq, MoveReq, RenameReq, ShareReq,
    },
    Site,
};

use crate::proto::Result;

pub async fn get_file_source(site: &Site, uris: Vec<String>) -> Result<FileSrouce> {
    let req = GetFileSourceReq { uris: uris };
    let ack = site
        .build(Method::PUT, "/file/source")
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<Vec<FileSrouce>>>()
        .await?;
    if ack.code == 0 {
        for v in ack.data.unwrap() {
            return Ok(v);
        }
        return Err(proto::AppError::Message(ack.code, ack.msg));
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn delete_lock(site: &Site, tokens: Vec<String>) -> Result<bool> {
    let req = DeleteTokenReq { tokens };
    let ack = site
        .build(Method::DELETE, "/file/token")
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<String>>()
        .await?;
    if ack.code == 0 {
        return Ok(true);
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn restore_file(site: &Site, uris: Vec<String>) -> Result<bool> {
    let req = BatchUrisReq { uris };
    let ack = site
        .build(Method::POST, "/file/restore")
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<String>>()
        .await?;
    if ack.code == 0 {
        return Ok(true);
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn get_files(site: &Site, req: GetFilesReq) -> Result<GetFilesAck> {
    let ack = site
        .build_query(Method::GET, "/file", req)
        .send()
        .await?
        .json::<proto::Ack<GetFilesAck>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn move_file(site: &Site, uris: Vec<String>, dst: &str, copy: bool) -> Result<bool> {
    let req = MoveReq {
        copy,
        dst: dst.to_string(),
        uris,
    };
    let ack = site
        .build(Method::POST, "/file/move")
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<String>>()
        .await?;
    if ack.code == 0 {
        return Ok(true);
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn delete_file(
    site: &Site,
    unlink: bool,
    soft_delete: bool,
    uris: Vec<String>,
) -> Result<Option<DeleteFileAck>> {
    let req = DeleteFileReq {
        unlink: unlink,
        skip_soft_delete: !soft_delete,
        uris: uris,
    };
    let ack = site
        .build(Method::DELETE, "/file")
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<DeleteFileAck>>()
        .await?;
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

pub async fn rename(site: &Site, name: String, uri: String) -> Result<FileDetailsInfo> {
    let req = RenameReq {
        new_name: name,
        uri: uri,
    };
    let ack = site
        .build(Method::POST, "/file/rename")
        .json(&req)
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
