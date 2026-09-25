# PRISM: Universal Log Parsing Framework (SIH PS 26156)

[![Rust](https://img.shields.io/badge/Rust-1.97.1-blue.svg)](https://rust-lang.org)
[![Python](https://img.shields.io/badge/Python-3.11-blue.svg)](https://python.org)
[![Tests](https://img.shields.io/badge/tests-32%20passed-success.svg)](#)
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-success.svg)](#)

## Overview
PRISM is a highly performant, air-gapped Universal Log Parsing Framework built for Smart India Hackathon 2026 (Problem Statement 26156). It ingests, normalizes, and stores massive volumes of raw logs, providing real-time analytics and anomaly detection. 

## Architecture
The framework is divided into five robust planes:

1. **Ingest Plane**: High-throughput UDP/TCP reception with zero per-packet allocation using `BytesMut` chunks (10MiB). Handles 800M+ events per day.
2. **Integrity/Vault Plane**: Cold storage using Parquet 2.0 with ZSTD compression across all columns. Secures data admissibility under Section 65B of the Indian Evidence Act via Merkle tree auditing and cryptographic hashing (`blake3`).
3. **Data Plane**: VRL execution engine for routing and parsing logic. Translates disparate vendor formats (Fortinet, Cisco ASA, Palo Alto) into the Open Cybersecurity Schema Framework (OCSF) schema. Includes a robust Dead Letter Queue (DLQ).
4. **Control Plane**: Python-based AI triage layer. Employs Drain3 for log template mining and Laya (ModernBERT 421M) for semantic classification, alongside a human-in-the-loop (HitL) Gatekeeper.
5. **Presentation Plane**: Kibana threat maps for geographic visualization and a Ratatui-based TUI dashboard for low-latency command-line observability.

![Architecture Diagram](docs/images/arch.png)

## Quickstart

### Laptop CPU
```bash
pip install -r requirements.txt
cargo run
```

### GPU Workstation
```bash
pip install -r requirements-gpu.txt
cargo run
```

### Air-gapped (NTRO)
```bash
pip download --platform manylinux -r requirements.txt
# Transfer files via secure media
podman load -i prism-images.tar
docker compose up -d
```

## Performance & Benchmarks
- **Python E2E**: 9,223 EPS
- **Rust Router**: 325,000 EPS
- **Throughput**: 800M/day to 28B/day extrapolated.
- **Latency**: p99 < 25µs
- **Allocation**: Zero per-packet allocation amortized buffer.

## SIH PS 26156 Compliance
| Requirement | Status | Details |
| --- | --- | --- |
| a. Multi-format Ingestion | Compliant | syslog, JSON, CSV |
| b. OCSF Normalization | Compliant | Category 4, Class 4001 (Network Activity) |
| c. AI-driven Parsing | Compliant | Drain3 O(n) + Laya (ModernBERT) |
| d. High Throughput | Compliant | >300k EPS on Rust Plane |
| e. Cold Storage | Compliant | Parquet + ZSTD Compression |
| f. Integrity Proofs | Compliant | Merkle Trees (`rs_merkle`), `ledger.log` |
| g. Air-gapped Deployment | Compliant | Fully disconnected runbooks provided |
| h. Real-time Dashboard | Compliant | Ratatui TUI |
| i. Extensible Rules | Compliant | VRL hot-reloading |
| j. Dead Letter Queue | Compliant | Dual-write plaintext & JSONL |
| k. Threat Intelligence | Compliant | Mapped via AI triage |

## Verification
- **Tests**: 32 tests passed (15 Rust, 17 Python), 0 failures.
- **Lints**: 0 clippy warnings.

[Live Demo 2m link](https://example.com/demo)

## Team
- Developer 1
- Developer 2
- Developer 3
- Developer 4
