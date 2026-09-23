use prism_ingest::common::IngestConfig;
use prism_ingest::dispatcher::Dispatcher;
use prism_ingest::listener::UdpListener;
use prism_common::LogSource;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use std::sync::Arc;

#[tokio::test]
async fn test_udp_ingest_heterogeneous_concurrent() {
    let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let config = IngestConfig {
        udp_bind_addr: bind_addr,
        buffer_size: 10 * 1024 * 1024,
        channel_capacity: 50_000,
    };

    let dispatcher = Dispatcher::new(&config);
    // Use data_receiver (representing plane 2)
    let receiver = dispatcher.data_receiver();
    
    let listener = Arc::new(UdpListener::new(config, dispatcher.sender()).await.unwrap());
    let local_addr = listener.local_addr().unwrap();
    
    let listener_clone = listener.clone();
    tokio::spawn(async move {
        listener_clone.run().await.unwrap();
    });

    let num_senders = 10;
    let pkts_per_sender = 2000; // 20k total
    let mut join_handles = vec![];

    let payloads: Vec<&[u8]> = vec![
        b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8", // RFC5424
        b"%ASA-6-302013: Built inbound TCP connection 2343234 for outside:10.1.1.1/1234 (10.1.1.1/1234)", // Cisco ASA
        b"date=2020-01-01 time=12:34:56 devname=FW01 devid=FGT60C123456 logid=0000000013 type=traffic subtype=forward", // Fortinet KV
        b"1,2019/01/01 10:00:00,001234567890,TRAFFIC,start,1,2019/01/01 10:00:00,10.0.0.1,10.0.0.2", // Palo Alto CSV
    ];

    for i in 0..num_senders {
        let payloads = payloads.clone();
        join_handles.push(tokio::spawn(async move {
            let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            for j in 0..pkts_per_sender {
                let p = payloads[j % payloads.len()];
                client_socket.send_to(p, local_addr).await.unwrap();
                if j % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }));
    }

    for h in join_handles {
        h.await.unwrap();
    }

    let mut received_count = 0;
    let expected_total = num_senders * pkts_per_sender;
    let mut last_timestamp = chrono::Utc::now();
    let mut initially_set = false;

    // Use a longer timeout for CI stability
    let result = timeout(Duration::from_secs(10), async {
        while let Ok(event) = receiver.recv_async().await {
            // Verify payload integrity and hash
            let expected_hash = blake3::hash(&event.payload);
            assert_eq!(event.metadata.hash, expected_hash);
            
            // Verify LogSource
            match event.source {
                LogSource::Udp(_) => {},
                _ => panic!("Expected UDP source"),
            }
            match event.metadata.source {
                LogSource::Udp(_) => {},
                _ => panic!("Expected UDP metadata source"),
            }

            // Verify monotonicity (within reason, since concurrent, but single listener loop is monotonic)
            if !initially_set {
                last_timestamp = event.timestamp;
                initially_set = true;
            } else {
                assert!(event.timestamp >= last_timestamp, "Timestamps are not monotonic");
                last_timestamp = event.timestamp;
            }

            received_count += 1;
            if received_count == expected_total {
                break;
            }
        }
    }).await;

    assert!(result.is_ok(), "Timed out waiting for {} packets. Received: {}", expected_total, received_count);
    assert_eq!(received_count, expected_total);
}
