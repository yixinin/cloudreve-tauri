use crate::hc;
use anyhow::Result;

pub struct AppState {
    db: redb::Database,
    hc_pool: hc::pool::ClientPool,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let db = redb::Database::builder().open("app.db")?;
        let hc_pool = hc::pool::ClientPool::new(20);
        Ok(Self { db, hc_pool })
    }
}
