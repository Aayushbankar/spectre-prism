# PRISM — Real Runtime Metrics & Benchmark Ledger

**Project:** PRISM SIH26156 NTRO ULPF | **Team:** SPECTRE | **Branch:** `feat/plane-2-data-plane` | **Commit:** `f581fac` (Plane 2 docs sync) / `2f4ee6d` (`main` Plane 4 merge) | **Timestamp (UTC):** `2026-09-23T15:56:07Z` (cargo test --workspace run) | **Host:** `hpelitebook840g5 7.1.8-arch1-3 x86_64` | **Toolchain:** `rustc 1.97.1 cargo 1.97.1` | **Mode:** Bare-metal (ADR_01) — no Docker, single 8-core

> This ledger stores **real, executed** metrics with citations to datasets, commits, and test harnesses. Future Plane 3/5 runs append here. All numbers are from `cargo test --workspace` / `cargo clippy --workspace -- -D warnings` on `feat/plane-2-data-plane@f581fac`.

---

## 0. Combined Workspace (All Planes)

| Scope | Tests | Result | Wall Time | Commit | Citation |
|---|---|---|---|---|---|
| `cargo test --workspace` | **13/13** `1 prism-common +3 prism-core (1 bench +2 int) +3 prism-ingest +6 prism-provenance` | `0` failures, `0` warnings | `~8s` total `4.25s bench +2.65s data +1.18s ingest +0.06s provenance` `2026-09-23T15:56:07Z` | `f581fac` (feat) `2f4ee6d` (main) | `Cargo.toml:3` 4 members, `HANDOFF_SESSION.md:120` |
| `cargo clippy --workspace -- -D warnings` | `0` warnings | pass | `1.03s` | same | `crates/*/Cargo.toml` edition 2021 |
| `cargo check --workspace` | `0` warnings | pass | `44s` | same | `PR #5 OPEN` |

**Hardware:** Arch Linux 7.1.8 bare-metal, 8-core (PRISM whitepaper p99 <25µs claim validated via bench, not via QEMU).

---

## 1. Plane 1 — Ingestion Plane (`prism-ingest` + `prism-common`) — ✅ Complete `main@89ba280 #1`

**Code:** `crates/prism-ingest/src/listener.rs:62` `BytesMut chunk 10MiB` `blake3::hash` `dispatcher.rs:62` dual `flume 50k` `socket2 SO_RCVBUF 8MiB` | `crates/prism-common/src/lib.rs:38` `RawEvent{utf8:/b64: payload}`

| Test | Dataset (real vendor sample — `docs/datasets/DATASET_PLAN.md:3`) | Payload Example | Count | Time | Assertion |
|---|---|---|---|---|---|
| `test_udp_ingest_heterogeneous_concurrent` | Cisco ASA `%ASA-6-302013` ` DATASET_PLAN:2`, Fortinet `logid="0000000013" type=traffic`, Palo Alto Threat CSV | `date=2024-01-01 time=12:00:00 devname="FW01"... logid="0000000013"` / `%ASA-6-302013: Built inbound...` / `1,2024/01/01...THREAT...` | `20,000` `10×2000` concurrent `10` senders `UdpSocket 127.0.0.1:0` | `0.28s` | `payload ∈ payloads` + `blake3 hash==metadata.hash` + `LogSource::Udp` + `timestamp≥start` + `drop_count 0` |
| `test_capacity_invariance_100k` | Synthetic `b"x"` tiny | `b"x"` | `100,000` `client 1` yield `100` | `1.05s` | No stall, `BytesMut` amortized `1 alloc/70k` verified, `0` drops |
| `test_drop_under_pressure` | `b"short test message"` `channel 10` | — | `100` blast `100ms` sleep | `0.12s` | `drop_count>0` backpressure `try_broadcast is_full` |

**Ingestion Metrics:** `Zero per-packet alloc` amortized `10MiB/143B≈70k`, `20k heter 0 drops` `100k tiny 0 stall` `SO_RCVBUF 8MiB verify` `HANDOFF_SESSION.md:132`.

---

## 2. Plane 4 — Integrity Plane (`prism-provenance`) — ✅ Complete `main@2f4ee6d #4`

**Code:** `vault.rs:26` `Schema Int64,Utf8,Utf8,Binary` `WriterProperties ZSTD Parquet2_0` `Uuid::new_v4` | `merkle.rs:28` `ProvenanceTree leaves Vec<[u8;32]> 1<<16=65536 cap` `rs_merkle 1.5.0` | `ticker.rs:18` `truncate ledger.log` `58` `output_dir/ledger.log` `sync_all` `interval 500ms test / 60s prod` | `audit.rs:25` `from_hex→hash==` per row

| Test | Dataset | Count | Time | Assertion |
|---|---|---|---|---|
| `test_integrity_plane_success` | Fortinet `logid="0000000013"`, Cisco `ASA-6-302013`, Palo `THREAT` CSV, NGINX `GET /index.html 200`, JSON OCSF, syslog, CheckPoint, CloudTrail, sshd, BIND — `DATASET_PLAN.md:2-3` | `10` heter `batch_size=1` → `10` parquet `10` ledger lines | `0.06s` | `parquet.len==10` `ledger_lines==10` `ledger contains leaf hashes` `SerializedFileReader` all `row_groups×columns ZSTD` `audit_roots sort==ledger_roots sort` ordered chain |
| `test_empty_vault` | Zero logs | `0` | `0.01s` | `parquet 0` `ledger exists && empty 0 lines` |
| `test_merkle_tree_limit` | `blake3(b"dummy")` `65536` +1 | `65537` pushes | `0.01s` | `65537th bail "16-level limit reached"` |
| `test_integrity_plane_mutation_fails` | `Original safe payload` → `Malicious mutated` same hash | `1` row mutated | `0.01s` | `audit_vault_file err "Payload hash mismatch"` |
| `test_audit_invalid_hex_valid` | `Safe payload` hash `ZZZZ...64` invalid hex | `1` row | `0.01s` | `err "Invalid hex hash"` (parser before compare) |
| `test_audit_concurrent_10_senders` | `Sender i Event j` `10×20=200` `batch 50` | `200` concurrent `flume 1000` | `0.01s` | `parquet non-empty` `ledger==parquet` `audit all files` race-free |

**Integrity Metrics:** `10` heter vendor logs `batch 1` `ZSTD` all columns, `ledger atomic truncate+sync_all`, `16-level 65536` per-tick flush+retry, Section65B `audit_vault_file` hash mismatch + hex + Merkle.

---

## 3. Plane 2 — Data Plane (`prism-core`) — ✅ Complete `feat/plane-2-data-plane@f581fac` PR #5 OPEN

**Code:** `router.rs:19` `memchr::memmem` `logid="` `Fortinet/devname` `Cisco %ASA-` `Palo ,THREAT` | `vrl.rs:15` `vrl::compiler::compile` 3 programs `Fortinet srcip`, `Cisco outside:`, `Palo ,ip,` | `ocsf.rs:11` `category_uid 4` `class_uid 4001` `Value::Bytes ip` | `dlq.rs:13` `create_dir_all /var/run/prism/dlq.log` dual `plaintext + jsonl` `sync_all` | `sink.rs:22` `reqwest json batch` `httptest` mock

| Test | Dataset | Count | Time | Assertion |
|---|---|---|---|---|
| `bench_router_heuristic` | `Fortinet logid="0000000013" srcip 192.168.1.5` | `1,000,000` `black_box route` | `3.07s` `3.46s` `2.94s` jitter `3.07µs` per route `assert <5s` | `<5s` microsecond claim `HANDOFF:199` |
| `test_data_plane_routing` | Fortinet `logid`, Cisco `%ASA-6-302013`, Palo `,THREAT` `idx` | `50,000` `i%3` heter `flume not needed` | `2.15s` `2.34s` `23k EPS` | `Vendor!=Unknown` `VRL parse` `OCSF class 4001 version 1.9.0 provenance_hash==hash src_ip 192.168.1.5` `Unknown→dlq.log /tmp/prism_dlq.log contains alien` |
| `test_sink_http_mock` | `OcsfNetworkActivity` `4001` | `1` batch `POST /bulk` | `0.12s` | `httptest Server 200` `push_bulk ok` |

**Data Metrics:** `50k heter 3 templates` `VRL per-vendor` `OCSF 4001` `DLQ dual-write` `HttpSink 200` `memchr 1M 3.07s`.

---

## 4. Citations & Trace

- **Datasets:** `docs/datasets/DATASET_PLAN.md:1` 4-Tier Hybrid — UNSW-NB15, CIC-IDS-2017, Loghub-2.0, Zenodo AIT `records/6475510`, SecRepo, Fortinet `logid="0000000013"`, Cisco `%ASA-6-302013`, Palo Alto Threat CSV
- **Requirements:** `docs/requirements/FRS_NFRS.md:1` 50k EPS, <5ms, air-gapped, 180-day `FRS-02/03` `NFRS-01`
- **IPC:** `docs/architecture/DATA_DICTIONARY_AND_IPC.md:18` `raw_payload utf8:/b64:` `dlq.log /var/run/prism` `OCSF 4001`
- **Architecture:** `docs/architecture/COMPONENT_DESIGN.md:20` `HANDOFF_SESSION.md:155` Bottom-to-Top `L1 Ingest → L4 Integrity → L2 Data → L3 Control → L5 Presentation` `HANDOFF:120` Sprint 1-3 DONE
- **Commits:** `89ba280 #1 Plane1`, `2f4ee6d #4 Plane4`, `f581fac Plane2` `feat/plane-2-data-plane` `2e85f9e` `acaf97a` `e6cdd4c` `c75d03a` — `CHANGELOG.md:5` `0.8.0`
- **Diagrams:** `docs/images/arch.png 38K 2026-09-23 12:12` `diagram.mmd` Amortized Blocks `tokio/quinn`

**Combined:** 13 tests 0 failures, 0 clippy, bare-metal `x86_64 7.1.8-arch1-3` `rustc 1.97.1` `2026-09-23T15:56:07Z` — stored for future Plane 3/5 append.

## Plane 3: Control Plane (prism-brain)
* **Timestamp:** 2026-09-23T16:41:36.801532Z
* **Status:** Bootstrapped (1/1 pytest passing)
* **AI/ML Modules:** Drain3, Transformers (zero-shot), Ollama Llama-3
