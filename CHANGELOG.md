# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Iteration 4 (Strictness Max & IPC Reliability)
- **Fixed:** Heuristic Base64 decoding bug in `prism-common` by implementing explicit `utf8:` and `b64:` prefixes. This guarantees zero corruption of valid UTF-8 binary logs that happen to resemble Base64.
- **Fixed:** `DispatcherSender` TOCTOU race condition and divergence risk. Implemented rollback `try_recv()` on `data_rx` if `provenance_tx` fails, guaranteeing atomic fan-out.
- **Fixed:** Documentation sync (updated `COMPONENT_DESIGN.md`, `DATA_DICTIONARY_AND_IPC.md`, and `HANDOFF_SESSION.md`).
- **Added:** Fully functional QUIC `quinn::Endpoint::server` stub with auto-generated self-signed certificates using `rcgen`.
- **Added:** `CHANGELOG.md` trace.

### Iteration 3 (Strictness Verification)
- **Fixed:** Base64 fallback correctly implements `base64::engine::general_purpose::STANDARD`.
- **Changed:** Refactored `udp_ingest.rs` testing to spawn receivers concurrently with senders, providing a truthful assertion of memory capacity invariance and backpressure capability at sustained load.
- **Added:** `test_capacity_invariance_100k` blasting 100k packets without blocking to test GC stalling.
- **Removed:** Cleared unused `hex` dependency and migrated `quinn` out of `prism-common` into `prism-ingest`.

### Iteration 2 (Audit Remediation)
- **Fixed:** Explicit `chunk_size` implemented in `IngestConfig` instead of overriding `buffer_size`.
- **Fixed:** `listener.rs` now evaluates and logs `SO_RCVBUF` size warnings utilizing `socket2` configuration.
- **Fixed:** Cleaned LLM-generated debug comments and optimized `RawEvent` footprint (dropped duplicate timestamps/sources).
- **Added:** `OcsfNetworkActivity` updated with `severity_id`, `status_id`, `confidence`, `type_uid` for stricter OCSF 4001 compliance.
- **Added:** Backpressure and drop metrics tracked via `Arc<AtomicU64>`.

### Iteration 1 (Initial Setup)
- **Added:** Cargo Workspace initialized with `prism-common` and `prism-ingest`.
- **Added:** `BytesMut` zero-copy chunking via `split_to()`.
- **Added:** `UdpSocket` loop capturing syslogs on Port 514.
- **Added:** In-flight SIMD `blake3` hashing on ingestion chunk boundaries.
- **Added:** `flume` MPMC dual-channel setup (Data Plane & Integrity Plane).
