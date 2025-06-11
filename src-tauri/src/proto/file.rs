use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFileAck {
    pub owner: ApplicationOwner,
    pub path: String,
    pub token: String,
    #[serde(rename = "type")]
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationOwner {
    #[serde(rename = "type")]
    owner_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTokenReq {
    pub tokens: Vec<String>,
}
