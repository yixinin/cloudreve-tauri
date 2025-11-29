use crate::hc::http_client_manager::HttpClientWrapper;
use crate::hc::Request;
use http::Method;

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
        let req = Request::new(Method::PUT, &self.base_url, "/share").with_body(req);
        let response = self
            .get_client()
            .await?
            .put::<_, proto::Ack<String>>(req)
            .await?;
        let ack = response.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_shares(&self, order_direction: &str) -> Result<GetSharesAck> {
        let req = Request::new(
            Method::GET,
            &self.base_url,
            &format!("/share?page_size=50&order_direction={}", order_direction),
        )
        .with_body(());
        let resp = self
            .get_client()
            .await?
            .get::<proto::Ack<GetSharesAck>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn delete_share(&self, id: &str) -> Result<bool> {
        let req =
            Request::new(Method::DELETE, &self.base_url, &format!("/share/{}", id)).with_body(());
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
        let req =
            Request::new(Method::POST, &self.base_url, &format!("/share/{}", id)).with_body(req);
        let resp = self
            .get_client()
            .await?
            .post::<_, proto::Ack<String>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }

    pub async fn get_share_info(&self, id: &str, owner_extended: bool) -> Result<ShareInfo> {
        let req = Request::new(
            Method::GET,
            &self.base_url,
            &format!("/share/info/{}?owner_extended={}", id, owner_extended),
        )
        .with_body(());
        let resp = self
            .get_client()
            .await?
            .get::<proto::Ack<ShareInfo>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }
    }
}
