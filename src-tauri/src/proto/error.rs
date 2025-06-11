use reqwest::header::InvalidHeaderValue;
use serde::Serialize;
use std::fmt::{Debug, Display};

use std::error::Error as StdError;
use std::result::Result as StdResult;

pub trait ErrorDisplay: Display + Debug {}
impl<T: Display + Debug> ErrorDisplay for T {}

#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    NoData,
    HttpStatus(u16),
    Io(std::io::Error),
    Anyhow(anyhow::Error),
    Requesst(reqwest::Error),
    Serde(anyhow::Error),
    Message(i64, String),
    Other(Box<dyn ErrorDisplay + Send + Sync + 'static>),
}

// // 实现从 anyhow::Error 转换
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Anyhow(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Requesst(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serde(err.into())
    }
}
impl From<serde_urlencoded::ser::Error> for AppError {
    fn from(err: serde_urlencoded::ser::Error) -> Self {
        AppError::Serde(err.into())
    }
}

impl From<serde_urlencoded::de::Error> for AppError {
    fn from(err: serde_urlencoded::de::Error) -> Self {
        AppError::Serde(err.into())
    }
}

impl From<InvalidHeaderValue> for AppError {
    fn from(err: InvalidHeaderValue) -> Self {
        AppError::Serde(err.into())
    }
}

// impl<T> From<T> for AppError
// where
//     T: ErrorDisplay + Send + Sync + 'static,
// {
//     fn from(err: T) -> Self {
//         AppError::Other(Box::new(err))
//     }
// }

// 实现 Display 和 Error
impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Unauthorized => write!(f, "unauthorized"),
            AppError::NoData => write!(f, "success, no data"),
            AppError::HttpStatus(err) => write!(f, "http status: {}", err),
            AppError::Io(err) => write!(f, "io: {}", err),
            AppError::Anyhow(err) => write!(f, "{}", err),
            AppError::Requesst(err) => write!(f, "request error: {}", err),
            AppError::Serde(err) => write!(f, "serde error: {}", err),
            AppError::Message(code, msg) => write!(f, "code:{}, msg:{}", code, msg),
            AppError::Other(err) => write!(f, "{}", err),
        }
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::Anyhow(err) => Some(err.as_ref()),
            AppError::Serde(err) => Some(err.as_ref()),
            AppError::Other(_) => None,
            _ => None,
        }
    }
}

pub type Result<T> = core::result::Result<T, AppError>;

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<i64>,
}

impl From<&AppError> for ErrorPayload {
    fn from(error: &AppError) -> Self {
        match error {
            AppError::Unauthorized => ErrorPayload {
                msg: format!("need login"),
                code: Some(401),
            },
            AppError::NoData => ErrorPayload {
                msg: format!("no data"),
                code: Some(200),
            },
            AppError::HttpStatus(e) => ErrorPayload {
                msg: format!("http status"),
                code: Some(e.to_owned() as i64),
            },
            AppError::Io(e) => ErrorPayload {
                msg: format!("io error: {}", e),
                code: Some(400),
            },
            AppError::Requesst(e) => ErrorPayload {
                msg: format!("request error: {}", e),
                code: Some(400),
            },
            AppError::Serde(e) => ErrorPayload {
                msg: format!("json error: {}", e),
                code: Some(400),
            },
            AppError::Anyhow(e) => ErrorPayload {
                msg: format!("error: {}", e),
                code: Some(500),
            },
            AppError::Other(e) => ErrorPayload {
                msg: format!("error: {}", e),
                code: Some(500),
            },
            AppError::Message(code, msg) => ErrorPayload {
                msg: msg.to_string(),
                code: Some(code.to_owned()),
            },
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct("AppError", &ErrorPayload::from(self))
    }
}

pub type JsonResult<T: Serialize> = core::result::Result<T, AppError>;
