use vrl::value::Value;
use prism_common::{
    OcsfEvent, OcsfNetworkActivity, OcsfAuthentication, OcsfWebActivity, OcsfFileActivity,
    OcsfBaseEvent, Endpoint, VaultMetadata, NetworkInfo, FileInfo, HttpInfo, HttpRequest, HttpResponse, UserInfo,
    OCSF_CLASS_UID_NETWORK_ACTIVITY, OCSF_CLASS_UID_AUTHENTICATION, OCSF_CLASS_UID_WEB_ACTIVITY, OCSF_CLASS_UID_FILE_ACTIVITY,
};
use std::collections::BTreeMap;

pub struct OcsfMapper;

impl OcsfMapper {
    pub fn map(value: Value, hash: &str, timestamp: i64) -> OcsfEvent {
        let obj = value.as_object();
        
        let mut class_uid = OCSF_CLASS_UID_NETWORK_ACTIVITY;
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
        let mut user_name = String::new();
        let mut user_domain = String::new();
        let mut url = String::new();
        let mut http_method = String::new();
        let mut http_status = 0;
        let mut file_name = String::new();
        let mut file_path = String::new();
        let mut file_size = 0;
        
        let mut fields_found = 0;
        let key_fields = 4;
        
        let mut unmapped = BTreeMap::new();

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
                    "class_uid" => class_uid = val_str.parse().unwrap_or(OCSF_CLASS_UID_NETWORK_ACTIVITY),
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
                    "user" | "username" | "srcuser" => user_name = val_str,
                    "domain" | "srcdomain" => user_domain = val_str,
                    "url" | "request_url" => url = val_str,
                    "method" | "http_method" => http_method = val_str,
                    "status" | "http_status" => http_status = val_str.parse().unwrap_or(0),
                    "filename" | "file_name" => file_name = val_str,
                    "filepath" | "file_path" => file_path = val_str,
                    "filesize" | "file_size" => file_size = val_str.parse().unwrap_or(0),
                    "message" => {}
                    _ => {
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
            "accept" | "allow" | "success" | "pass" => (1, 1),
            "deny" | "drop" | "block" | "fail" | "failure" => (2, 2),
            "login" | "logon" => (1, 1),
            "logout" | "logoff" => (2, 1),
            _ => (0, 0),
        };
        
        let metadata = VaultMetadata {
            version: "1.9.0".to_string(),
            vault_uri: "local".to_string(),
            provenance_hash: hash.to_string(),
        };
        
        let unmapped_val = if unmapped.is_empty() { 
            None 
        } else { 
            let map: serde_json::Map<String, serde_json::Value> = unmapped.into_iter().collect();
            Some(serde_json::Value::Object(map)) 
        };
        
        let base = OcsfBaseEvent {
            activity_id,
            category_uid,
            class_uid,
            severity_id,
            severity: severity.to_string(),
            status_id,
            confidence,
            type_uid,
            time: timestamp,
            metadata,
            unmapped: unmapped_val,
        };
        
        let src_endpoint = Endpoint { ip: src_ip.clone(), port: src_port };
        let dst_endpoint = Endpoint { ip: dst_ip.clone(), port: dst_port };

        let event = match class_uid {
            OCSF_CLASS_UID_AUTHENTICATION => {
                let user = if !user_name.is_empty() {
                    Some(UserInfo {
                        name: user_name,
                        domain: if user_domain.is_empty() { None } else { Some(user_domain) },
                        uid: None,
                        groups: vec![],
                    })
                } else {
                    None
                };
                OcsfEvent::Authentication(OcsfAuthentication {
                    base,
                    src_endpoint,
                    dst_endpoint,
                    user,
                    observables: vec![],
                    raw_data: obj.and_then(|m| m.get("message")).and_then(|v| {
                        if let Value::Bytes(b) = v { Some(String::from_utf8_lossy(b).to_string()) } else { None }
                    }),
                })
            }
            OCSF_CLASS_UID_WEB_ACTIVITY => {
                let http = if !http_method.is_empty() || http_status > 0 {
                    Some(HttpInfo {
                        request: if !http_method.is_empty() {
                            Some(HttpRequest {
                                method: http_method,
                                url: url.clone(),
                                user_agent: None,
                            })
                        } else { None },
                        response: if http_status > 0 {
                            Some(HttpResponse {
                                status_code: http_status,
                                content_type: None,
                            })
                        } else { None },
                    })
                } else { None };
                OcsfEvent::WebActivity(OcsfWebActivity {
                    base,
                    src_endpoint,
                    dst_endpoint,
                    http,
                    user_agent: None,
                    url: if url.is_empty() { None } else { Some(url) },
                    observables: vec![],
                    raw_data: obj.and_then(|m| m.get("message")).and_then(|v| {
                        if let Value::Bytes(b) = v { Some(String::from_utf8_lossy(b).to_string()) } else { None }
                    }),
                })
            }
            OCSF_CLASS_UID_FILE_ACTIVITY => {
                OcsfEvent::FileActivity(OcsfFileActivity {
                    base,
                    src_endpoint,
                    file: FileInfo {
                        name: if file_name.is_empty() { "unknown".to_string() } else { file_name },
                        path: file_path,
                        size: file_size,
                        hash: None,
                    },
                    observables: vec![],
                    raw_data: obj.and_then(|m| m.get("message")).and_then(|v| {
                        if let Value::Bytes(b) = v { Some(String::from_utf8_lossy(b).to_string()) } else { None }
                    }),
                })
            }
            _ => {
                let network_info = NetworkInfo {
                    protocol: proto,
                    direction: String::new(),
                    bytes_in: rcvdbyte,
                    bytes_out: sentbyte,
                };
                OcsfEvent::NetworkActivity(OcsfNetworkActivity {
                    base,
                    src_endpoint,
                    dst_endpoint,
                    observables: vec![],
                    raw_data: obj.and_then(|m| m.get("message")).and_then(|v| {
                        if let Value::Bytes(b) = v { Some(String::from_utf8_lossy(b).to_string()) } else { None }
                    }),
                    network: Some(network_info),
                })
            }
        };
        
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vrl::value::Value;

    #[test]
    fn test_dynamic_uid_mapping_network() {
        let mut map = BTreeMap::new();
        map.insert("class_uid".into(), Value::from(OCSF_CLASS_UID_NETWORK_ACTIVITY));
        map.insert("category_uid".into(), Value::from(4));
        map.insert("type_uid".into(), Value::from(400101));
        map.insert("srcip".into(), Value::from("10.0.0.1"));
        let val = Value::Object(map);
        let result = OcsfMapper::map(val, "hash", 0);
        assert_eq!(result.class_uid(), OCSF_CLASS_UID_NETWORK_ACTIVITY);
    }
    
    #[test]
    fn test_dynamic_uid_mapping_authentication() {
        let mut map = BTreeMap::new();
        map.insert("class_uid".into(), Value::from(OCSF_CLASS_UID_AUTHENTICATION));
        map.insert("category_uid".into(), Value::from(3));
        map.insert("type_uid".into(), Value::from(300101));
        map.insert("srcip".into(), Value::from("10.0.0.1"));
        map.insert("user".into(), Value::from("jdoe"));
        let val = Value::Object(map);
        let result = OcsfMapper::map(val, "hash", 0);
        assert_eq!(result.class_uid(), OCSF_CLASS_UID_AUTHENTICATION);
        match result {
            OcsfEvent::Authentication(auth) => {
                assert!(auth.user.is_some());
                assert_eq!(auth.user.unwrap().name, "jdoe");
            }
            _ => panic!("Expected Authentication event"),
        }
    }
    
    #[test]
    fn test_dynamic_uid_mapping_web() {
        let mut map = BTreeMap::new();
        map.insert("class_uid".into(), Value::from(OCSF_CLASS_UID_WEB_ACTIVITY));
        map.insert("category_uid".into(), Value::from(5));
        map.insert("type_uid".into(), Value::from(500101));
        map.insert("srcip".into(), Value::from("10.0.0.1"));
        map.insert("url".into(), Value::from("example.com/path"));
        map.insert("method".into(), Value::from("GET"));
        let val = Value::Object(map);
        let result = OcsfMapper::map(val, "hash", 0);
        assert_eq!(result.class_uid(), OCSF_CLASS_UID_WEB_ACTIVITY);
        match result {
            OcsfEvent::WebActivity(web) => {
                assert!(web.http.is_some());
                assert_eq!(web.http.unwrap().request.unwrap().method, "GET");
            }
            _ => panic!("Expected WebActivity event"),
        }
    }
    
    #[test]
    fn test_dynamic_uid_mapping_file() {
        let mut map = BTreeMap::new();
        map.insert("class_uid".into(), Value::from(OCSF_CLASS_UID_FILE_ACTIVITY));
        map.insert("category_uid".into(), Value::from(8));
        map.insert("type_uid".into(), Value::from(800101));
        map.insert("srcip".into(), Value::from("10.0.0.1"));
        map.insert("file_name".into(), Value::from("test.exe"));
        let val = Value::Object(map);
        let result = OcsfMapper::map(val, "hash", 0);
        assert_eq!(result.class_uid(), OCSF_CLASS_UID_FILE_ACTIVITY);
        match result {
            OcsfEvent::FileActivity(file) => {
                assert_eq!(file.file.name, "test.exe");
            }
            _ => panic!("Expected FileActivity event"),
        }
    }
}