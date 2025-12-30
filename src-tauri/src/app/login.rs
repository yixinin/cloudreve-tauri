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
        let resp = req.send().await?.json::<Ack<PrepareAck>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        }

        Err(AppError::Message(resp.code, resp.msg))
    }
    pub async fn login(&self, email: &str, pass: &str) -> Result<User> {
        let request = LoginReq {
            email: email.to_string(),
            password: pass.to_string(),
        };
        let req = self.request_with_body(Method::POST, "/session/token", request)?;
        let resp = req.send().await?;
        let text = resp.text().await?;
        println!("login resp:{}", text);
        let ack = serde_json::from_str::<Ack<LoginAck>>(&text)?;
        if ack.code == 0 {
            if let Some(data) = ack.data {
                if let Err(e) = self.set_token(data.token).await {
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
        ])
        .await?;
        Ok(())
    }
    pub async fn refresh_token(&self) -> Result<Token> {
        let token = self.get_token().await?;
        let request = RefreshTokenReq {
            refresh_token: token.refresh_token,
        };

        let req = self.request_with_body(Method::POST, "/session/token/refresh", request)?;
        let resp = req.send().await?.json::<Ack<Token>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        }

        Err(AppError::Message(resp.code, resp.msg))
    }

    pub async fn get_capacity(&self) -> Result<GetCapacityAck> {
        let req = self.request(Method::GET, "/user/capacity")?;
        let resp = req.send().await?.json::<Ack<GetCapacityAck>>().await?;
        if resp.code == 0 {
            return Ok(resp.data.unwrap());
        } else {
            return Err(AppError::Message(resp.code, resp.msg));
        }
    }
}
