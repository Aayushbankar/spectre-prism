
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::Result;

use crate::dispatcher::DispatcherSender;
use prism_common::{LogSource, RawEvent, ProvenanceMeta};
use bytes::Bytes;
use chrono::Utc;

pub struct TcpIngest {
    bind_addr: std::net::SocketAddr,
    sender: DispatcherSender,
}

impl TcpIngest {
    pub fn new(bind_addr: std::net::SocketAddr, sender: DispatcherSender) -> Self {
        Self { bind_addr, sender }
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        println!("TCP Listener started on {}", self.bind_addr);

        loop {
            let (socket, peer_addr) = listener.accept().await?;
            let sender_clone = self.sender.clone();
            
            // Spawn a task for each connection
            tokio::spawn(async move {
                let mut reader = BufReader::new(socket);
                let mut line = String::new();
                
                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            if line.ends_with('\n') {
                                line.pop();
                                if line.ends_with('\r') {
                                    line.pop();
                                }
                            }
                            
                            if line.is_empty() {
                                continue;
                            }
                            
                            // Send payload to dispatcher
                            let payload_bytes = Bytes::copy_from_slice(line.as_bytes());
                            let source = LogSource::NetworkTcp { 
                                ip: peer_addr.ip().to_string(), 
                                port: peer_addr.port() 
                            };
                            let hash = blake3::hash(&payload_bytes);
                            let timestamp = Utc::now();
                            
                            let event = RawEvent {
                                payload: payload_bytes,
                                metadata: ProvenanceMeta {
                                    hash: hash.into(),
                                    timestamp,
                                    source,
                                },
                            };
                            
                            // If channel is full, this natively blocks the socket reader, applying TCP backpressure
                            let _ = sender_clone.try_broadcast(event);
                        }
                        Err(e) => {
                            eprintln!("TCP read error from {}: {}", peer_addr, e);
                            break;
                        }
                    }
                }
            });
        }
    }
}
