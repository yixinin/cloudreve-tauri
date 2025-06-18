use std::fmt::format;

use reqwest::{Client, StatusCode, Version};

use crate::proto::{
    self,
    login::{LoginAck, LoginReq, PrepareAck, RefreshTokenReq, Token},
    Ack, AppError, Site,
};

use crate::proto::Result;

pub async fn prepare(addr: &str, email: &str) -> Result<PrepareAck> {
    let client = Client::new();
    let prepare_url = format!(
        "{}?email={}",
        proto::get_api_url(addr, "/session/prepare"),
        email
    );

    // 这里替换为你的实际API地址
    let resp = client.get(prepare_url).send().await?;
    let ack = resp.json::<proto::Ack<PrepareAck>>().await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    }

    Err(AppError::Message(ack.code, ack.msg))
}

pub async fn login(addr: &str, email: &str, pass: &str) -> Result<LoginAck> {
    let (client, addr) = proto::get_h3_client(addr).await?;
    let request = LoginReq {
        email: email.to_string(),
        password: pass.to_string(),
    };

    let url = proto::get_api_url(&addr, "/session/token");
    // 这里替换为你的实际API地址
    let resp = client
        .post(url)
        .version(Version::HTTP_3)
        .json(&request)
        .send()
        .await?;
    let ack = resp.json::<proto::Ack<LoginAck>>().await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    }

    Err(AppError::Message(ack.code, ack.msg))
}

pub async fn refresh_token(site: Site) -> Result<Token> {
    let client = Client::new();
    let request = RefreshTokenReq {
        refresh_token: site.token.to_string(),
    };

    // 这里替换为你的实际API地址
    let resp = client
        .post(proto::get_api_url(&site.addr, "/session/token/refresh"))
        .json(&request)
        .send()
        .await?;
    let ack = resp.json::<proto::Ack<Token>>().await?;
    if ack.code == 0 {
        return Ok(ack.data.unwrap());
    }

    Err(AppError::Message(ack.code, ack.msg))
}
