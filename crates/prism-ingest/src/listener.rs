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
        
        let desired_buf_size = 8 * 1024 * 1024;
        let _ = socket2_sock.set_recv_buffer_size(desired_buf_size);
        
        socket2_sock.bind(&config.udp_bind_addr.into())?;
        socket2_sock.set_nonblocking(true)?;

        // Verify SO_RCVBUF
        if let Ok(size) = socket2_sock.recv_buffer_size() {
            if size < desired_buf_size {
                eprintln!("Warning: Expected SO_RCVBUF of {} but got {}. This may cause packet drops.", desired_buf_size, size);
            }
        }

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
        let chunk_size = self.config.chunk_size;
        let mut buf = BytesMut::with_capacity(chunk_size);
        
        loop {
            // Reserve exactly chunk_size when we don't have space for one max packet.
            // This maintains amortized O(1) allocations.
            if buf.capacity() < 65536 {
                buf.reserve(chunk_size);
            }
            
            match self.socket.recv_buf_from(&mut buf).await {
                Ok((len, addr)) => {
                    if len == 0 {
                        continue;
                    }

                    // Strict boundary hashing before freeze
                    let payload_slice = &buf[..len];
                    let hash = blake3::hash(payload_slice);

                    // Zero-copy freeze
                    let data = buf.split_to(len).freeze();
                    
                    let timestamp = Utc::now();
                    let source = LogSource::Udp(addr);

                    let payload = RawEvent {
                        payload: data,
                        metadata: ProvenanceMeta {
                            hash,
                            timestamp,
                            source,
                        },
                    };
                    
                    // Atomically send to dual planes. Drops are internally tracked by DispatcherSender.
                    let _ = self.sender.try_broadcast(payload);
                }
                Err(e) => {
                    eprintln!("UDP recv error: {}", e);
                }
            }
        }
    }
}
