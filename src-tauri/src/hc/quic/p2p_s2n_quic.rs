use crate::rendezvouser;

use http::Uri;
use s2n_quic::provider::tls;
use std::error::Error as StdError;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;

use reqwest::{H3Connecting, H3Connection, H3Connector};

/// H3 Client Config
#[derive(Clone)]
pub(crate) struct H3ClientConfig {
    /// Set the maximum HTTP/3 header size this client is willing to accept.
    ///
    /// See [header size constraints] section of the specification for details.
    ///
    /// [header size constraints]: https://www.rfc-editor.org/rfc/rfc9114.html#name-header-size-constraints
    ///
    /// Please see docs in [`Builder`] in [`h3`].
    ///
    /// [`Builder`]: https://docs.rs/h3/latest/h3/client/struct.Builder.html#method.max_field_section_size
    pub(crate) max_field_section_size: Option<u64>,

    /// Enable whether to send HTTP/3 protocol grease on the connections.
    ///
    /// Just like in HTTP/2, HTTP/3 also uses the concept of "grease"
    ///
    /// to prevent potential interoperability issues in the future.
    /// In HTTP/3, the concept of grease is used to ensure that the protocol can evolve
    /// and accommodate future changes without breaking existing implementations.
    ///
    /// Please see docs in [`Builder`] in [`h3`].
    ///
    /// [`Builder`]: https://docs.rs/h3/latest/h3/client/struct.Builder.html#method.send_grease
    pub(crate) send_grease: Option<bool>,
}

impl Default for H3ClientConfig {
    fn default() -> Self {
        Self {
            max_field_section_size: None,
            send_grease: None,
        }
    }
}

#[derive(Clone)]
pub(crate) struct H3S2nQuicConnector {
    client_config: H3ClientConfig,
    tls_config: rustls::ClientConfig,
    signal_cli: rendezvouser::SignalClient,
}

impl H3S2nQuicConnector {
    pub fn new(
        signal_url: &str,
        stun_addrs: Option<Vec<String>>,
        tls: rustls::ClientConfig,
        client_config: H3ClientConfig,
    ) -> Result<H3S2nQuicConnector, Box<dyn StdError + Send + Sync>> {
        let signal_cli = rendezvouser::SignalClient::new(signal_url, stun_addrs);
        Ok(Self {
            signal_cli,
            tls_config: tls,
            client_config,
        })
    }

    pub async fn connect_dest(
        &mut self,
        dest: Uri,
    ) -> Result<H3Connection, Box<dyn StdError + Send + Sync>> {
        let (local_addr, _, remote_addr) = self.signal_cli.get_remote_addr().await?;

        let client = s2n_quic::Client::builder()
            .with_io(local_addr)?
            .with_tls(self.tls_config)?
            .start()?;
        self.remote_connect(client, remote_addr).await
    }

    async fn remote_connect(
        &mut self,
        client: s2n_quic::Client,
        remote_addr: SocketAddr,
    ) -> Result<H3Connection, Box<dyn StdError + Send + Sync>> {
        let mut err = None;
        let connect = s2n_quic::client::Connect::new(remote_addr)
            .with_server_name(remote_addr.ip().to_string());

        match client.connect(connect).await {
            Ok(new_conn) => {
                let mut h3_client_builder = h3::client::builder();
                if let Some(max_field_section_size) = self.client_config.max_field_section_size {
                    h3_client_builder.max_field_section_size(max_field_section_size);
                }
                if let Some(send_grease) = self.client_config.send_grease {
                    h3_client_builder.send_grease(send_grease);
                }
                let conn = super::s2n_quic_h3::Connection::new(new_conn);
                return Ok(h3_client_builder.build(conn).await?);
            }
            Err(e) => err = Some(e),
        }
        match err {
            Some(e) => Err(Box::new(e) as Box<dyn StdError + Send + Sync>),
            None => Err("failed to establish connection for HTTP/3 request".into()),
        }
    }
}

impl H3Connector for H3S2nQuicConnector {
    fn connect(&self, dest: Uri) -> H3Connecting {
        let mut connector = self.clone();
        Box::pin(async move {
            let connection = connector.connect_dest(dest).await?;
            Ok(connection)
        })
    }
}

impl std::fmt::Debug for H3S2nQuicConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "quinn quic connector")
    }
}
