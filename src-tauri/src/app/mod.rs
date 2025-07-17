use crate::{
    hc,
    proto::{
        login::{LoginAck, LoginReq, Token},
        settings::{NetworkMode, NetworkSettings},
        Ack, AppError,
    },
};
use anyhow::Result;
use redb::TableDefinition;

const TABLE_SETTINGS: TableDefinition<&str, String> = TableDefinition::new("settings");

pub struct AppState {
    pub db: redb::Database,
    pub hc_pool: hc::pool::ClientPool,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let db = redb::Database::builder().open("app.db")?;
        let hc_pool = hc::pool::ClientPool::new(20);
        Ok(Self { db, hc_pool })
    }

    pub async fn login(&self, email: &str, pass: &str) -> crate::proto::Result<LoginAck> {
        let addr = self.get_addr()?;
        let client = match addr.mode {
            NetworkMode::P2P => self.hc_pool.get_h3(&addr.addr),
            _ => self.hc_pool.get(),
        }?;
        let builder = client.post(addr.get_url("/session/token"));
        let request = LoginReq {
            email: email.to_string(),
            password: pass.to_string(),
        };
        match builder.json(&request).send().await {
            Ok(resp) => {
                let ack = resp.json::<Ack<LoginAck>>().await?;
                if ack.code == 0 {
                    return Ok(ack.data.unwrap());
                }
                return Err(AppError::Message(ack.code, ack.msg));
            }
            Err(e) => {
                return Err(AppError::Anyhow(anyhow::format_err!("send error: {}", e)));
            }
        }
    }

    pub fn refresh_token() -> Result<Token> {}
    pub fn get_token() -> Result<Token> {}
    pub fn get_settings() {}
    pub fn get_addr(&self) -> Result<NetworkSettings> {
        let rd = self.db.begin_read()?;
        let settings = rd.open_table(TABLE_SETTINGS)?;
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

    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        let client = self.hc_pool.get().unwrap();
        let bulder = client.post(url);

        bulder
    }
}
