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
        let prepare_url = format!("{}?email={}", "/session/prepare", email);
        let resp = self.request(Method::GET, &prepare_url).await?;
        let ack = resp.json::<Ack<PrepareAck>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        }

        Err(AppError::Message(ack.code, ack.msg))
    }
    pub async fn login(&self, email: &str, pass: &str) -> Result<User> {
        let network = self.get_addr()?;
        let addr = network.get_addr();

        let client = self
            .get_client(&addr, network.mode == NetworkMode::P2P)
            .await?;
        let url = network.get_url(client.get_addr(), "/session/token");
        let builder = client.post(&url).version(Version::HTTP_3);
        let request = LoginReq {
            email: email.to_string(),
            password: pass.to_string(),
        };
        println!("send request to: {}, body: {:#?}", &url, &request);
        match builder.json(&request).send().await {
            Ok(resp) => {
                self.hc_pool.put(client);
                let text = resp.text().await?;
                let ack: Ack<LoginAck> = serde_json::from_str(&text)?;
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
            Err(e) => {
                return Err(AppError::Anyhow(anyhow::format_err!("send error: {}", e)));
            }
        }
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

        let network = self.get_addr()?;
        let addr = network.get_addr();

        let client = self
            .get_client(&addr, network.mode == NetworkMode::P2P)
            .await?;
        let url = network.get_url(client.get_addr(), "/session/token/refresh");

        let resp = client.post(url).json(&request).send().await?;
        self.hc_pool.put(client);
        let ack = resp.json::<Ack<Token>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        }

        Err(AppError::Message(ack.code, ack.msg))
    }

    pub async fn get_capacity(&self) -> Result<GetCapacityAck> {
        let resp = self.request(Method::GET, "/user/capacity").await?;
        let ack = resp.json::<Ack<GetCapacityAck>>().await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(AppError::Message(ack.code, ack.msg));
        }
    }
}
