use reqwest::{Client, Method};

use crate::proto::{
    self,
    share::{GetSharesAck, GetSharesReq, ShareInfo},
    storage::ShareReq,
};

use crate::hc::Site;
use crate::proto::Result;

pub async fn share_file(
    site: &Site,
    downloads: u64,
    expire: u64,
    is_private: bool,
    uri: &str,
) -> Result<String> {
    let mut req = ShareReq {
        uri: uri.to_string(),
        downloads: None,
        expire: None,
        is_private: None,
    };
    if is_private {
        req.is_private = Some(true);
    }
    if downloads > 0 {
        req.downloads = Some(downloads);
    }
    if expire > 0 {
        req.expire = Some(expire);
    }
    let response = site
        .build(Method::PUT, "/share")
        .await?
        .json(&req)
        .send()
        .await?;
    let body = response.text().await?;
    println!("response: {}", &body);

    let ack = serde_json::from_str::<proto::Ack<String>>(&body)?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn get_shares(site: &Site, order_direction: &str) -> Result<GetSharesAck> {
    let req = GetSharesReq {
        page_size: 50,
        order_direction: order_direction.to_string(),
    };
    let ack = site
        .build_query(Method::GET, "/share", req)
        .await?
        .send()
        .await?
        .json::<proto::Ack<GetSharesAck>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn delete_share(site: &Site, id: &str) -> Result<bool> {
    let ack = site
        .build(Method::DELETE, &format!("/share/{}", id))
        .await?
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

pub async fn update_share(
    site: &Site,
    id: &str,
    downloads: u8,
    expire: u64,
    uri: &str,
) -> Result<String> {
    let mut req = ShareReq {
        downloads: None,
        expire: None,
        is_private: None,
        uri: uri.to_string(),
    };
    if downloads > 0 {
        req.downloads = Some(downloads as u64);
    }
    if expire > 0 {
        req.expire = Some(expire)
    }
    let ack = site
        .build(Method::POST, &format!("/share/{}", id))
        .await?
        .json(&req)
        .send()
        .await?
        .json::<proto::Ack<String>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}

pub async fn get_share_info(site: &Site, id: &str, owner_extended: bool) -> Result<ShareInfo> {
    let ack = site
        .build(
            Method::GET,
            &format!("/share/info/{}?owner_extended={}", id, owner_extended),
        )
        .await?
        .send()
        .await?
        .json::<proto::Ack<ShareInfo>>()
        .await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    } else {
        return Err(proto::AppError::Message(ack.code, ack.msg));
    }
}
