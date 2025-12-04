use crate::hc::http_client_manager::HttpClientWrapper;
use crate::hc::Request;
use http::{Method, Version};

use crate::proto::{
    login::{LoginAck, LoginReq, PrepareAck, RefreshTokenReq, Token, User},
    settings::NetworkMode,
    storage::GetCapacityAck,
    Ack, AppError,
};

use crate::proto::Result;

impl super::AppState {
    pub async fn prepare(&self, email: &str) -> Result<PrepareAck> {
        let req = self.request_with_query(
            Method::GET,
            "/session/prepare",
            std::collections::HashMap::from([("email", email.to_string())]),
        )?;
        let resp = self.get_client().await?.get::<Ack<PrepareAck>>(req).await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        }

        Err(AppError::Message(ack.code, ack.msg))
    }
    pub async fn login(&self, email: &str, pass: &str) -> Result<User> {
        let request = LoginReq {
            email: email.to_string(),
            password: pass.to_string(),
        };
        let req = self.request_with_body(Method::POST, "/session/token", request)?;
        let resp = self
            .get_client()
            .await?
            .post::<_, Ack<LoginAck>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            if let Some(data) = ack.data {
                if let Err(e) = self.set_token(data.token) {
                    println!("set token error:{}", e);
                }
                return Ok(data.user);
            }
        }
        return Err(AppError::Message(ack.code, ack.msg));
    }

    pub async fn logout(&self) -> Result<()> {
        self.delete_settings(vec![
            "access_token",
            "access_token_ttl",
            "refresh_token",
            "refresh_token_ttl",
        ])?;
        Ok(())
    }
    pub async fn refresh_token(&self) -> Result<Token> {
        let token = self.get_token()?;
        let request = RefreshTokenReq {
            refresh_token: token.refresh_token,
        };

        let req = self.request_with_body(Method::POST, "/session/token/refresh", request)?;
        let resp = self.get_client().await?.post::<_, Ack<Token>>(req).await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        }

        Err(AppError::Message(ack.code, ack.msg))
    }

    pub async fn get_capacity(&self) -> Result<GetCapacityAck> {
        let req = self.request(Method::GET, "/user/capacity")?;
        let resp = self
            .get_client()
            .await?
            .get::<Ack<GetCapacityAck>>(req)
            .await?;
        let ack = resp.into_data();
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(AppError::Message(ack.code, ack.msg));
        }
    }
}
