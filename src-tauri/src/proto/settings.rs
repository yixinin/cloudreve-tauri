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
    pub iroh_endpoint: String,
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
        // P2P模式使用实际的Iroh端点地址
        if self.mode == NetworkMode::P2P {
            // 优先使用专门的iroh_endpoint字段作为Iroh端点地址
            if !self.iroh_endpoint.is_empty() {
                return self.iroh_endpoint.clone();
            }
            // 如果iroh_endpoint为空，尝试使用addr字段
            if !self.addr.is_empty() {
                return self.addr.clone();
            }
            // 如果都为空，使用默认的P2P地址标记
            return "iroh://p2p".to_string();
        }

        // 选择合适的地址
        let selected_addr = if !self.addr6.is_empty() {
            match self.mode {
                NetworkMode::IPv6 => &self.addr6,
                NetworkMode::Auto => {
                    // 优化：使用本地网络接口检查而非外部连接
                    if crate::net::has_ipv6_connectivity() {
                        &self.addr6
                    } else {
                        &self.addr
                    }
                }
                _ => &self.addr,
            }
        } else {
            &self.addr
        };

        // 确保地址有正确的协议前缀
        if selected_addr.starts_with("http") {
            return selected_addr.clone();
        }
        return format!("https://{}", selected_addr);
    }

    pub fn get_url(&self, addr: Option<String>, path: &str) -> String {
        if let Some(addr) = addr {
            return format!("{}/api/v4{}", addr, path);
        }
        format!("{}/api/v4{}", self.get_addr(), path)
    }

    pub fn get_query<T>(&self, addr: Option<String>, path: &str, req: T) -> String
    where
        T: Serialize,
    {
        let addr = if let Some(addr) = addr {
            format!("{}", addr)
        } else {
            self.get_addr()
        };
        format!(
            "{}/api/v4/{}?{}",
            addr,
            path,
            serde_urlencoded::to_string(req).unwrap_or_default()
        )
    }
}
