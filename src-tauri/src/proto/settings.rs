use std::{fmt::format, str::FromStr};

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

    pub fn get_url(&self, path: &str) -> String {
        if !self.addr6.is_empty() {
            match self.mode {
                NetworkMode::IPv6 => {
                    return format!("{}{}", self.addr6, path);
                }
                NetworkMode::Auto => {
                    if crate::net::has_ipv6_connectivity() {
                        return format!("{}{}", self.addr6, path);
                    }
                }
                _ => {
                    return format!("{}{}", self.addr, path);
                }
            }
        }

        return format!("{}{}", self.addr, path);
    }
    pub fn get_query<T>(&self, path: &str, req: T) -> String
    where
        T: Serialize,
    {
        let url = self.get_url(path);
        format!(
            "{}?{}",
            url,
            serde_urlencoded::to_string(req).unwrap_or_default()
        )
    }
}
