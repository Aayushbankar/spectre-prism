use prism_core::{
    router::{HeuristicRouter, Vendor},
    vrl::VrlEngine,
    ocsf::OcsfMapper,
    dlq::DeadLetterQueue,
    sink::HttpSink,
};
use prism_common::{RawEvent, ProvenanceMeta, LogSource, OcsfEvent, Endpoint, VaultMetadata};
use bytes::Bytes;
use chrono::Utc;
use std::fs;
use httptest::{Server, Expectation, matchers::*, responders::*};

fn setup_test_rules(dir: &std::path::Path) {
    let _ = fs::create_dir_all(dir);
    let fortinet = r#"
        m1 = parse_regex(string!(.message), r'(?s).*?srcip=(?P<srcip>\d+\.\d+\.\d+\.\d+).*') ?? {}
        if exists(m1.srcip) { .srcip = m1.srcip }
        m2 = parse_regex(string!(.message), r'(?s).*?dstip=(?P<dstip>\d+\.\d+\.\d+\.\d+).*') ?? {}
        if exists(m2.dstip) { .dstip = m2.dstip }
        m3 = parse_regex(string!(.message), r'(?s).*?srcport=(?P<srcport>\d+).*') ?? {}
        if exists(m3.srcport) { .srcport = m3.srcport }
        m4 = parse_regex(string!(.message), r'(?s).*?dstport=(?P<dstport>\d+).*') ?? {}
        if exists(m4.dstport) { .dstport = m4.dstport }
        m5 = parse_regex(string!(.message), r'(?s).*?action="(?P<action>[^"]+)".*') ?? {}
        if exists(m5.action) { .action = m5.action }
        m6 = parse_regex(string!(.message), r'(?s).*?level="(?P<level>[^"]+)".*') ?? {}
        if exists(m6.level) { .level = m6.level }
    "#;

    let cisco = r#"
        m1 = parse_regex(string!(.message), r'(?s).*?outside:(?P<srcip>\d+\.\d+\.\d+\.\d+).*') ?? {}
        if exists(m1.srcip) { .srcip = m1.srcip }
        m2 = parse_regex(string!(.message), r'(?s).*?outside:\d+\.\d+\.\d+\.\d+/(?P<srcport>\d+).*') ?? {}
        if exists(m2.srcport) { .srcport = m2.srcport }
        m3 = parse_regex(string!(.message), r'(?s).*?inside:(?P<dstip>\d+\.\d+\.\d+\.\d+).*') ?? {}
        if exists(m3.dstip) { .dstip = m3.dstip }
        m4 = parse_regex(string!(.message), r'(?s).*?inside:\d+\.\d+\.\d+\.\d+/(?P<dstport>\d+).*') ?? {}
        if exists(m4.dstport) { .dstport = m4.dstport }
        m5 = parse_regex(string!(.message), r'(?s).*?%ASA-\d-(?P<msgid>\d+).*') ?? {}
        if exists(m5.msgid) { .msgid = m5.msgid }
    "#;

    let palo = r#"
        parts = split(string!(.message), ",")
        .srcip = parts[7]
        .dstip = parts[8]
        .srcport = parts[24]
        .dstport = parts[25]
        .action = parts[29]
    "#;

    fs::write(dir.join("fortinet.vrl"), fortinet).unwrap();
    fs::write(dir.join("cisco.vrl"), cisco).unwrap();
    fs::write(dir.join("palo.vrl"), palo).unwrap();
}

fn get_network_activity(ocsf: &OcsfEvent) -> Option<&prism_common::OcsfNetworkActivity> {
    match ocsf {
        OcsfEvent::NetworkActivity(na) => Some(na),
        _ => None,
    }
}

#[tokio::test]
async fn test_data_plane_routing() {
    let _ = std::process::Command::new("docker")
        .args(["compose", "down", "-v"])
        .current_dir("../../")
        .output();

    // Start docker-compose if docker is available
    let docker_started = match std::process::Command::new("docker")
        .args(["compose", "up", "-d", "elasticsearch", "--wait", "--wait-timeout", "60"])
        .current_dir("../../")
        .output()
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    };
    
    // give it an extra moment if needed, but --wait should block until healthy

    let dlq_path = "/tmp/prism_dlq_integration_test.log";
    let _ = fs::remove_file(dlq_path);
    let mut dlq = DeadLetterQueue::new(Some(dlq_path)).unwrap();
    
    let rules_dir = std::path::Path::new("/tmp/prism_test_rules_integration");
    setup_test_rules(rules_dir);
    let vrl = VrlEngine::new(Some(rules_dir)).unwrap();
    
    let mut batch = Vec::new();
    
    let client = reqwest::Client::new();
    let es_available = client.get("http://localhost:9200").send().await.map(|r| r.status().is_success()).unwrap_or(false);

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
        
        let network_activity = get_network_activity(&ocsf).expect("Expected NetworkActivity");
        
        assert_eq!(ocsf.class_uid(), 4001, "Failed for i={}", i);
        assert_eq!(network_activity.base.metadata.version, "1.9.0", "Failed for i={}", i);
        assert_eq!(network_activity.base.metadata.provenance_hash, hash.to_hex().as_str(), "Failed for i={}", i);
        assert_eq!(network_activity.src_endpoint.ip, "192.168.1.5", "Failed for i={}, payload={}, ocsf={:?}", i, payload_str, ocsf);
        
        match i % 3 {
            0 => {
                assert_eq!(network_activity.dst_endpoint.ip, "8.8.8.8");
                assert_eq!(network_activity.base.severity_id, 2);
                assert_eq!(network_activity.base.activity_id, 1);
            }
            1 => {
                assert_eq!(network_activity.dst_endpoint.ip, "10.0.0.1");
                assert_eq!(network_activity.src_endpoint.port, 54321);
                assert_eq!(network_activity.dst_endpoint.port, 80);
                assert_eq!(network_activity.base.severity_id, 1);
            }
            _ => {
                assert_eq!(network_activity.base.severity_id, 1);
            }
        }
        
        if es_available {
            batch.push(ocsf);
            if batch.len() >= 10000 {
                let sink = HttpSink::new("http://localhost:9200/prism-ocsf/_bulk");
                let resp = sink.push_bulk(&batch).await;
                assert!(resp.is_ok(), "bulk failed {:?}", resp);
                batch.clear();
            }
        }
    }

    if es_available {
        // Refresh the index to make documents visible to search immediately
        let _ = client.post("http://localhost:9200/prism-ocsf/_refresh").send().await;

        // Verify count
        let count_resp = client.get("http://localhost:9200/prism-ocsf/_count").send().await.unwrap();
        let count_json: serde_json::Value = count_resp.json().await.unwrap();
        assert_eq!(count_json["count"].as_u64().unwrap(), 50000);
    } else {
        println!("Elasticsearch not reachable at localhost:9200; validated 50,000 log routes & OCSF mappings in-memory");
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
    
    // Shut down docker compose if we started it
    if docker_started {
        let _ = std::process::Command::new("docker")
            .args(["compose", "down", "-v"])
            .current_dir("../../")
            .output();
    }
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
    let batch = vec![OcsfEvent::NetworkActivity(prism_common::OcsfNetworkActivity {
        base: prism_common::OcsfBaseEvent {
            activity_id: 1,
            category_uid: 4,
            class_uid: 4001,
            severity_id: 1,
            severity: "Informational".to_string(),
            status_id: 1,
            confidence: 100,
            type_uid: 400101,
            time: 123456,
            metadata: VaultMetadata {
                version: "1.9.0".to_string(),
                vault_uri: "local".to_string(),
                provenance_hash: "hash".to_string(),
            },
            unmapped: None,
        },
        src_endpoint: Endpoint { ip: "1.1.1.1".to_string(), port: 0 },
        dst_endpoint: Endpoint { ip: "2.2.2.2".to_string(), port: 0 },
        observables: vec![],
        raw_data: None,
        network: None,
    })];

    let res = sink.push_bulk(&batch).await;
    assert!(res.is_ok());
}