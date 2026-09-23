use std::net::SocketAddr;
use std::sync::Arc;
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use crate::dispatcher::DispatcherSender;

pub struct QuicListener {
    pub bind_addr: SocketAddr,
    #[allow(dead_code)]
    sender: DispatcherSender,
}

impl QuicListener {
    pub async fn new(bind_addr: SocketAddr, sender: DispatcherSender) -> std::io::Result<Self> {
        Ok(Self { bind_addr, sender })
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let cert_der = cert.cert.der().to_vec();
        let key_der = cert.signing_key.serialize_der();
        
        let mut server_crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![CertificateDer::from(cert_der)],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der)),
            )
            .unwrap();
        
        server_crypto.alpn_protocols = vec![b"prism-ingest".to_vec()];
        let quic_server_config = quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto).unwrap();
        let server_config = ServerConfig::with_crypto(Arc::new(quic_server_config));

        let endpoint = Endpoint::server(server_config, self.bind_addr)?;
        eprintln!("QUIC listening on {}", endpoint.local_addr()?);

        // Keep alive for connections (stub for Phase 1)
        while let Some(conn) = endpoint.accept().await {
            // Placeholder: simply ignore connections for now as full QUIC stream
            // parsing is planned for Phase 3 when certificates are fully integrated.
            tokio::spawn(async move {
                if let Ok(_connection) = conn.await {
                    eprintln!("QUIC connection accepted");
                }
            });
        }
        
        Ok(())
    }
}
