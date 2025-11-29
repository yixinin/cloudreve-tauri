use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    hc::{
        self,
        http_client_manager::{HttpClientDispatcher, HttpClientManager},
        HttpClient,
    },
    proto::{
        login::Token,
        settings::{NetworkMode, NetworkSettings},
    },
};
use anyhow::Result;
use iroh::{Endpoint, SecretKey};
use sled::Db;

pub struct AppState {
    pub db: Arc<Db>,
    pub hcm: HttpClientManager,
    pub ct: hc::http_client_manager::ClientType,
    pub base_url: String,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let db = sled::open("app.db")?;
        Ok(Self {
            db: Arc::new(db),
            hcm: HttpClientManager::new(),
            ct: hc::http_client_manager::ClientType::Iroh,
            base_url: String::new(),
        })
    }

    pub async fn get_client(&self) -> Result<HttpClientDispatcher> {
        self.hcm.get_client(self.ct).await
    }

    pub async fn init_base_url(&mut self, base_url: String) -> Result<()> {
        self.base_url = base_url;
        self.hcm.init_reqwest_client().await?;
        self.ct = hc::http_client_manager::ClientType::Reqwest;
        Ok(())
    }

    pub async fn init_iroh_endpoint(&mut self, token: &str) -> Result<()> {
        let token = "endpointaaw67fgj5qswjmgrj26s7mvou7ejojq5ips3iubm4ebyqgjouxyq2ayaf5uhi5dqom5c6l3bobztcljrfzzgk3dbpexg4mbonfzg62bnmnqw4ylspexgs4tpnaxgy2lonmxc6aiavqlaabgkyybqcajaaeg3qaabaaaaaaaaaaaaaaaezpdag";
        let secret_key = get_or_create_secret();
        if let Ok(ticket) = dumbpipe::EndpointTicket::from_str(token) {
            let addr = ticket.endpoint_addr();
            let builder = Endpoint::builder().secret_key(secret_key);
            let endpoint = builder.bind().await?;
            self.hcm.init_iroh_endpoint(endpoint, addr.clone()).await?;
            self.ct = hc::http_client_manager::ClientType::Iroh;
            return Ok(());
        }
        Err(anyhow::format_err!("invalid token"))
    }

    pub fn gen_id(&self) -> Result<u32> {
        let db = self.db.clone();
        let mut id = 1;
        if let Some(val) = db.get("id")? {
            let id_str = String::from_utf8(val.to_vec())?;
            id = id_str.parse().unwrap_or(1);
        }
        db.insert("id", (id + 1).to_string().as_bytes())?;
        Ok(id)
    }

    pub fn get_token(&self) -> Result<Token> {
        let mut settings = self
            .get_settings(vec![
                "access_token",
                "access_expires",
                "refresh_token",
                "refresh_expires",
            ])
            .map_err(|e| anyhow::format_err!("get settings error: {}", e))?;
        if settings.len() != 4 {
            return Err(anyhow::format_err!("not found"));
        }
        let token = Token {
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
        };
        Ok(token)
    }
    pub fn set_token(&self, token: Token) -> Result<()> {
        let db = self.db.clone();
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
    }
    pub fn get_addr(&self) -> Result<NetworkSettings> {
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

        Ok(NetworkSettings {
            addr: addr,
            addr6: addr6,
            mode: mode.parse().unwrap_or(NetworkMode::Auto),
        })
    }
    pub fn set_addr(
        &self,
        addr: Option<String>,
        addr6: Option<String>,
        mode: NetworkMode,
    ) -> Result<()> {
        let db = self.db.clone();
        if let Some(value) = addr {
            db.insert("addr", value.as_bytes())?;
        }
        if let Some(value) = addr6 {
            db.insert("addr6", value.as_bytes())?;
        }

        db.insert("mode", mode.to_string().as_bytes())?;

        Ok(())
    }
    pub fn get_settings(&self, keys: Vec<&str>) -> Result<HashMap<String, String>> {
        let db = self.db.clone();
        let mut m = HashMap::with_capacity(keys.len());
        for key in keys {
            if let Some(val) = db.get(key)? {
                m.insert(key.to_string(), String::from_utf8(val.to_vec())?);
            }
        }
        Ok(m)
    }
    pub fn delete_settings(&self, keys: Vec<&str>) -> Result<()> {
        let db = self.db.clone();
        for key in keys {
            db.remove(key)?;
        }
        Ok(())
    }
}

fn get_or_create_secret() -> SecretKey {
    let key = SecretKey::generate(&mut rand::rng());
    let key_str = hex::encode(key.to_bytes());
    eprintln!("using secret key {key_str}");
    key
}
