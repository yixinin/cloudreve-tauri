use crate::proto::login::Token;
use reqwest::Client;
use reqwest::Method;

pub struct FileClient {
    token: Token,
    client: Client,
}

impl FileClient {
    pub fn new(client: Client, token: Token) -> Self {
        Self { client, token }
    }

    pub async fn request(&self, method: Method, url: &str) -> Result<String, Error> {
        let response = self
            .client
            .request(method, url)
            .header("key", self.token.access_token)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            Err(Error::from(response.status()))
        }
    }
}

impl FileClient {
    pub async fn upload_file(&self, file_path: &str, remote_path: &str) -> Result<(), Error> {
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut content = Vec::new();
        reader.read_to_end(&mut content).await?;

        let response = self.client.post(remote_path).body(content).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::from(response.status()))
        }
    }
}
