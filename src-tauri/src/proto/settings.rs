use std::{
    fmt::{self, format},
    str::FromStr,
};

use reqwest::Version;
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, PartialEq, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
pub enum NetworkMode {
    Auto = 1,
    Normal = 2,
    IPv6 = 3,
    P2P = 4,
}

impl FromStr for NetworkMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mode = match s {
            "2" => NetworkMode::Normal,
            "3" => NetworkMode::IPv6,
            "4" => NetworkMode::P2P,
            _ => NetworkMode::Auto,
        };
        Ok(mode)
    }
}

impl fmt::Display for NetworkMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = match self {
            NetworkMode::P2P => "4",
            NetworkMode::IPv6 => "3",
            NetworkMode::Normal => "2",
            _ => "1",
        };

        write!(f, "{}", mode)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub addr: String,
    pub addr6: String,
    pub mode: NetworkMode,
}

impl NetworkSettings {
    pub fn get_http_version(&self) -> Version {
        match self.mode {
            NetworkMode::P2P => Version::HTTP_3,
            NetworkMode::IPv6 => Version::HTTP_2,
            _ => Version::HTTP_11,
        }
    }

    pub fn get_addr(&self) -> String {
        let addr: String;
        if !self.addr6.is_empty() {
            addr = match self.mode {
                NetworkMode::IPv6 => self.addr6.clone(),
                NetworkMode::Auto => {
                    if crate::net::has_ipv6_connectivity() {
                        self.addr6.clone()
                    } else {
                        self.addr.clone()
                    }
                }
                _ => self.addr.clone(),
            };
        } else {
            addr = self.addr.clone();
        }
        if addr.starts_with("http") {
            return addr;
        }
        return format!("https://{}", addr);
    }

    pub fn get_url(&self, path: &str) -> String {
        format!("{}{}", self.get_addr(), path)
    }

    pub fn get_query<T>(&self, path: &str, req: T) -> String
    where
        T: Serialize,
    {
        let addr = self.get_addr();
        format!(
            "{}{}?{}",
            addr,
            path,
            serde_urlencoded::to_string(req).unwrap_or_default()
        )
    }
}
