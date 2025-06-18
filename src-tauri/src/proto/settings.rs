use reqwest::Version;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkMode {
    Auto = 1,
    Normal = 2,
    IPv6 = 3,
    P2P = 4,
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
}
