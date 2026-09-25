use prism_core::{HeuristicRouter, VrlEngine, ocsf::OcsfMapper};
use prism_core::accounting::{account_fortinet, account_cisco};
use std::fs;
use std::path::PathBuf;

fn get_data_dir() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d.pop();
    d.join("data").join("samples")
}

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
        if length(parts) > 29 {
            .srcip = parts[7]
            .dstip = parts[8]
            .srcport = parts[24]
            .dstport = parts[25]
            .action = parts[29]
        }
    "#;

    fs::write(dir.join("fortinet.vrl"), fortinet).unwrap();
    fs::write(dir.join("cisco.vrl"), cisco).unwrap();
    fs::write(dir.join("palo.vrl"), palo).unwrap();
}

#[tokio::test]
async fn test_real_data_samples() {
    let samples = vec![
        "fortinet_fortigate.log",
        "cisco_asa.log",
        "paloalto_threat.log",
    ];
    let rules_dir = std::path::Path::new("/tmp/prism_test_rules_real_data");
    setup_test_rules(rules_dir);
    let vrl = VrlEngine::new(Some(rules_dir)).unwrap();

    for sample in samples {
        let path = get_data_dir().join(sample);
        let content = fs::read_to_string(&path).expect(&format!("Could not read {:?}", path));
        for line in content.lines() {
            if line.trim().is_empty() { continue; }
            let vendor = HeuristicRouter::route(line.as_bytes());
            assert_ne!(vendor, prism_core::router::Vendor::Unknown, "Unknown vendor for line: {}", line);
            
            if let Ok(parsed) = vrl.process(&vendor, line) {
                let ocsf = OcsfMapper::map(parsed, "test_hash", 0);
                assert_ne!(ocsf.src_endpoint.ip, "0.0.0.0");
                assert_ne!(ocsf.src_endpoint.port, 0);
            }
            
            if sample == "fortinet_fortigate.log" {
                let acc = account_fortinet(line);
                assert!(acc.closure_ratio > 0.0);
            } else if sample == "cisco_asa.log" {
                let acc = account_cisco(line);
                assert!(acc.closure_ratio > 0.0);
            }
        }
    }
}
