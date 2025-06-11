use ::serde::{Deserialize, Serialize};

use super::storage::Pagination;

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub id: String,
    pub name: String,
    pub expired: bool,
    pub source_type: u8,
    pub unlocked: bool,
    pub url: String,
    pub visited: u64,
    pub created_at: String,
    pub owner: UserOwner,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserOwner {
    pub id: String,
    pub nickname: String,
    pub email: String,
    pub created_at: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetSharesAck {
    pub shares: Vec<Share>,
    pub pagination: Pagination,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GetSharesReq {
    pub page_size: u64,
    pub order_direction: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ShareOwner {
    pub id: String,
    pub email: String,
    pub nickname: String,
    pub created_at: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ShareInfo {
    pub id: String,
    pub name: String,
    pub visited: i32,
    pub unlocked: bool,
    pub source_type: i32,
    pub owner: Option<ShareOwner>,
    pub created_at: String,
    pub expired: bool,
    pub url: String,
    pub source_uri: String,
}
