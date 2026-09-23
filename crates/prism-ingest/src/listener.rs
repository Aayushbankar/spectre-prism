use std::sync::Arc;
use tokio::net::UdpSocket;
use chrono::Utc;
use prism_common::{LogSource, RawEvent, ProvenanceMeta};
use crate::common::IngestConfig;
use crate::dispatcher::DispatcherSender;
use bytes::BytesMut;

pub struct UdpListener {
    socket: Arc<UdpSocket>,
    sender: DispatcherSender,
    config: IngestConfig,
}

impl UdpListener {
    pub async fn new(config: IngestConfig, sender: DispatcherSender) -> std::io::Result<Self> {
        let domain = if config.udp_bind_addr.is_ipv4() {
            socket2::Domain::IPV4
        } else {
            socket2::Domain::IPV6
        };
        let socket2_sock = socket2::Socket::new(domain, socket2::Type::DGRAM, Some(socket2::Protocol::UDP))?;
        
        let _ = socket2_sock.set_recv_buffer_size(8 * 1024 * 1024);
        socket2_sock.bind(&config.udp_bind_addr.into())?;
        socket2_sock.set_nonblocking(true)?;

        let std_socket: std::net::UdpSocket = socket2_sock.into();
        let socket = tokio::net::UdpSocket::from_std(std_socket)?;

        Ok(Self {
            socket: Arc::new(socket),
            sender,
            config,
        })
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.socket.local_addr()
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let mut drop_count = 0u64;
        
        // We use a single large block of BytesMut.
        // We set the chunk size to a large value (e.g., 10MB) to amortize allocations.
        // If config.buffer_size is small, we override it for the chunk allocation.
        let chunk_size = self.config.buffer_size.max(10 * 1024 * 1024);
        let mut buf = BytesMut::with_capacity(chunk_size);
        
        loop {
            // Only reserve when we don't have enough space for a max UDP packet.
            if buf.capacity() < 65536 {
                buf.reserve(chunk_size);
            }
            
            match self.socket.recv_buf_from(&mut buf).await {
                Ok((len, addr)) => {
                    if len == 0 {
                        continue;
                    }

                    // In-flight BLAKE3 SIMD Hash
                    let hash = blake3::hash(&buf[buf.len() - len..]);

                    // Zero-copy chunking
                    let data = buf.split_to(len).freeze();
                    
                    let timestamp = Utc::now();
                    let source = LogSource::Udp(addr);

                    let payload = RawEvent {
                        payload: data,
                        source: source.clone(),
                        timestamp,
                        metadata: ProvenanceMeta {
                            hash,
                            timestamp,
                            source,
                        },
                    };
                    
                    // Backpressure handle: try_broadcast so we don't block the hot loop
                    if let Err(_) = self.sender.try_broadcast(payload) {
                        drop_count += 1;
                        if drop_count % 1000 == 0 {
                            eprintln!("UdpListener: dropped {} packets due to channel backpressure", drop_count);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("UDP recv error: {}", e);
                }
            }
        }
    }
}
