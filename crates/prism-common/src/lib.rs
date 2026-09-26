use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::net::SocketAddr;

pub type Blake3Hash = blake3::Hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSource {
    Udp(SocketAddr),
    NetworkTcp { ip: String, port: u16 },
    FileTail { path: String, offset: usize },
    Quic(SocketAddr),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMeta {
    #[serde(serialize_with = "serialize_hash", deserialize_with = "deserialize_hash")]
    pub hash: Blake3Hash,
    pub timestamp: DateTime<Utc>,
    pub source: LogSource,
}

fn serialize_hash<S>(hash: &Blake3Hash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hash.to_hex())
}

fn deserialize_hash<'de, D>(deserializer: D) -> Result<Blake3Hash, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    blake3::Hash::from_hex(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    #[serde(rename = "raw_payload", serialize_with = "serialize_bytes", deserialize_with = "deserialize_bytes")]
    pub payload: Bytes,
    pub metadata: ProvenanceMeta,
}

fn serialize_bytes<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Ok(s) = std::str::from_utf8(bytes) {
        serializer.serialize_str(&format!("utf8:{}", s))
    } else {
        use base64::Engine;
        serializer.serialize_str(&format!("b64:{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
    }
}

fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if let Some(utf8_str) = s.strip_prefix("utf8:") {
        Ok(Bytes::from(utf8_str.to_string().into_bytes()))
    } else if let Some(b64_str) = s.strip_prefix("b64:") {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64_str).map_err(serde::de::Error::custom)?;
        Ok(Bytes::from(decoded))
    } else {
        Ok(Bytes::from(s.into_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub version: String,
    pub vault_uri: String,
    pub provenance_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub protocol: String,
    pub direction: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpInfo {
    pub request: Option<HttpRequest>,
    pub response: Option<HttpResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status_code: u32,
    pub content_type: Option<String>,
}

pub const OCSF_CLASS_UID_NETWORK_ACTIVITY: u32 = 4001;
pub const OCSF_CLASS_UID_AUTHENTICATION: u32 = 3001;
pub const OCSF_CLASS_UID_WEB_ACTIVITY: u32 = 5001;
pub const OCSF_CLASS_UID_FILE_ACTIVITY: u32 = 8001;
pub const OCSF_CLASS_UID_PROCESS_ACTIVITY: u32 = 1001;
pub const OCSF_CLASS_UID_DNS_ACTIVITY: u32 = 6001;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfBaseEvent {
    pub activity_id: u32,
    pub category_uid: u32,
    pub class_uid: u32,
    pub severity_id: u32,
    pub severity: String,
    pub status_id: u32,
    pub confidence: u32,
    pub type_uid: u32,
    pub time: i64,
    pub metadata: VaultMetadata,
    pub unmapped: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfNetworkActivity {
    #[serde(flatten)]
    pub base: OcsfBaseEvent,
    pub src_endpoint: Endpoint,
    pub dst_endpoint: Endpoint,
    pub observables: Vec<String>,
    pub raw_data: Option<String>,
    pub network: Option<NetworkInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfAuthentication {
    #[serde(flatten)]
    pub base: OcsfBaseEvent,
    pub src_endpoint: Endpoint,
    pub dst_endpoint: Endpoint,
    pub user: Option<UserInfo>,
    pub observables: Vec<String>,
    pub raw_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfWebActivity {
    #[serde(flatten)]
    pub base: OcsfBaseEvent,
    pub src_endpoint: Endpoint,
    pub dst_endpoint: Endpoint,
    pub http: Option<HttpInfo>,
    pub user_agent: Option<String>,
    pub url: Option<String>,
    pub observables: Vec<String>,
    pub raw_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfFileActivity {
    #[serde(flatten)]
    pub base: OcsfBaseEvent,
    pub src_endpoint: Endpoint,
    pub file: FileInfo,
    pub observables: Vec<String>,
    pub raw_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub name: String,
    pub domain: Option<String>,
    pub uid: Option<String>,
    pub groups: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OcsfEvent {
    NetworkActivity(OcsfNetworkActivity),
    Authentication(OcsfAuthentication),
    WebActivity(OcsfWebActivity),
    FileActivity(OcsfFileActivity),
}

impl OcsfEvent {
    pub fn class_uid(&self) -> u32 {
        match self {
            OcsfEvent::NetworkActivity(e) => e.base.class_uid,
            OcsfEvent::Authentication(e) => e.base.class_uid,
            OcsfEvent::WebActivity(e) => e.base.class_uid,
            OcsfEvent::FileActivity(e) => e.base.class_uid,
        }
    }
    
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_raw_event_serialization_roundtrip() {
        let event = RawEvent {
            payload: Bytes::from(vec![0xff, 0xfe, 0xfd]),
            metadata: ProvenanceMeta {
                hash: blake3::hash(&[0xff, 0xfe, 0xfd]),
                timestamp: Utc::now(),
                source: LogSource::Unknown,
            }
        };

        let json = serde_json::to_string(&event).unwrap();
        let decoded: RawEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(event.payload, decoded.payload);
        assert_eq!(event.metadata.hash, decoded.metadata.hash);
    }
}