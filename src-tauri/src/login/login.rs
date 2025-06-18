use std::fmt::format;

use reqwest::{Client, Method, StatusCode, Version};

use crate::{
    hc,
    proto::{
        self,
        login::{LoginAck, LoginReq, PrepareAck, RefreshTokenReq, Token},
        AppError,
    },
};

use crate::hc::Site;

use crate::proto::Result;

pub async fn prepare(addr: &str, email: &str, version: Version) -> Result<PrepareAck> {
    let site = Site {
        token: "".to_string(),
        addr: addr.to_string(),
        version: version,
    };
    let prepare_url = format!("{}?email={}", "/session/prepare", email);

    // 这里替换为你的实际API地址
    let resp = site.build(Method::GET, &prepare_url).await?.send().await?;
    let ack = resp.json::<proto::Ack<PrepareAck>>().await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    }

    Err(AppError::Message(ack.code, ack.msg))
}

pub async fn login(addr: &str, email: &str, pass: &str, version: Version) -> Result<LoginAck> {
    let site = Site {
        token: "".to_string(),
        addr: addr.to_string(),
        version: version,
    };

    let request = LoginReq {
        email: email.to_string(),
        password: pass.to_string(),
    };

    match site.build(Method::POST, "/session/token").await {
        Ok(builder) => match builder.json(&request).send().await {
            Ok(resp) => {
                let ack = resp.json::<proto::Ack<LoginAck>>().await?;
                if ack.code == 0 {
                    return Ok(ack.data.unwrap());
                }
                return Err(AppError::Message(ack.code, ack.msg));
            }
            Err(e) => {
                return Err(AppError::Anyhow(anyhow::format_err!("send error: {}", e)));
            }
        },
        Err(e) => {
            return Err(AppError::Anyhow(anyhow::format_err!("build error: {}", e)));
        }
    }
}

pub async fn refresh_token(site: Site) -> Result<Token> {
    let request = RefreshTokenReq {
        refresh_token: site.token.to_string(),
    };

    // 这里替换为你的实际API地址
    let resp = site
        .build(Method::POST, "/session/token/refresh")
        .await?
        .json(&request)
        .send()
        .await?;
    let ack = resp.json::<proto::Ack<Token>>().await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    }

    Err(AppError::Message(ack.code, ack.msg))
}
