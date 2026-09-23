use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub type Blake3Hash = [u8; 32];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogSource {
    Udp(SocketAddr),
    Quic(SocketAddr),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RawEvent {
    pub payload: Bytes,
    pub source: LogSource,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMeta {
    pub hash: Blake3Hash,
    pub timestamp: DateTime<Utc>,
    pub source: LogSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcsfNetworkActivity {
    pub activity_id: u32,
    pub category_uid: u32,
    pub class_uid: u32,
    pub raw_data: Option<String>,
    pub metadata: ProvenanceMeta,
    // Add other fields as required by OCSF 4001
}
