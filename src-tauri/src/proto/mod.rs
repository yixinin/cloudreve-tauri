pub mod error;
pub mod file;
pub mod login;
pub mod settings;
pub mod share;
pub mod storage;

pub use error::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ack<T> {
    pub code: i64,
    pub data: Option<T>,
    pub msg: String,
    pub correlation_id: Option<String>,
}

impl<T> Ack<T> {
    pub fn get_data(self) -> JsonResult<T> {
        if self.code == 0 {
            if let Some(data) = self.data {
                return Ok(data);
            }
            return Err(AppError::NoData);
        }
        Err(AppError::Message(self.code, self.msg))
    }
}
