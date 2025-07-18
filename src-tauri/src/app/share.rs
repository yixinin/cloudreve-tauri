use reqwest::Method;

use crate::proto::{
    self,
    share::{GetSharesAck, GetSharesReq, ShareInfo},
    storage::ShareReq,
};

use crate::proto::Result;

impl super::AppState {
    pub async fn share_file(
        &self,
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
        let response = self.request_json(Method::PUT, "/share", req).await?;
        let body = response.text().await?;
        println!("response: {}", &body);

        let ack = serde_json::from_str::<proto::Ack<String>>(&body)?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_shares(&self, order_direction: &str) -> Result<GetSharesAck> {
        let req = GetSharesReq {
            page_size: 50,
            order_direction: order_direction.to_string(),
        };
        let resp = self.request_query(Method::GET, "/share", req).await?;
        let ack = resp.json::<proto::Ack<GetSharesAck>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn delete_share(&self, id: &str) -> Result<bool> {
        let resp = self
            .request(Method::DELETE, &format!("/share/{}", id))
            .await?;
        let ack = resp.json::<proto::Ack<String>>().await?;
        if ack.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn update_share(
        &self,
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
        let resp = self
            .request_json(Method::POST, &format!("/share/{}", id), req)
            .await?;
        let ack = resp.json::<proto::Ack<String>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_share_info(&self, id: &str, owner_extended: bool) -> Result<ShareInfo> {
        let resp = self
            .request(
                Method::GET,
                &format!("/share/info/{}?owner_extended={}", id, owner_extended),
            )
            .await?;
        let ack = resp.json::<proto::Ack<ShareInfo>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }
}
