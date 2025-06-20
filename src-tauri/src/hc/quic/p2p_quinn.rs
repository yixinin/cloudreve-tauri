use crate::rendezvouser;

use h3_quinn::Connection;

use http::Uri;
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{ClientConfig, Endpoint, TransportConfig};
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
pub(crate) struct H3QuinnConnector {
    client_config: H3ClientConfig,
    quic_config: ClientConfig,
    signal_cli: rendezvouser::SignalClient,
}

impl H3QuinnConnector {
    pub fn new(
        signal_url: &str,
        stun_addrs: Option<Vec<String>>,
        tls: rustls::ClientConfig,
        transport_config: TransportConfig,
        client_config: H3ClientConfig,
    ) -> Result<H3QuinnConnector, Box<dyn StdError + Send + Sync>> {
        let quic_client_config = Arc::new(QuicClientConfig::try_from(tls)?);
        let mut config = ClientConfig::new(quic_client_config);
        // FIXME: Replace this when there is a setter.
        config.transport_config(Arc::new(transport_config));

        let signal_cli = rendezvouser::SignalClient::new(signal_url, stun_addrs);

        Ok(Self {
            signal_cli,
            quic_config: config,
            client_config,
        })
    }

    pub async fn connect_dest(
        &mut self,
        _dest: Uri,
    ) -> Result<H3Connection, Box<dyn StdError + Send + Sync>> {
        let (local_addr, _, remote_addr) = self.signal_cli.get_remote_addr().await?;
        let mut endpoint = Endpoint::client(local_addr)?;
        endpoint.set_default_client_config(self.quic_config.clone());
        let _ = rendezvouser::simple_udp_hole_punching(local_addr, remote_addr).await;
        self.remote_connect(endpoint, remote_addr).await
    }

    async fn remote_connect(
        &mut self,
        endpoint: Endpoint,
        remote_addr: SocketAddr,
    ) -> Result<H3Connection, Box<dyn StdError + Send + Sync>> {
        let mut err = None;
        match endpoint
            .connect(remote_addr, &remote_addr.ip().to_string())?
            .await
        {
            Ok(new_conn) => {
                let quinn_conn = Connection::new(new_conn);
                let mut h3_client_builder = h3::client::builder();
                if let Some(max_field_section_size) = self.client_config.max_field_section_size {
                    h3_client_builder.max_field_section_size(max_field_section_size);
                }
                if let Some(send_grease) = self.client_config.send_grease {
                    h3_client_builder.send_grease(send_grease);
                }
                return Ok(h3_client_builder.build(quinn_conn).await?);
            }
            Err(e) => err = Some(e),
        }

        match err {
            Some(e) => Err(Box::new(e) as Box<dyn StdError + Send + Sync>),
            None => Err("failed to establish connection for HTTP/3 request".into()),
        }
    }
}

impl H3Connector for H3QuinnConnector {
    fn connect(&self, dest: Uri) -> H3Connecting {
        let mut connector = self.clone();
        Box::pin(async move {
            let connection = connector.connect_dest(dest).await?;
            Ok(connection)
        })
    }
}

impl std::fmt::Debug for H3QuinnConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "quinn quic connector")
    }
}
