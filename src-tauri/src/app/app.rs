use std::collections::HashMap;

use crate::{
    hc,
    proto::{
        login::Token,
        settings::{NetworkMode, NetworkSettings},
    },
};
use anyhow::Result;
use redb::{ReadableTable, TableDefinition};

const TABLE_SETTING: TableDefinition<&str, String> = TableDefinition::new("setting");

pub struct AppState {
    pub db: redb::Database,
    pub hc_pool: hc::pool::ClientPool,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let db = redb::Database::builder().create("app.db")?;
        // {
        //     let db = db.begin_write()?;
        //     let mut id = 1;
        //     {
        //         let setting = db.open_table(TABLE_SETTING)?;
        //         let val = setting.get("id")?;

        //         if let Some(value) = val {
        //             let id_str = value.value();
        //             id = id_str.parse().unwrap_or(1);
        //         }
        //     }
        //     {
        //         let mut setting = db.open_table(TABLE_SETTING)?;
        //         setting.insert("id", (id + 1).to_string())?;
        //     }

        //     db.commit()?;
        //     println!("current id: {}", id);
        // }
        let hc_pool = hc::pool::ClientPool::new(20);
        Ok(Self { db, hc_pool })
    }

    pub fn reset_pool(&mut self) {
        self.hc_pool = hc::pool::ClientPool::new(20);
    }

    pub fn gen_id(&self) -> Result<u32> {
        let db = self.db.begin_write()?;
        let mut id = 1;
        {
            let setting = db.open_table(TABLE_SETTING)?;
            let val = setting.get("id")?;

            if let Some(value) = val {
                let id_str = value.value();
                id = id_str.parse().unwrap_or(1);
            }
        }
        {
            let mut setting = db.open_table(TABLE_SETTING)?;
            setting.insert("id", (id + 1).to_string())?;
        }

        db.commit()?;
        Ok(id)
    }

    pub fn get_token(&self) -> Result<Token> {
        let mut settings = self.get_settings(vec![
            "access_token",
            "access_token_ttl",
            "refresh_token",
            "refresh_token_ttl",
        ])?;
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
        let db = self.db.begin_write()?;
        {
            let mut setting = db.open_table(TABLE_SETTING)?;
            setting.insert("access_token", token.access_token);
            setting.insert("access_expires", token.access_expires.to_string());
            setting.insert("refresh_token", token.refresh_token);
            setting.insert("refresh_expires", token.refresh_expires.to_string());
        }

        db.commit()?;
        Ok(())
    }
    pub fn get_addr(&self) -> Result<NetworkSettings> {
        let rd = self.db.begin_read()?;
        let settings = rd.open_table(TABLE_SETTING)?;
        let addr = if let Some(val) = settings.get("addr")? {
            val.value()
        } else {
            String::new()
        };
        let addr6 = if let Some(val) = settings.get("addr6")? {
            val.value()
        } else {
            String::new()
        };
        let mode = if let Some(val) = settings.get("mode")? {
            val.value()
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
        let db = self.db.begin_write()?;
        {
            let mut setting = db.open_table(TABLE_SETTING)?;
            if let Some(value) = addr {
                setting.insert("addr", value);
            }
            if let Some(value) = addr6 {
                setting.insert("addr6", value);
            }

            setting.insert("mode", mode.to_string());
        }

        db.commit()?;
        Ok(())
    }
    pub fn get_settings(&self, keys: Vec<&str>) -> Result<HashMap<String, String>> {
        let db = self.db.begin_read()?;
        let setting = db.open_table(TABLE_SETTING)?;
        let mut m = HashMap::with_capacity(keys.len());
        for key in keys {
            if let Some(val) = setting.get(key)? {
                m.insert(key.to_string(), val.value());
            }
        }
        Ok(m)
    }
    pub fn delete_settings(&self, keys: Vec<&str>) -> Result<()> {
        let db = self.db.begin_write()?;
        {
            let mut setting = db.open_table(TABLE_SETTING)?;
            for key in keys {
                setting.remove(key);
            }
        }
        db.commit()?;
        Ok(())
    }

    pub async fn request_json<T>(
        &self,
        method: reqwest::Method,
        path: &str,
        req: T,
    ) -> Result<reqwest::Response>
    where
        T: serde::Serialize,
    {
        let network = self.get_addr()?;
        let addr = network.get_addr();
        let url = network.get_url(path);
        let client = self.hc_pool.get(&addr, network.mode == NetworkMode::P2P)?;

        let mut builder = client.request(method, url);
        if network.mode == NetworkMode::P2P {
            builder = builder.version(http::Version::HTTP_3)
        }
        let token = self.get_token()?;
        builder = builder.header("authorization", token.access_token);
        let resp = builder.json(&req).send().await?;
        self.hc_pool.put(client);
        Ok(resp)
    }
    pub async fn request_query<T>(
        &self,
        method: reqwest::Method,
        path: &str,
        req: T,
    ) -> Result<reqwest::Response>
    where
        T: serde::Serialize,
    {
        let network = self.get_addr()?;
        let addr = network.get_addr();
        let url = network.get_url(path);
        let client = self.hc_pool.get(&addr, network.mode == NetworkMode::P2P)?;

        let mut builder = client.request(method, url);
        if network.mode == NetworkMode::P2P {
            builder = builder.version(http::Version::HTTP_3)
        }
        let token = self.get_token()?;
        builder = builder.header("authorization", token.access_token);
        let resp = builder.query(&req).send().await?;
        self.hc_pool.put(client);
        Ok(resp)
    }

    pub async fn request(&self, method: reqwest::Method, path: &str) -> Result<reqwest::Response> {
        let network = self.get_addr()?;
        let addr = network.get_addr();
        let url = network.get_url(path);
        let client = self.hc_pool.get(&addr, network.mode == NetworkMode::P2P)?;

        let mut builder = client.request(method, url);
        if network.mode == NetworkMode::P2P {
            builder = builder.version(http::Version::HTTP_3)
        }
        let token = self.get_token()?;
        builder = builder.header("authorization", token.access_token);
        let resp = builder.send().await?;
        self.hc_pool.put(client);
        Ok(resp)
    }
}
