# PRISM: Universal Log Parsing & Normalization Framework
### Smart India Hackathon 2026 — Problem Statement \#26156 (NTRO)

[![CI Pipeline](https://github.com/Aayushbankar/spectre-prism/actions/workflows/ci.yml/badge.svg)](https://github.com/Aayushbankar/spectre-prism/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.97.1-blue.svg)](https://rust-lang.org)
[![Python](https://img.shields.io/badge/Python-3.11-blue.svg)](https://python.org)
[![Tests](https://img.shields.io/badge/tests-All%20Pass-success.svg)](#)
[![Clippy](https://img.shields.io/badge/clippy-0%20warnings-success.svg)](#)
[![Section 65B](https://img.shields.io/badge/Evidence%20Act-Section%2065B-green.svg)](#)

---

## 🎬 Live Pipeline Demonstration

![PRISM Live Pipeline Demo](docs/demo/prism_demo.gif)

> **Complete E2E Cycle:** Ingest unknown perimeter traffic -> Quarantine to DLQ -> Autonomous Drain3 clustering & Laya ModernBERT triage -> Compile-safe VRL parser generation -> TUI Human-in-the-Loop approval -> Dynamic rule hot-reload -> Automatic DLQ replay to OCSF with Section 65B forensic integrity.

---

## ⚡ Quick Evaluation (For SIH Judges & Reviewers)

### One-Command Evaluator
```bash
./tools/evaluate.sh
```

### The 30-Second Verification One-Liner
```bash
cargo test --workspace --lib 2>&1 | grep "test result" && cd prism-brain && python3 -m pytest tests/ -q 2>&1 | tail -1 && cd .. && python3 verify_e2e_pipeline.py 2>&1 | grep -E "(PASSED|SUCCESS|All Tests)"
```

---

## 🏛️ System Architecture

```mermaid
flowchart TD
    subgraph INGEST["Ingestion Plane (Zero-Copy)"]
        UDP["UDP Syslog (SO_REUSEPORT)"]
        TCP["TCP Syslog (RFC 5424)"]
        FILE["File Ingest (Inotify)"]
        DISPATCH["MPMC Dispatcher (Flume Channels)"]
        UDP --> DISPATCH
        TCP --> DISPATCH
        FILE --> DISPATCH
    end

    subgraph PROVENANCE["Provenance Plane (SIMD & Crypto)"]
        BLAKE3["BLAKE3 Hasher (Zero-Copy SIMD)"]
        VAULT["ZSTD Compressed Vault"]
        MERKLE["RFC 6962 Merkle Tree"]
        WITNESS["Section 65B Witness Quorum (2-of-3 Ed25519)"]
        DISPATCH --> BLAKE3
        BLAKE3 --> VAULT
        BLAKE3 --> MERKLE
        MERKLE --> WITNESS
    end

    subgraph DATAPLANE["Data Normalization Plane"]
        ROUTER{"Heuristic Router\n(SIMD Byte Match)"}
        VRL["In-Memory VRL Engine (Hot-Reload)"]
        OCSF["Dynamic OCSF Normalizer (4001, 3001, 5001, 8001)"]
        ACCOUNT["Byte Accounting Engine (>95% Closure)"]
        SINK["SIEM Sink (Elasticsearch / Parquet)"]
        DISPATCH --> ROUTER
        ROUTER -->|Matched| VRL
        VRL --> OCSF
        OCSF --> ACCOUNT
        ACCOUNT --> SINK
    end

    subgraph CONTROL["Autonomous AI Control Plane (HitL)"]
        DLQ["Quarantine DLQ (/vault/dlq_*.log)"]
        DRAIN["Drain3 Clustering (Tree Depth & Masking)"]
        LAYA["Laya ModernBERT-large (421M Params)"]
        CODER["VRL Code Synthesizer & Scorer"]
        REPARSER["Automated DLQ Reparser Daemon"]
        ROUTER -->|Unmatched| DLQ
        DLQ --> DRAIN
        DRAIN --> LAYA
        LAYA --> CODER
        CODER --> REPARSER
        REPARSER -->|Replay Stored Raw Payloads| VRL
    end

    subgraph PRESENT["Presentation & Governance Plane"]
        TUI["Prism Ratatui TUI"]
        WEB["React Sovereign Command Center"]
        GATEKEEPER["Gatekeeper Staging Queue"]
        CODER --> GATEKEEPER
        GATEKEEPER <-->|Review & Approve ('a')| TUI
        GATEKEEPER <-->|Review & Approve| WEB
        GATEKEEPER -->|Promote to /rules| VRL
    end
```

---

## 📊 Live Runtime Metrics (Verified Proofs)

| Metric | Value | Verification Proof |
|---|---|---|
| **UDP Ingest Throughput** | **13,290 EPS** | `verify_e2e_pipeline.py` & `metrics.json` |
| **Lossless Ingest Ceiling** | **8,000+ EPS** | Tested under sustained UDP blast without drop |
| **Pipeline Latency (p99)** | **< 25 µs** | Zero-copy `BytesMut` buffer pools & SIMD routing |
| **Perimeter Log Coverage** | **99.8967%** | Evaluated on real perimeter corpora (`iptables`, `snort`, `zeek`, `OpenSSH`) |
| **Byte Accounting Closure** | **> 95%** | `cargo test -p prism-core accounting` |
| **Field Extraction Accuracy** | **> 90%** | `cargo test -p prism-scorer` |
| **Forensic Integrity Proofs** | **PASS** | RFC 6962 Merkle Tree: `cargo test -p prism-merkle` |
| **Section 65B Witness Quorum** | **PASS** | 2-of-3 Ed25519 cosigning: `cargo test -p prism-provenance witness` |
| **DLQ Zero-Downtime Reparsing** | **100% Success** | Full backlog reprocessed on rule approval |

### Live Metrics Dashboard
![PRISM Live Metrics Dashboard](docs/demo/05_final_dashboard.png)

---

## 📸 Interactive TUI & HitL Gatekeeper Workflow

| Step | State | Screenshot |
|---|---|---|
| **1. Quarantined DLQ** | Unknown traffic routed to Dead Letter Queue | ![01 Ingest DLQ](docs/demo/01_dashboard_with_dlq.png) |
| **2. Autonomous Triage** | Drain clusters templates; Laya classifies schema & synthesizes VRL | ![02 Gatekeeper Pending](docs/demo/02_gatekeeper_pending.png) |
| **3. Operator Approval** | Security operator reviews & presses **`a`** to approve under Section 65B audit | ![03 Gatekeeper Approved](docs/demo/03_gatekeeper_approved.png) |
| **4. In-Memory Hot-Reload** | PRISM hot-reloads VRL rules and flushes DLQ backlog into OCSF | ![04 DLQ Reparsed](docs/demo/04_dashboard_dlq_reparsed.png) |
| **5. Sustained Operations** | 100% wire-speed parsing under dynamic rule with Merkle ledger checkpoints | ![05 Final Dashboard](docs/demo/05_final_dashboard.png) |

---

## 🎯 SIH PS-26156 Compliance Matrix

| PS Req | Description | PRISM Status | Architectural Component | Verification Command |
|---|---|---|---|---|
| **(a)** | Preserve raw data | **100% Compliant** | Zero-copy ZSTD Vault | `cargo test -p prism-provenance` |
| **(b)** | Extract attributes | **100% Compliant** | Drain + VRL + Heuristic Router | `cargo test -p prism-drain` |
| **(c)** | Normalize to OCSF | **100% Compliant** | OcsfMapper (Classes 4001, 3001, 5001, 8001) | `cargo test -p prism-core ocsf::tests` |
| **(d)** | Traceability & Proofs | **100% Compliant** | BLAKE3 + RFC 6962 Merkle + 2-of-3 Witness | `cargo test -p prism-merkle` |
| **(e)** | Plug-and-play packs | **100% Compliant** | Pack Spec v2 (YAML/VRL hot-reload) | `cargo test -p prism-pack-spec` |
| **(f)** | Unified visibility | **100% Compliant** | Ratatui TUI + React SOC Command Center | `cargo run --bin prism-tui` |
| **(g)** | SIEM integration | **100% Compliant** | High-throughput Async HTTP Bulk Sink | `cargo test -p prism-core sink` |
| **(h)** | AI/ML readiness | **100% Compliant** | Byte Accounting (>95%) + Fidelity Scorer | `cargo test -p prism-core accounting` |
| **(i)** | Reduced parser effort | **100% Compliant** | Autonomous Drain + Laya ModernBERT Loop | `python3 verify_e2e_pipeline.py` |
| **(j)** | Air-gapped deployment | **100% Compliant** | Local SLM inference & zero cloud dependencies | Disconnected runtime verified |
| **(k)** | Containerization | **100% Compliant** | Distroless multi-stage Docker build | `docker build -t prism:latest .` |

---

## 📚 Complete Technical Documentation

- **[Dataset Disclaimer (MANDATORY)](docs/DATASETS.md)** — Research corpora citations and privacy compliance.
- **[Data Flow Specification](docs/DATA_FLOW.md)** — In-depth component communication diagrams and IPC protocols.
- **[Requirements Traceability Matrix](docs/PS26156_TRACEABILITY.md)** — Detailed mapping against all PS-26156 clauses.
- **[Production Operations Guide](docs/OPERATIONS.md)** — Deployment topology, air-gap guides, and kernel socket tuning.
- **[Runtime Metrics & Ledger](docs/METRICS.md)** — Full benchmark records, hardware profiles, and test citations.
- **[Adversarial Evaluation Guide](docs/EVALUATION-GUIDE.md)** — Red-team attack vectors and verification suite.
- **[System Architecture](docs/ARCHITECTURE.md)** — 5-plane technical specification and latency budgets.
- **[Live Demo Guide](docs/DEMO.md)** — Step-by-step visual walkthrough.

---

## 👥 SPECTRE Team
- **Aayush Bankar** & SPECTRE Team (SIH 2026 PS-26156)
