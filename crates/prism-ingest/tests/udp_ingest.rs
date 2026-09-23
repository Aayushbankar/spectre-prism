use prism_ingest::common::IngestConfig;
use prism_ingest::dispatcher::Dispatcher;
use prism_ingest::listener::UdpListener;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use std::sync::Arc;

#[tokio::test]
async fn test_udp_ingest_10k_packets() {
    let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let dispatcher = Dispatcher::new(20_000);
    let receiver = dispatcher.receiver();
    
    let config = IngestConfig {
        udp_bind_addr: bind_addr,
        buffer_size: 1024 * 1024 * 10, // 10MB to avoid reallocations
        channel_capacity: 20_000,
    };

    let listener = Arc::new(UdpListener::new(config, dispatcher.sender()).await.unwrap());
    let local_addr = listener.local_addr().unwrap();
    
    let listener_clone = listener.clone();
    tokio::spawn(async move {
        listener_clone.run().await.unwrap();
    });

    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    let num_packets = 10_000;
    let syslog_msg = b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
    
    // Blast 10,000 packets
    for i in 0..num_packets {
        client_socket.send_to(syslog_msg, local_addr).await.unwrap();
        if i % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }
    
    let mut received_count = 0;
    
    let result = timeout(Duration::from_secs(5), async {
        while let Ok(event) = receiver.recv_async().await {
            assert_eq!(&event.payload[..], syslog_msg);
            received_count += 1;
            if received_count == num_packets {
                break;
            }
        }
    }).await;
    
    assert!(result.is_ok(), "Timed out waiting for 10,000 packets. Received: {}", received_count);
    assert_eq!(received_count, num_packets);
}
