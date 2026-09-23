use std::sync::Arc;
use tokio::net::UdpSocket;
use bytes::BytesMut;
use flume::Sender;
use chrono::Utc;
use prism_common::{LogSource, RawEvent};
use crate::common::IngestConfig;

pub struct UdpListener {
    socket: Arc<UdpSocket>,
    sender: Sender<RawEvent>,
    config: IngestConfig,
}

impl UdpListener {
    pub async fn new(config: IngestConfig, sender: Sender<RawEvent>) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(config.udp_bind_addr).await?;
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
        let mut buf = BytesMut::with_capacity(self.config.buffer_size);
        
        loop {
            if buf.capacity() < 65536 {
                buf.reserve(self.config.buffer_size.max(65536));
            }
            
            match self.socket.recv_buf_from(&mut buf).await {
                Ok((len, addr)) => {
                    if len == 0 {
                        continue;
                    }
                    let data = buf.split_to(len).freeze();
                    let payload = RawEvent {
                        payload: data,
                        source: LogSource::Udp(addr),
                        timestamp: Utc::now(),
                    };
                    
                    if let Err(e) = self.sender.send_async(payload).await {
                        eprintln!("Failed to dispatch log: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("UDP recv error: {}", e);
                }
            }
        }
        Ok(())
    }
}
