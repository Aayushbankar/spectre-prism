use vrl::value::Value;
use prism_common::{OcsfNetworkActivity, Endpoint, VaultMetadata};

pub struct OcsfMapper;

impl OcsfMapper {
    pub fn map(value: Value, hash: &str, timestamp: i64) -> OcsfNetworkActivity {
        let obj = value.as_object();
        
        let mut class_uid = 4001;
        let mut category_uid = 4;
        let mut type_uid = 400101;
        
        let mut src_ip = "0.0.0.0".to_string();
        let mut dst_ip = "0.0.0.0".to_string();
        let mut src_port = 0;
        let mut dst_port = 0;
        let mut action = String::new();
        let mut proto = String::new();
        let mut level = String::new();
        let mut sentbyte = 0;
        let mut rcvdbyte = 0;
        
        let mut fields_found = 0;
        let key_fields = 4; // src_ip, dst_ip, src_port, dst_port
        
        let mut unmapped = serde_json::Map::new();

        if let Some(m) = obj {
            for (k, v) in m.iter() {
                let key_str = k.as_str();
                let val_str = if let Value::Bytes(b) = v {
                    String::from_utf8_lossy(b).to_string()
                } else if let Value::Null = v {
                    continue;
                } else {
                    v.to_string()
                };

                match key_str {
                    "class_uid" => class_uid = val_str.parse().unwrap_or(4001),
                    "category_uid" => category_uid = val_str.parse().unwrap_or(4),
                    "type_uid" => type_uid = val_str.parse().unwrap_or(400101),
                    "srcip" => { src_ip = val_str; fields_found += 1; }
                    "dstip" => { dst_ip = val_str; fields_found += 1; }
                    "srcport" => { 
                        src_port = val_str.parse().unwrap_or(0); 
                        fields_found += 1; 
                    }
                    "dstport" => { 
                        dst_port = val_str.parse().unwrap_or(0); 
                        fields_found += 1; 
                    }
                    "action" => action = val_str,
                    "proto" => proto = val_str,
                    "level" => level = val_str,
                    "sentbyte" => sentbyte = val_str.parse().unwrap_or(0),
                    "rcvdbyte" => rcvdbyte = val_str.parse().unwrap_or(0),
                    "message" => {}
                    _ => {
                        // Unmapped vendor fields
                        unmapped.insert(format!("vendor://{}", key_str), serde_json::Value::String(val_str));
                    }
                }
            }
        }

        let confidence = if fields_found >= key_fields { 90 } else { 50 };

        let (severity_id, severity) = match level.to_lowercase().as_str() {
            "notice" => (2, "Notice"),
            "warning" => (3, "Warning"),
            "error" => (4, "Error"),
            "critical" => (5, "Critical"),
            _ => (1, "Informational"),
        };

        let (activity_id, status_id) = match action.to_lowercase().as_str() {
            "accept" | "allow" => (1, 1), // Allow -> Success
            "deny" | "drop" | "block" => (2, 2), // Deny -> Failure
            _ => (0, 0), // Unknown
        };
        
        let unmapped_val = if unmapped.is_empty() { None } else { Some(serde_json::Value::Object(unmapped)) };
        
        let network_info = prism_common::NetworkInfo {
            protocol: proto,
            direction: String::new(),
            bytes_in: rcvdbyte,
            bytes_out: sentbyte,
        };

        OcsfNetworkActivity {
            activity_id,
            category_uid,
            class_uid,
            severity_id,
            severity: severity.to_string(),
            status_id,
            confidence,
            type_uid,
            time: timestamp,
            src_endpoint: Endpoint {
                ip: src_ip,
                port: src_port,
            },
            dst_endpoint: Endpoint {
                ip: dst_ip,
                port: dst_port,
            },
            observables: vec![],
            raw_data: obj.and_then(|m| m.get("message")).and_then(|v| {
                if let Value::Bytes(b) = v { Some(String::from_utf8_lossy(b).to_string()) } else { None }
            }),
            metadata: VaultMetadata {
                version: "1.9.0".to_string(),
                vault_uri: "local".to_string(),
                provenance_hash: hash.to_string(),
            },
            unmapped: unmapped_val,
            network: Some(network_info),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use vrl::value::Value;

    #[test]
    fn test_dynamic_uid_mapping() {
        let mut map = std::collections::BTreeMap::new();
        map.insert("class_uid".to_string(), Value::from(4002));
        map.insert("category_uid".to_string(), Value::from(5));
        map.insert("type_uid".to_string(), Value::from(400201));
        map.insert("srcip".to_string(), Value::from("10.0.0.1"));
        let val = Value::Object(map);
        let result = OcsfMapper::map(val, "hash", 0);
        assert_eq!(result.class_uid, 4002);
        assert_eq!(result.category_uid, 5);
        assert_eq!(result.type_uid, 400201);
    }
}
