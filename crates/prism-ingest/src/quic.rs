use std::net::SocketAddr;
use prism_common::RawEvent;
use crate::dispatcher::DispatcherSender;

pub struct QuicListener {
    // Stub for quinn integration
    pub bind_addr: SocketAddr,
}

impl QuicListener {
    pub async fn new(bind_addr: SocketAddr, _sender: DispatcherSender) -> std::io::Result<Self> {
        Ok(Self { bind_addr })
    }

    pub async fn run(&self) -> std::io::Result<()> {
        // Implementation pending quinn certificate setup
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}
