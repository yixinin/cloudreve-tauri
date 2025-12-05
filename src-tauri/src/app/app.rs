use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    hc::{
        self,
        http_client_manager::{HttpClientDispatcher, HttpClientManager},
        HttpClient, Request,
    },
    proto::{
        login::Token,
        settings::{NetworkMode, NetworkSettings},
    },
};
use anyhow::Result;
use http::Method;
use iroh::{Endpoint, SecretKey};
use sled::{Db, Mode};
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

    pub fn request(&self, method: Method, url: &str) -> Result<Request<()>> {
        self.request_with_body(method, url, ())
    }
    pub fn request_with_query<T>(&self, method: Method, url: &str, query: T) -> Result<Request<()>>
    where
        T: serde::Serialize,
    {
        self.request(
            method,
            &format!("{}?{}", url, serde_urlencoded::to_string(query)?),
        )
    }

    pub fn request_with_body<T>(&self, method: Method, url: &str, body: T) -> Result<Request<T>>
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
        }
        Ok(req.with_body(body))
    }

    pub async fn get_client(&self) -> Result<HttpClientDispatcher> {
        self.hcm.get_client(self.ct).await
    }

    pub fn init_base_url(&mut self, base_url: &str) -> Result<()> {
        self.base_url = format!("{}/api/v4", base_url);
        println!("init base url: {}", &self.base_url);
        Ok(())
    }

    fn parse_subdomain(subdomain: &str) -> anyhow::Result<iroh::EndpointAddr> {
        // first try to parse as a endpoint id
        if let Ok(endpoint_id) = iroh::EndpointId::from_str(subdomain) {
            return Ok(iroh::EndpointAddr::new(endpoint_id));
        }
        // then try to parse as a endpoint ticket
        if let Ok(ticket) = dumbpipe::EndpointTicket::from_str(subdomain) {
            return Ok(ticket.endpoint_addr().clone());
        }
        Err(anyhow::anyhow!("invalid subdomain"))
    }

    pub async fn init_iroh_endpoint(&mut self) -> Result<()> {
        // 检查是否配置了代理
        let settings = self.get_network_settings()?;
        if settings.proxy_url.is_some() {
            // Iroh 不直接支持代理，使用 Reqwest 客户端替代
            println!("Proxy configured, using Reqwest client instead of Iroh");
            self.hcm.init_reqwest_client_with_proxy(
                settings.proxy_url.as_deref(),
                settings.proxy_username.as_deref(),
                settings.proxy_password.as_deref(),
            )?;
            self.ct = hc::http_client_manager::ClientType::Reqwest;
            return Ok(());
        }

        // 没有代理配置，继续使用 Iroh
        let token = "endpointaaw67fgj5qswjmgrj26s7mvou7ejojq5ips3iubm4ebyqgjouxyq2ayaf5uhi5dqom5c6l3bobztcljrfzzgk3dbpexg4mbonfzg62bnmnqw4ylspexgs4tpnaxgy2lonmxc6aiavqlaabgkyybqcajaaeg3qaabaaaaaaaaaaaaaaaezpdag";
        let secret_key = get_or_create_secret();
        let addr = Self::parse_subdomain(token)?;
        self.hcm.init_iroh_endpoint(addr.clone())?;
        return Ok(());
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
    pub async fn get_network_settings(&self) -> Result<NetworkSettings> {
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
        let mode = if let Some(val) = db.get("mode")? {
            String::from_utf8(val.to_vec())?
        } else {
            String::new()
        };
        let proxy_url = if let Some(val) = db.get("proxy_url")? {
            Some(String::from_utf8(val.to_vec())?)
        } else {
            None
        };
        let proxy_username = if let Some(val) = db.get("proxy_username")? {
            Some(String::from_utf8(val.to_vec())?)
        } else {
            None
        };
        let proxy_password = if let Some(val) = db.get("proxy_password")? {
            Some(String::from_utf8(val.to_vec())?)
        } else {
            None
        };

        Ok(NetworkSettings {
            addr: addr,
            addr6: addr6,
            mode: mode.parse().unwrap_or(NetworkMode::Auto),
            proxy_url: proxy_url,
            proxy_username: proxy_username,
            proxy_password: proxy_password,
        })
        .await?
    }

    pub async fn set_addr(
        &self,
        addr: Option<String>,
        addr6: Option<String>,
        mode: NetworkMode,
    ) -> Result<()> {
        let db = self.db.clone();
        let addr_clone = addr.clone();
        let addr6_clone = addr6.clone();

        // 使用spawn_blocking将阻塞的数据库操作移到单独的线程
        tokio::task::spawn_blocking(move || {
            if let Some(value) = addr_clone {
                db.insert("addr", value.as_bytes())?;
            }
            if let Some(value) = addr6_clone {
                db.insert("addr6", value.as_bytes())?;
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
