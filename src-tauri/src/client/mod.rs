pub mod file;

use http::Version;

use crate::hc::Site;

pub struct Client {
    site: Site,
    hc_pool: crate::hc::pool::HttpClientPool,
}

impl Client {
    pub fn new(site: Site, hc_pool: crate::hc::pool::HttpClientPool) -> Self {
        Self { site, hc_pool }
    }

    pub async fn get_client(&self) -> reqwest::Client {
        let client = self.hc_pool.get_client(Version::HTTP_3).await;
        let client = client.client().to_owned();
        client
    }
}
