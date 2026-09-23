use prism_core::{
    router::{HeuristicRouter, Vendor},
    vrl::VrlEngine,
    ocsf::OcsfMapper,
    dlq::DeadLetterQueue,
};
use prism_common::{RawEvent, ProvenanceMeta, LogSource};
use bytes::Bytes;
use chrono::Utc;
use std::fs;

#[tokio::test]
async fn test_data_plane_routing() {
    let _ = fs::remove_file("dlq.log");
    let mut dlq = DeadLetterQueue::new("dlq.log").unwrap();
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
        
        // OCSF schema validation (basic invariants)
        assert_eq!(ocsf.class_uid, 4001);
        assert_eq!(ocsf.metadata.version, "1.9.0");
        assert_eq!(ocsf.metadata.provenance_hash, hash.to_hex().as_str());
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
    
    let dlq_contents = fs::read_to_string("dlq.log").unwrap();
    assert!(dlq_contents.contains("alien raw bytes"));
}
