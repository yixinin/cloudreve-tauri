pub mod error;
pub mod file;
pub mod login;
pub mod settings;
pub mod share;
pub mod storage;

pub use error::*;

use reqwest::{Client, Method, Version};
use serde::{Deserialize, Serialize};

use crate::rendezvouser;
#[derive(Debug, Serialize, Deserialize)]
pub struct Ack<T> {
    pub code: i64,
    pub data: Option<T>,
    pub msg: String,
    pub correlation_id: Option<String>,
}
