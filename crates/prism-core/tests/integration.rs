use prism_core::{
    router::{HeuristicRouter, Vendor},
    vrl::VrlEngine,
    ocsf::OcsfMapper,
    dlq::DeadLetterQueue,
    sink::HttpSink,
};
use prism_common::{RawEvent, ProvenanceMeta, LogSource, OcsfNetworkActivity, Endpoint, VaultMetadata};
use bytes::Bytes;
use chrono::Utc;
use std::fs;
use httptest::{Server, Expectation, matchers::*, responders::*};

#[tokio::test]
async fn test_data_plane_routing() {
    let dlq_path = "/tmp/prism_dlq.log";
    let _ = fs::remove_file(dlq_path);
    let mut dlq = DeadLetterQueue::new(Some(dlq_path)).unwrap();
    let vrl = VrlEngine::new().unwrap();
    
    // Test 50,000 heterogeneous logs
    for i in 0..50_000 {
        let payload = match i % 3 {
            0 => format!("date=2024-01-01 time=12:00:00 devname=\"FW01\" devid=\"FG100\" logid=\"0000000013\" type=\"traffic\" subtype=\"forward\" level=\"notice\" srcip=192.168.1.5 dstip=8.8.8.8 action=\"accept\" idx={}", i).into_bytes(),
            1 => format!("%ASA-6-302013: Built inbound TCP connection 44439167 for outside:192.168.1.5/54321 to inside:10.0.0.1/80 idx={}", i).into_bytes(),
            _ => format!("1,2024/01/01 12:00:00,0011C1234,THREAT,vulnerability,1,2024/01/01 12:00:00,192.168.1.5,10.0.0.1 idx={}", i).into_bytes(),
        };
        
        let hash = blake3::hash(&payload);
        let vendor = HeuristicRouter::route(&payload);
        assert!(vendor != Vendor::Unknown);
        
        let payload_str = std::str::from_utf8(&payload).unwrap();
        let parsed = vrl.process(&vendor, payload_str).unwrap();
        let ocsf = OcsfMapper::map(parsed, &hash.to_hex(), Utc::now().timestamp_millis());
        
        assert_eq!(ocsf.class_uid, 4001);
        assert_eq!(ocsf.metadata.version, "1.9.0");
        assert_eq!(ocsf.metadata.provenance_hash, hash.to_hex().as_str());
        assert_eq!(ocsf.src_endpoint.ip, "192.168.1.5");
    }

    // Unknown -> DLQ
    let alien = b"alien raw bytes";
    let vendor = HeuristicRouter::route(alien);
    assert_eq!(vendor, Vendor::Unknown);
    
    let alien_event = RawEvent {
        payload: Bytes::from(alien.to_vec()),
        metadata: ProvenanceMeta { hash: blake3::hash(alien), timestamp: Utc::now(), source: LogSource::Unknown },
    };
    dlq.push(&alien_event, "Unknown Vendor").unwrap();
    
    let dlq_contents = fs::read_to_string(dlq_path).unwrap();
    assert!(dlq_contents.contains("alien raw bytes"));
}

#[tokio::test]
async fn test_sink_http_mock() {
    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("POST", "/bulk"))
            .respond_with(status_code(200)),
    );

    let url = server.url_str("/bulk");
    let sink = HttpSink::new(&url);
    let batch = vec![OcsfNetworkActivity {
        activity_id: 1,
        category_uid: 4,
        class_uid: 4001,
        severity_id: 1,
        severity: "Informational".to_string(),
        status_id: 1,
        confidence: 100,
        type_uid: 400101,
        time: 123456,
        src_endpoint: Endpoint { ip: "1.1.1.1".to_string(), port: 0 },
        dst_endpoint: Endpoint { ip: "2.2.2.2".to_string(), port: 0 },
        observables: vec![],
        raw_data: None,
        metadata: VaultMetadata {
            version: "1.9.0".to_string(),
            vault_uri: "local".to_string(),
            provenance_hash: "hash".to_string(),
        },
    }];

    let res = sink.push_bulk(&batch).await;
    assert!(res.is_ok());
}
