use vrl::value::Value;
use prism_common::{OcsfNetworkActivity, Endpoint, VaultMetadata};

pub struct OcsfMapper;

impl OcsfMapper {
    pub fn map(value: Value, hash: &str, timestamp: i64) -> OcsfNetworkActivity {
        // VRL object contains the parsed IP if successful
        let src_ip = match value.as_object().and_then(|m| m.get("ip")) {
            Some(Value::Bytes(b)) => String::from_utf8_lossy(b).to_string(),
            _ => "0.0.0.0".to_string(),
        };
        let dst_ip = "0.0.0.0".to_string(); // Placeholder or extracted if needed

        OcsfNetworkActivity {
            activity_id: 1,
            category_uid: 4,
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
            raw_data: value.as_object().and_then(|m| m.get("message")).and_then(|v| {
                if let Value::Bytes(b) = v { Some(String::from_utf8_lossy(b).to_string()) } else { None }
            }),
            metadata: VaultMetadata {
                version: "1.9.0".to_string(),
                vault_uri: "local".to_string(),
                provenance_hash: hash.to_string(),
            },
        }
    }
}
