use serde_json::Value;
use prism_common::{OcsfNetworkActivity, Endpoint, VaultMetadata};

pub struct OcsfMapper;

impl OcsfMapper {
    pub fn map(value: Value, hash: &str, timestamp: i64) -> OcsfNetworkActivity {
        // Fallbacks if mapping fails, ideally read from VRL output `value`
        let src_ip = value.get("srcip").and_then(|v| v.as_str()).unwrap_or("0.0.0.0").to_string();
        let dst_ip = value.get("dstip").and_then(|v| v.as_str()).unwrap_or("0.0.0.0").to_string();

        OcsfNetworkActivity {
            activity_id: 1,
            category_uid: 2,
            class_uid: 4001,
            severity_id: 1,
            severity: "Informational".to_string(),
            status_id: 1,
            confidence: 100,
            type_uid: 400101,
            time: timestamp,
            src_endpoint: Endpoint {
                ip: src_ip,
                port: 0,
            },
            dst_endpoint: Endpoint {
                ip: dst_ip,
                port: 0,
            },
            observables: vec![],
            raw_data: value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string()),
            metadata: VaultMetadata {
                version: "1.9.0".to_string(),
                vault_uri: "local".to_string(),
                provenance_hash: hash.to_string(),
            },
        }
    }
}
