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

#[test]
fn test_real_data_samples() {
    let samples = vec![
        "fortinet_fortigate.log",
        "cisco_asa.log",
        "paloalto_threat.log",
    ];
    let vrl = VrlEngine::new().unwrap();

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
