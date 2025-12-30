use http::Method;

use crate::proto::{
    self,
    share::{GetSharesAck, ShareInfo},
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
        let req = self.request_with_body(Method::PUT, "/share", req)?;
        let resp = req.send().await?.json::<proto::Ack<String>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn get_shares(&self, order_direction: &str) -> Result<GetSharesAck> {
        let req = self.request_with_query(
            Method::GET,
            "/share",
            [("page_size", "50"), ("order_direction", order_direction)]
                .iter()
                .cloned()
                .collect::<std::collections::HashMap<_, _>>(),
        )?;
        let resp = req.send().await?.json::<proto::Ack<GetSharesAck>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn delete_share(&self, id: &str) -> Result<bool> {
        let req = self.request(Method::DELETE, &format!("/share/{}", id))?;
        let resp = req.send().await?.json::<proto::Ack<String>>().await?;
        if resp.code == 0 {
            return Ok(true);
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
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
        let req = self.request_with_body(Method::POST, &format!("/share/{}", id), req)?;
        let resp = req.send().await?.json::<proto::Ack<String>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }

    pub async fn get_share_info(&self, id: &str, owner_extended: bool) -> Result<ShareInfo> {
        let req = self.request_with_query(
            Method::GET,
            &format!("/share/info/{}", id),
            [("owner_extended", owner_extended.to_string())]
                .iter()
                .cloned()
                .collect::<std::collections::HashMap<_, _>>(),
        )?;
        let resp = req.send().await?.json::<proto::Ack<ShareInfo>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(proto::AppError::Message(resp.code, resp.msg));
        }
    }
}
