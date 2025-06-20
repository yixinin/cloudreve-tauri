use std::time::Duration;

use super::{pool::ClientPool, url};
use crate::{proto::login, rendezvouser, storage::file};
use anyhow::Result;
use reqwest::Request;

pub struct Client {
    client: ClientPool,
    token: login::Token,
    addr: String,
    addr6: String,
}

impl Client {
    pub fn new(addr: &str, addr6: &str, signal_url: &str, token: login::Token) -> Self {
        let client = ClientPool::new(signal_url, 10, Duration::from_secs(300));
        Client {
            client,
            token,
            addr: addr.to_string(),
            addr6: addr6.to_string(),
        }
    }

    pub async fn get_files(
        &self,
        path: &str,
        category: &str,
        page: u64,
        page_size: u64,
        order_by: &str,
        order: &str,
    ) -> Result<()> {
        let client = self.client.get_client().await;
        let req = file::GetFilesReq {
            path: path.to_string(),
            category: category.to_string(),
            page,
            page_size,
            order_by: order_by.to_string(),
            order: order.to_string(),
        };
        let ack = client
            .get("/file", req)
            .send()
            .await?
            .json::<proto::Ack<GetFilesAck>>()
            .await?;
        if ack.code == 0 {
            return Ok(ack.data.unwrap());
        } else {
            return Err(proto::AppError::Message(ack.code, ack.msg));
        }

        Ok(())
    }
}
