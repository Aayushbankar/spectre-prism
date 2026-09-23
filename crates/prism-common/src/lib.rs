use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Use blake3::Hash natively now that we have the serde feature
pub type Blake3Hash = blake3::Hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSource {
    Udp(SocketAddr),
    Quic(SocketAddr),
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    // We can't directly serialize Bytes nicely without a custom serializer or 
    // wrapping it, but since bytes::Bytes doesn't implement Serialize out of the box,
    // we use `#[serde(with = "bytes_ser")]` or just use a wrapper. Wait, `bytes` doesn't have `serde` enabled.
    // Let's use `serde_bytes` or enable `serde` feature on `bytes`.
    // Actually, `bytes` crate has a `serde` feature! Let's enable it.
    pub payload: Bytes,
    pub source: LogSource,
    pub timestamp: DateTime<Utc>,
    pub metadata: ProvenanceMeta, // Added ProvenanceMeta directly here to pass hash
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMeta {
    pub hash: Blake3Hash,
    pub timestamp: DateTime<Utc>,
    pub source: LogSource,
}

// Fixed OcsfNetworkActivity based on the audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfNetworkActivity {
    pub activity_id: u32,
    pub category_uid: u32,
    pub class_uid: u32,
    pub time: i64,
    pub src_endpoint: Endpoint,
    pub dst_endpoint: Endpoint,
    pub raw_data: Option<String>,
    pub metadata: VaultMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub vault_uri: String,
    pub provenance: ProvenanceMeta,
}
