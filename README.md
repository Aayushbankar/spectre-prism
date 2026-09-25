# PRISM: Universal Log Parsing Framework (SIH PS 26156)

[![Rust](https://img.shields.io/badge/Rust-1.97.1-blue.svg)](https://rust-lang.org)
[![Python](https://img.shields.io/badge/Python-3.11-blue.svg)](https://python.org)
[![Tests](https://img.shields.io/badge/tests-32%20passed-success.svg)](#)
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-success.svg)](#)

## Overview
PRISM is a highly performant, air-gapped Universal Log Parsing Framework built for Smart India Hackathon 2026 (Problem Statement 26156). It ingests, normalizes, and stores massive volumes of raw logs, providing real-time analytics and anomaly detection.

## Architecture
The framework is divided into five planes:
1. **Ingest Plane**: High-throughput UDP/TCP reception with zero per-packet allocation.
2. **Integrity/Vault Plane**: Cold storage via Parquet 2.0 with ZSTD compression and Merkle tree auditing.
3. **Data Plane**: VRL execution engine for OCSF schema normalization.
4. **Control Plane**: Python-based AI triage using Drain3 and Laya (ModernBERT).
5. **Presentation Plane**: Kibana threat maps and Ratatui TUI dashboard.

![Architecture Diagram](diagram.mmd)

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
# Transfer files
podman load -i prism-images.tar
docker compose up -d
```

## Performance & Benchmarks
- **Python E2E**: 9,223 EPS
- **Rust Router**: 325,000 EPS
- **Throughput**: 800M/day to 28B/day
- **Latency**: p99 < 25µs
- **Allocation**: Zero per-packet allocation amortized buffer.

## SIH PS 26156 Compliance
| Requirement | Status |
| --- | --- |
| a. Multi-format Ingestion | Compliant (syslog, JSON, CSV) |
| b. OCSF Normalization | Compliant |
| c. AI-driven Parsing | Compliant (Drain3 + Laya) |
| d. High Throughput | Compliant (>300k EPS) |
| e. Cold Storage | Compliant (Parquet + ZSTD) |
| f. Integrity Proofs | Compliant (Merkle Trees) |
| g. Air-gapped Deployment | Compliant |
| h. Real-time Dashboard | Compliant (Ratatui TUI) |
| i. Extensible Rules | Compliant (VRL) |
| j. Dead Letter Queue | Compliant |
| k. Threat Intelligence | Compliant |

## Verification
- **Tests**: 32 tests passed (15 Rust, 17 Python), 0 failures.
- **Lints**: 0 clippy warnings.

[Live Demo 2m link](https://example.com/demo)

## Team
- Developer 1
- Developer 2
- Developer 3
- Developer 4
