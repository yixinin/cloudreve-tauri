use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    hc::{
        self,
        http_client_manager::{HttpClientDispatcher, HttpClientManager},
        iroh_client, Request,
    },
    proto::{
        login::Token,
        settings::{NetworkMode, NetworkSettings},
    },
};
use anyhow::Result;
use http::Method;
use sled::Db;
use tauri::Manager;

pub struct AppState {
    pub db: Arc<Db>,
    pub hcm: HttpClientManager,
    pub ct: hc::http_client_manager::ClientType,
    pub base_url: String,
}

impl AppState {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        // 获取应用数据目录，确保数据库文件存储在正确位置
        let app_data_dir = app_handle.path().app_data_dir()?;
        let db_path = app_data_dir.join("app.db");

        // 使用sled的默认配置，让sled自动处理数据库文件的创建和打开
        let db = sled::open(db_path)?;
        Ok(Self {
            db: Arc::new(db),
            hcm: HttpClientManager::new(),
            ct: hc::http_client_manager::ClientType::Reqwest,
            base_url: String::new(),
        })
    }

    pub fn request(&self, method: Method, url: &str) -> Result<Request> {
        self.request_with_body(method, url, ())
    }
    pub fn request_with_query<T>(&self, method: Method, url: &str, query: T) -> Result<Request>
    where
        T: serde::Serialize,
    {
        self.request(
            method,
            &format!("{}?{}", url, serde_urlencoded::to_string(query)?),
        )
    }

    pub fn request_with_body<T>(&self, method: Method, url: &str, body: T) -> Result<Request>
    where
        T: serde::Serialize,
    {
        let host = self
            .base_url
            .parse::<http::Uri>()?
            .host()
            .unwrap_or_default()
            .to_string();
        let mut req = Request::new(method, &self.base_url, url);
        req = req.with_header("Host", &host);
        if let Ok(tokens) = self.get_token_sync() {
            req = req.with_header("Authorization", &format!("Bearer {}", tokens.access_token));
            eprintln!("token found: {:?}", tokens.access_token);
        } else {
            eprintln!("no token found");
        }
        Ok(req.json(body)?)
    }

    pub async fn get_client(&self) -> Result<HttpClientDispatcher> {
        self.hcm.get_client(self.ct).await
    }

    pub async fn init_base_url(&mut self, settings: &NetworkSettings) -> Result<()> {
        eprintln!("init base url: {:?}", settings);
        // 决定HTTP地址：根据mode和网络连通性选择IPv6或IPv4
        let http_addr = if !settings.addr6.is_empty() {
            match settings.mode {
                NetworkMode::IPv6 => &settings.addr6,
                NetworkMode::Auto => {
                    // 使用本地网络接口检查而非外部连接
                    if crate::net::has_ipv6_connectivity() {
                        &settings.addr6
                    } else {
                        &settings.addr
                    }
                }
                NetworkMode::P2P => {
                    // P2P模式下，HTTP地址可能不被使用，但仍需要设置默认值
                    &settings.addr
                }
                _ => &settings.addr,
            }
        } else {
            &settings.addr
        };

        // 构建带有正确协议的HTTP地址
        let full_http_addr = if http_addr.starts_with("http") {
            http_addr.clone()
        } else {
            format!("https://{}", http_addr)
        };
        self.base_url = format!("{}/api/v4", full_http_addr);
        println!("init base url: {}", &self.base_url);

        // 处理P2P模式下的Iroh端点
        if settings.mode == NetworkMode::P2P {
            // 优先使用专门的iroh_endpoint字段作为Iroh端点地址
            let iroh_addr = if !settings.iroh_endpoint.is_empty() {
                settings.iroh_endpoint.clone()
            } else {
                "iroh://p2p".to_string()
            };

            // 检查是否是默认的P2P标记
            if iroh_addr != "iroh://p2p" {
                // 初始化Iroh端点 - 使用实际的端点地址（去掉iroh://前缀）
                if let Ok(addr) = iroh_client::parse_subdomain(&iroh_addr) {
                    self.hcm.init_iroh_endpoint(&iroh_addr).await?;
                    println!("Iroh endpoint initialized with address: {:?}", addr);
                } else {
                    println!("Failed to parse Iroh endpoint address: {}", iroh_addr);
                }
            } else {
                // 如果是默认的P2P标记，不初始化Iroh端点
                println!("Using default P2P address marker, skipping Iroh endpoint initialization");
            }
        }

        Ok(())
    }

    pub async fn gen_id(&self) -> Result<u32> {
        let db = self.db.clone();

        tokio::task::spawn_blocking(move || {
            let mut id = 1;
            if let Some(val) = db.get("id")? {
                let id_str = String::from_utf8(val.to_vec())?;
                id = id_str.parse().unwrap_or(1);
            }
            db.insert("id", (id + 1).to_string().as_bytes())?;
            Ok(id)
        })
        .await?
    }

    pub fn get_token_sync(&self) -> Result<Token> {
        let db = self.db.clone();

        // 使用block_in_place在当前线程上执行阻塞操作
        tokio::task::block_in_place(move || {
            let mut settings = HashMap::new();
            let keys = [
                "access_token",
                "access_expires",
                "refresh_token",
                "refresh_expires",
            ];

            for key in &keys {
                if let Some(val) = db.get(key)? {
                    settings.insert(key.to_string(), String::from_utf8(val.to_vec())?);
                }
            }

            if settings.len() < 4 {
                return Err(anyhow::format_err!("get tokens error: {:?}", settings));
            }

            Ok(Token {
                access_token: settings.remove("access_token").unwrap_or_default(),
                access_expires: settings
                    .remove("access_expires")
                    .unwrap_or_default()
                    .parse()?,
                refresh_token: settings.remove("refresh_token").unwrap_or_default(),
                refresh_expires: settings
                    .remove("refresh_expires")
                    .unwrap_or_default()
                    .parse()?,
            })
        })
    }

    pub async fn get_token(&self) -> Result<Token> {
        self.get_token_sync()
    }
    pub async fn set_token(&self, token: Token) -> Result<()> {
        eprintln!("set token: {:?}", token);
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            db.insert("access_token", token.access_token.as_bytes())?;
            db.insert(
                "access_expires",
                token.access_expires.to_string().as_bytes(),
            )?;
            db.insert("refresh_token", token.refresh_token.as_bytes())?;
            db.insert(
                "refresh_expires",
                token.refresh_expires.to_string().as_bytes(),
            )?;
            Ok(())
        })
        .await?
    }
    pub fn get_network_settings(&self) -> Result<NetworkSettings> {
        let db = self.db.clone();
        let addr = if let Some(val) = db.get("addr")? {
            String::from_utf8(val.to_vec())?
        } else {
            String::new()
        };
        let addr6 = if let Some(val) = db.get("addr6")? {
            String::from_utf8(val.to_vec())?
        } else {
            String::new()
        };
        let iroh_endpoint = if let Some(val) = db.get("iroh_endpoint")? {
            String::from_utf8(val.to_vec())?
        } else {
            String::new()
        };
        let mode = if let Some(val) = db.get("mode")? {
            String::from_utf8(val.to_vec())?
        } else {
            String::new()
        };

        Ok(NetworkSettings {
            addr: addr,
            addr6: addr6,
            iroh_endpoint: iroh_endpoint,
            mode: mode.parse().unwrap_or(NetworkMode::Auto),
        })
    }

    pub async fn set_addr(
        &self,
        addr: Option<String>,
        addr6: Option<String>,
        iroh_endpoint: Option<String>,
        mode: NetworkMode,
    ) -> Result<()> {
        let db = self.db.clone();
        let addr_clone = addr.clone();
        let addr6_clone = addr6.clone();
        let iroh_endpoint_clone = iroh_endpoint.clone();

        // 使用spawn_blocking将阻塞的数据库操作移到单独的线程
        tokio::task::spawn_blocking(move || {
            if let Some(value) = addr_clone {
                db.insert("addr", value.as_bytes())?;
            }
            if let Some(value) = addr6_clone {
                db.insert("addr6", value.as_bytes())?;
            }
            if let Some(value) = iroh_endpoint_clone {
                db.insert("iroh_endpoint", value.as_bytes())?;
            }

            db.insert("mode", mode.to_string().as_bytes())?;

            Ok(())
        })
        .await?
    }
    pub async fn get_settings(&self, keys: Vec<&str>) -> Result<HashMap<String, String>> {
        let db = self.db.clone();
        let keys_owned: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();

        tokio::task::spawn_blocking(move || {
            let mut m = HashMap::with_capacity(keys_owned.len());
            for key in keys_owned {
                if let Some(val) = db.get(key.clone())? {
                    m.insert(key, String::from_utf8(val.to_vec())?);
                }
            }
            Ok(m)
        })
        .await?
    }
    pub async fn delete_settings(&self, keys: Vec<&str>) -> Result<()> {
        let db = self.db.clone();
        let keys_owned: Vec<String> = keys.into_iter().map(|k| k.to_string()).collect();

        tokio::task::spawn_blocking(move || {
            for key in keys_owned {
                db.remove(key)?;
            }
            Ok(())
        })
        .await?
    }
}
