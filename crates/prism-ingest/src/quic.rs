use std::net::SocketAddr;
use crate::dispatcher::DispatcherSender;

// QUIC dual-ingestion endpoint
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
        // Fully qualified QUIC setup requires certificates (ALPN, TLS 1.3).
        // This will be implemented when the Vault/Control Plane provides the certs.
        // For now, we block on a future that never resolves to keep the task alive.
        std::future::pending::<()>().await;
        Ok(())
    }
}
