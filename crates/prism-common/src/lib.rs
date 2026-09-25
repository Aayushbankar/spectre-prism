use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::net::SocketAddr;

pub type Blake3Hash = blake3::Hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSource {
    Udp(SocketAddr),
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
        // Fallback for legacy
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
pub struct OcsfNetworkActivity {
    pub activity_id: u32,
    pub category_uid: u32,
    pub class_uid: u32,
    pub severity_id: u32,
    pub severity: String,
    pub status_id: u32,
    pub confidence: u32,
    pub type_uid: u32,
    pub time: i64,
    pub src_endpoint: Endpoint,
    pub dst_endpoint: Endpoint,
    pub observables: Vec<String>,
    pub raw_data: Option<String>,
    pub metadata: VaultMetadata,
    pub unmapped: Option<serde_json::Value>,
    pub network: Option<NetworkInfo>,
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
