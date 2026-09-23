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
    // IPC contract expects string for dlq.log
    if let Ok(s) = std::str::from_utf8(bytes) {
        serializer.serialize_str(s)
    } else {
        use base64::Engine;
        serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }
}

fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(Bytes::from(s.into_bytes()))
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
pub struct OcsfNetworkActivity {
    pub activity_id: u32,
    pub category_uid: u32,
    pub class_uid: u32,
    pub severity_id: u32,
    pub status_id: u32,
    pub time: i64,
    pub src_endpoint: Endpoint,
    pub dst_endpoint: Endpoint,
    pub observables: Vec<String>,
    pub raw_data: Option<String>,
    pub metadata: VaultMetadata,
}
