use prism_ingest::common::IngestConfig;
use prism_ingest::dispatcher::Dispatcher;
use prism_ingest::listener::UdpListener;
use prism_common::LogSource;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::time::{timeout, Duration};
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[tokio::test]
async fn test_udp_ingest_heterogeneous_concurrent() {
    let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let config = IngestConfig {
        udp_bind_addr: bind_addr,
        chunk_size: 10 * 1024 * 1024,
        channel_capacity: 50_000,
    };

    let dispatcher = Dispatcher::new(&config);
    let data_rx = dispatcher.data_receiver();
    let prov_rx = dispatcher.provenance_receiver();
    let drop_count = dispatcher.drop_count();
    
    let listener = Arc::new(UdpListener::new(config, dispatcher.sender()).await.unwrap());
    let local_addr = listener.local_addr().unwrap();
    
    let listener_clone = listener.clone();
    tokio::spawn(async move {
        listener_clone.run().await.unwrap();
    });

    let num_senders = 10;
    let pkts_per_sender = 2000;
    let mut join_handles = vec![];

    let start_time = chrono::Utc::now();

    let payloads: Vec<&[u8]> = vec![
        b"<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8",
        b"%ASA-6-302013: Built inbound TCP connection 2343234 for outside:10.1.1.1/1234 (10.1.1.1/1234)",
        b"date=2020-01-01 time=12:34:56 devname=FW01 devid=FGT60C123456 logid=0000000013 type=traffic subtype=forward",
        b"1,2019/01/01 10:00:00,001234567890,TRAFFIC,start,1,2019/01/01 10:00:00,10.0.0.1,10.0.0.2",
    ];

    for _ in 0..num_senders {
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

    let expected_total = num_senders * pkts_per_sender;
    
    let result = timeout(Duration::from_secs(15), async {
        let mut received_count = 0;
        while let Ok(data_event) = data_rx.recv_async().await {
            let prov_event = prov_rx.recv_async().await.unwrap();

            assert!(payloads.contains(&&data_event.payload[..]), "Received unknown/corrupted payload");
            assert_eq!(&data_event.payload[..], &prov_event.payload[..]);

            let expected_hash = blake3::hash(&data_event.payload);
            assert_eq!(data_event.metadata.hash, expected_hash);
            assert_eq!(prov_event.metadata.hash, expected_hash);
            
            assert!(matches!(data_event.metadata.source, LogSource::Udp(_)));

            let event_time = data_event.metadata.timestamp;
            assert!(event_time >= start_time, "Timestamp from before test start");

            received_count += 1;
            if received_count == expected_total {
                break;
            }
        }
        received_count
    }).await;

    // Now await the senders just to be clean
    for h in join_handles {
        h.await.unwrap();
    }

    assert!(result.is_ok(), "Timed out or failed");
    let received_count = result.unwrap();
    assert_eq!(received_count, expected_total);
    assert_eq!(drop_count.load(Ordering::SeqCst), 0, "Expected 0 dropped packets");
}

#[tokio::test]
async fn test_drop_under_pressure() {
    let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let config = IngestConfig {
        udp_bind_addr: bind_addr,
        chunk_size: 10 * 1024 * 1024,
        channel_capacity: 10, // Tiny capacity
    };

    let dispatcher = Dispatcher::new(&config);
    let drop_count = dispatcher.drop_count();
    
    let listener = Arc::new(UdpListener::new(config, dispatcher.sender()).await.unwrap());
    let local_addr = listener.local_addr().unwrap();
    
    let listener_clone = listener.clone();
    tokio::spawn(async move {
        listener_clone.run().await.unwrap();
    });

    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let payload = b"short test message";
    for _ in 0..100 {
        client_socket.send_to(payload, local_addr).await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let drops = drop_count.load(Ordering::SeqCst);
    assert!(drops > 0, "Expected dropped packets under pressure");
}

#[tokio::test]
async fn test_capacity_invariance_100k() {
    let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let config = IngestConfig {
        udp_bind_addr: bind_addr,
        chunk_size: 10 * 1024 * 1024,
        channel_capacity: 100_000,
    };

    let dispatcher = Dispatcher::new(&config);
    let data_rx = dispatcher.data_receiver();
    let prov_rx = dispatcher.provenance_receiver();
    
    let listener = Arc::new(UdpListener::new(config, dispatcher.sender()).await.unwrap());
    let local_addr = listener.local_addr().unwrap();
    
    let listener_clone = listener.clone();
    tokio::spawn(async move {
        listener_clone.run().await.unwrap();
    });

    let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    
    // Blast 100,000 tiny packets to prove capacity amortization
    let payload = b"x";
    
    let sender_handle = tokio::spawn(async move {
        for j in 0..100_000 {
            client_socket.send_to(payload, local_addr).await.unwrap();
            if j % 100 == 0 {
                tokio::task::yield_now().await;
            }
        }
    });

    let result = timeout(Duration::from_secs(10), async {
        let mut count = 0;
        while let Ok(_) = data_rx.recv_async().await {
            let _ = prov_rx.recv_async().await.unwrap();
            count += 1;
            if count == 100_000 {
                break;
            }
        }
    }).await;

    sender_handle.await.unwrap();
    assert!(result.is_ok(), "Failed to process 100,000 packets within timeout - indicating potential GC/allocation stalls");
}
