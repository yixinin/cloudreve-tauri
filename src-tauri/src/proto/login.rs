use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct PrepareAck {
    pub webauthn_enabled: bool,
    pub password_enabled: bool,
}

#[derive(Serialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}
#[derive(Serialize)]
pub struct RefreshTokenReq {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginAck {
    pub user: User,
    pub token: Token,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub nickname: String,
    pub status: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub group: Group,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub permission: String,
    #[serde(rename = "direct_link_batch_size")]
    pub direct_link_batch_size: i32,
    #[serde(rename = "trash_retention")]
    pub trash_retention: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    #[serde(rename = "access_expires")]
    pub access_expires: DateTime<Utc>,
    #[serde(rename = "refresh_expires")]
    pub refresh_expires: DateTime<Utc>,
}
