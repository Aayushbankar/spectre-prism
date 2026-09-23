# Project PRISM — Session Handoff & Implementation Blueprint

**Project Name:** PRISM (Programmable Routing & Intelligent Semantic Mapper)  
**Problem Statement:** SIH26156 — Universal Log Pre-processing Framework (ULPF)  
**Organization:** National Technical Research Organisation (NTRO)  
**Team:** SPECTRE  
**Repository:** `https://github.com/Aayushbankar/spectre-prism` (branch `main`)  
**Workspace Path:** `/mnt/work/projects/sih/prism`  
**Methodology:** Waterfall Requirements Architecture + Iterative Sprint Development  

---

## 1. What Was Completed in This Session

### A. Architectural & Requirements Foundations (100% Complete)
1. **Problem Statement Analysis:** Complete verbatim requirements and evaluation criteria analyzed (`docs/PS_SIH26156_VERBATIM.md`).
2. **Flagship Technical Whitepaper:** Created and mathematically grounded with 2024–2026 academic citations (`docs/PRISM_WHITEPAPER.md`).
3. **Formal FRS & NFRs:** Rigorous functional and non-functional specifications covering 50,000+ EPS throughput, <5ms latency, air-gapped constraints, and lossless raw preservation (`docs/requirements/FRS_NFRS.md`).
4. **Architectural Decision Records (ADRs):**
   * **ADR 01 Containerization Strategy:** Defined the "Dual-Deployment Flex" — bare-metal static binary for wire-speed live hackathon demo; Docker Compose provided solely for evaluator reproduction and platform independence (`docs/architecture/ADR_01_CONTAINERIZATION.md`).
   * **Cryptographic Ticker Decision:** BLAKE3 selected for SIMD wire-speed hashing with a modular trait toggle to FIPS-compliant SHA-256 for government regulatory audits.
   * **Transport Decision:** Dual Ingestion supporting legacy UDP (port 514) and next-gen QUIC (`quinn`).
5. **SIEM Integration & Dual-Dashboard Demo Strategy:** Defined the HTTP Bulk Exporter and the split-screen presentation (Ratatui TUI "Engine Room" + Kibana "Executive SIEM") (`docs/architecture/SIEM_INTEGRATION_AND_DEMO.md`).
6. **Compiled PNG Architecture Diagrams:** Headless Chromium (`mermaid-cli`) used to compile transparent architectural diagrams embedded into docs (`docs/images/arch.png`, `ai_flow.png`, `integrity.png`).
7. **Multi-Agent Evaluation by Teamwork Swarm:** Passed a zero-context 26/26 assertion victory audit with `teamwork_eval/ARCHITECTURAL_COMPARISON.md` (covering queuing theory, 1.2T FLOP metrics, and Section 65B Indian Evidence Act court admissibility) and `teamwork_eval/JURY_PRESENTATION_DECK.md` (5-slide SIH compliant pitch + technical defense appendix).
8. **Data Dictionary & IPC Contracts:** Defined file-watching IPC boundaries (`dlq.log` and `/rules/`) and strict OCSF v1.9.0 Class 4001 (Network Activity) JSON schema (`docs/architecture/DATA_DICTIONARY_AND_IPC.md`).
9. **Agent-Optimized Prompt Matrix:** 17 granular micro-tasks across 6 team member categories formatted with exact dependencies, agent instructions, and acceptance criteria (`docs/TEAM_TASK_BREAKDOWN.md`).
10. **Comprehensive Dataset Strategy:** Detailed 4-tier benchmark corpus including UNSW-NB15, CIC-IDS-2017, Loghub-2.0, Zenodo AIT, real vendor samples (Fortinet, Cisco ASA, Palo Alto), and synthetic stream replay (`docs/datasets/DATASET_PLAN.md`).
11. **Repository Hygiene:** Root `README.md` and comprehensive `.gitignore` committed and pushed.

---

## 2. System Architecture Overview (The 4 Planes)

```
                       +---------------------------------------------+
                       |              INCOMING LOGS                  |
                       |       (UDP / QUIC on Port 514)              |
                       +---------------------------------------------+
                                              |
                                              v
+------------------------------------------------------------------------------------------+
| PLANE 1: INGESTION PLANE (Rust - `prism-ingest`)                                         |
| - `tokio::net::UdpSocket` / `quinn` async listeners                                       |
| - Zero-Copy Slab Allocator (`bytes::BytesMut`)                                           |
| - Pre-parsing BLAKE3 SIMD cryptographic hashing                                          |
+------------------------------------------------------------------------------------------+
           |                                                              |
           v (Zero-copy slice + hash)                                     v (Raw bytes + hash)
+------------------------------------+             +---------------------------------------+
| PLANE 2: DATA PLANE                |             | PLANE 4: INTEGRITY PLANE              |
| (Rust - `prism-core`)              |             | (Rust - `prism-provenance`)           |
| - Fast Byte Heuristic Router       |             | - Columnar Apache Parquet batcher     |
| - Embedded VRL Engine (Datadog)    |             | - Zstandard (`zstd`) compression      |
| - OCSF 4001 Normalizer + Hash link |             | - 16-level Merkle Tree ticker         |
| - HTTP Bulk Exporter -> SIEM       |             | - Section 65B Indian Evidence ledger  |
+------------------------------------+             +---------------------------------------+
           | (On Parse Failure / Unknown)                             |
           v                                                          v
+------------------------------------+             +---------------------------------------+
| Dead Letter Queue (`dlq.log`)      |             | Tamper-Proof Cold Vault & Merkle Root |
+------------------------------------+             +---------------------------------------+
           |
           v (File Watchdog)
+------------------------------------------------------------------------------------------+
| PLANE 3: CONTROL PLANE (Python - `prism-brain`)                                          |
| - Drain3 Fixed-Depth Tree: Compresses 50k logs -> 1 template                             |
| - System 1 AI (Open Jev / DeBERTa): Fast, 0% hallucination typed classification          |
| - System 2 AI (Local Ollama / Llama-3): Generates VRL code and Router signature           |
| - Human-in-the-Loop (HitL) Gatekeeper: 1-click admin approval on TUI -> Hot Reload      |
+------------------------------------------------------------------------------------------+
                                              |
                                              v (Pushes .vrl to `/rules/`)
                                   Hot-Reloaded by Data Plane
```

---

## 3. Tech Stack Locked In

*   **Planes 1, 2, 4 & TUI:** **Rust** (Edition 2021)
    *   *Async runtime:* `tokio` (full features)
    *   *Memory:* `bytes`, `slab`, `flume` (MPSC channels)
    *   *Transformation:* `vrl` (Vector Remap Language), `serde`, `serde_json`
    *   *Integrity:* `blake3`, `parquet`, `arrow`, `rs-merkle`
    *   *Network & Exporter:* `quinn` (QUIC), `reqwest` (HTTP Bulk)
    *   *TUI:* `ratatui`, `crossterm`
*   **Plane 3 (AI Brain):** **Python 3.12**
    *   *Clustering:* `drain3`
    *   *System 1 AI:* `transformers`, `torch` (`open-jev-deberta-v3-large`)
    *   *System 2 AI:* `ollama-python` (Llama-3-8B-Instruct)
    *   *IPC Watcher:* `watchdog`
*   **Infrastructure / SIEM:**
    *   `docker-compose.yml` running `elasticsearch:8` and `kibana:8`

---

## 4. Key References & Research Files

| File Path | Description |
| :--- | :--- |
| `README.md` | Public repository overview & developer quickstart |
| `docs/PRISM_WHITEPAPER.md` | Flagship technical defense, pain point analysis, citations |
| `docs/requirements/FRS_NFRS.md` | Complete FRS & NFR specifications |
| `docs/architecture/COMPONENT_DESIGN.md` | Concrete module breakdown |
| `docs/architecture/ADR_01_CONTAINERIZATION.md` | Bare-metal vs. Docker architectural decision |
| `docs/architecture/DATA_DICTIONARY_AND_IPC.md` | IPC contracts & OCSF 4001 field mapping |
| `docs/architecture/SIEM_INTEGRATION_AND_DEMO.md` | Dual-dashboard presentation plan |
| `docs/datasets/DATASET_PLAN.md` | Benchmark datasets, vendor samples, and live replay plan |
| `docs/TEAM_TASK_BREAKDOWN.md` | Master Agent Prompt Matrix (17 micro-tasks) |
| `teamwork_eval/ARCHITECTURAL_COMPARISON.md` | Multi-agent mathematical and queuing theory evaluation |
| `teamwork_eval/JURY_PRESENTATION_DECK.md` | 5-slide core pitch + 12-slide jury defense appendix |

---

## 5. Next Session Implementation Roadmap (Start Here!)

In the new chat, transition directly from the Waterfall Design phase into **Iterative Implementation**.

### Sprint Phase 1: Workspace & Ingestion Foundation
1. **Initialize Cargo Workspace** in `/mnt/work/projects/sih/prism`:
   ```toml
   [workspace]
   members = [
       "crates/prism-common",
       "crates/prism-ingest",
       "crates/prism-core",
       "crates/prism-provenance",
       "crates/prism-tui",
   ]
   ```
2. **Build `prism-common`:** Shared data structures (`RawEvent`, `ProvenanceMeta`, `OcsfNetworkActivity`, `Blake3Hash`).
3. **Build `prism-ingest`:**
   * Tokio UDP listener bound to `0.0.0.0:514`.
   * Zero-copy buffer management using `bytes::BytesMut`.
   * Dispatch raw byte slices to high-speed `flume` channels.
4. **Build `prism-provenance`:**
   * Compute in-flight BLAKE3 hash on the `&[u8]` slice.
   * Basic batcher writing raw logs into Zstd-compressed Apache Parquet blocks.

### Sprint Phase 2: Data Plane & VRL Transformation
1. **Integrate VRL:**
   * Embed `vrl` crate into `prism-core`.
   * Write sample VRL remap scripts for Fortinet (`fortigate.vrl`) and Cisco ASA (`cisco_asa.vrl`).
   * Implement the byte-heuristic sniffer for microsecond vendor identification.
2. **OCSF Serializer & DLQ Switch:**
   * Map VRL output into verified OCSF Class 4001 JSON.
   * If parsing fails or vendor is unrecognized, route raw payload to `/var/run/prism/dlq.log`.
3. **HTTP Bulk Sink:**
   * Implement batch exporter (`reqwest`) pushing to Elasticsearch `_bulk` endpoint.

### Sprint Phase 3: Control Plane AI (`prism-brain`)
1. Create `prism-brain/` directory with `requirements.txt` (`drain3`, `transformers`, `ollama`, `watchdog`).
2. Implement `cluster.py` to watch `dlq.log` and extract templates via Drain3.
3. Implement `coder.py` connecting to local Ollama to auto-generate VRL scripts for unknown logs.
4. Test dynamic hot-reloading in `prism-core` using `notify`.

### Sprint Phase 4: Observability & Presentation
1. Build `prism-tui` using `ratatui` (Real-time EPS gauge, packet stream, DLQ counter, HitL approval modal).
2. Create `docker-compose.yml` for local Elasticsearch + Kibana sink.
3. Build the synthetic traffic generator (`tools/streamer.py`) to blast 50,000+ EPS for testing.

---

## 6. How the Next Agent Should Resume

Simply start the new chat with:
> "Read `HANDOFF_SESSION.md` in `/mnt/work/projects/sih/prism` and let's begin Phase 1: Cargo Workspace initialization and `prism-ingest` UDP listener implementation."
