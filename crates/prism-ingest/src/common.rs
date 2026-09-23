use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub udp_bind_addr: SocketAddr,
    pub buffer_size: usize,
    pub channel_capacity: usize,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            udp_bind_addr: "0.0.0.0:514".parse().unwrap(),
            buffer_size: 65536,
            channel_capacity: 10_000,
        }
    }
}
