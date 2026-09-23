# Changelog

All notable changes to this project will be documented in this file.

## [0.9.0] 2026-09-23 — feat: Phase 3 Control Plane init
- Bootstrapped `prism-brain` with `watcher`, `cluster` (Drain3), `triage` (Open Jev), `coder` (Ollama), and `gatekeeper` (HitL) modules.
- Added end-to-end `pytest` gate testing NGINX alien log parsing.

## [0.8.0] 2026-09-23 — feat: Phase 2 Data Plane (f428b5d+2e85f9e) — HeuristicRouter memchr 1M <5s, VRL compile per-vendor srcip/outside/,ip, OcsfMapper category_uid 4, DLQ /var/run/prism/dlq.log create_dir_all, HttpSink httptest mock, 50k heter Fortinet/Cisco/Palo test

## [0.7.0] 2026-09-23 — docs: sync Phase1 ingestion plane (ac202e1)
- Synchronized documentation (CHANGELOG, diagrams, IPC contract) to match code reality.

## [0.6.0] 2026-09-23 — fix: fourth iteration max strictness audit and documentation gap (9d8f39f)
- Added `utf8:` and `b64:` prefix tag to `RawEvent` payload to solve heuristic Base64 corruption (`TWFu` -> `Man`).
- Added rollback `data_rx.try_recv` in `dispatcher.rs` to guarantee absolute atomicity on partial provenance send failure.

## [0.5.0] 2026-09-23 — fix: third iteration audit findings (ecd2208)
- Updated `deserialize_bytes` to use `STANDARD.decode` fallback.
- Added `test_capacity_invariance_100k` and modified integration tests to drain concurrently.
- Re-scoped `quinn` to `prism-ingest` and removed unused `hex` crate.

## [0.4.0] 2026-09-23 — fix: added QUIC stub (0b4ed84)
- Added initial QUIC stub leveraging `quinn` and `rcgen` self-signed certs.

## [0.3.0] 2026-09-23 — fix: second iteration audit (e8d16f7)
- Replaced `buffer_size` with `chunk_size` explicit contract (ADR implied).
- Verified `SO_RCVBUF` OS socket tuning via `socket2`.
- Updated `OcsfNetworkActivity` to strictly support Class 4001 fields.
- Replaced boolean drop state with atomic `drop_count` (`AtomicU64`).
- Deleted hollow crates `prism-provenance`, `prism-core`, and `prism-tui`.

## [0.2.0] 2026-09-23 — fix: address audit (eff7c04)
- Fixed buffer reallocation by employing `BytesMut::with_capacity`.
- Implemented `flume` dual fan-out channels.
- Added cryptographic BLAKE3 in-flight hashing on payloads.
- Validated heterogeneous payload lengths using 20k burst test.

## [0.1.0] 2026-09-23 — feat: Phase1 (4bab104)
- Initial workspace scaffold.
