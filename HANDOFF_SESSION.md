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
| - Amortized Zero-Copy Blocks (`chunk_size` 10MiB)                                        |
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
    *   *Memory:* `bytes`, `flume` (MPSC channels)
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

### Sprint Phase 1: Workspace & Ingestion Foundation - DONE
### Sprint Phase 2: Integrity Plane - DONE
### Sprint Phase 3: Data Plane - DONE
1. **Initialize Cargo Workspace** in `/mnt/work/projects/sih/prism`:
   ```toml
   [workspace]
   members = [
       "crates/prism-common",
       "crates/prism-ingest",
       "crates/prism-provenance",
       "crates/prism-core",
   ]
   ```
2. **Build `prism-common`:** Shared data structures (`RawEvent`, `ProvenanceMeta`, `OcsfNetworkActivity`, `Blake3Hash`).
3. **Build `prism-ingest`:**
   * Tokio UDP listener bound to `0.0.0.0:514`.
   * Zero-copy buffer management via `bytes::BytesMut` block allocation (`chunk_size: 10MiB` contract).
   * Strict `SO_RCVBUF` verification (8MiB) via `socket2`.
   * Dispatch raw byte slices to high-speed `flume` channels.

---

## 5. Development Governance & Branching Strategy

To keep `main` 100% clean and protected, development adheres to strict engineering rules:

### A. Git Branching Rules
1. **Protected `main`:** No direct development commits on `main`. `main` only receives verified, fully tested merge commits.
2. **Branch Per Plane:** Every plane is developed in isolation on its own feature branch:
   * **Plane 1 (Ingestion):** `feat/plane-1-ingestion`
   * **Plane 4 (Integrity):** `feat/plane-4-integrity`
   * **Plane 2 (Data Plane):** `feat/plane-2-data-plane`
   * **Plane 3 (Control Plane):** `feat/plane-3-control-plane`
   * **Plane 5 (Presentation):** `feat/plane-5-presentation`
3. **Correction & Refactor Rule:** Any incorrect build, regression, or design deviation requires spinning up a dedicated fix branch (e.g. `fix/plane-1-buffer-leak`) rather than patching over failures on feature branches.

### B. Strict Bottom-to-Top Module Hierarchy
Development proceeds bottom-to-top, ensuring each layer rests on a battle-tested foundation before the next is written:

```
[Layer 5] Presentation Plane (TUI + SIEM Dashboards)           ^  TOP
    ^                                                          |
[Layer 3] Control Plane (Brain: Drain3 + Open Jev + Ollama)    |
    ^                                                          |
[Layer 2] Data Plane (router/vrl/ocsf/dlq/sink)                |
    ^                                                          |
[Layer 4] Integrity Plane (BLAKE3 Hashing + Parquet Vault)     |
    ^                                                          |
[Layer 1] Ingestion Plane (UDP/QUIC Sockets + Zero-Copy Blocks) |  BOTTOM (START HERE)
```

### C. "No Mocking" Real-Life Testing Protocol
* **Zero Dummy Mocks:** Every module must be aggressively tested against actual, real-world formats (real Cisco ASA syslog strings, real Fortinet key-value lines, real Palo Alto CSVs, real network packets).
* **Test Gates Before Merging:** Each module requires:
  1. *Unit Tests:* Direct byte manipulation, boundary cases, malformed log lines.
  2. *Integration Tests:* Sending live UDP datagrams to `127.0.0.1:514` and asserting byte preservation.
  3. *Performance / Leak Tests:* Stressing the zero-copy buffer under high loop rates to verify zero memory creep.

---

## 6. Iterative Implementation Roadmap (Bottom to Top)

### Phase 1: Ingestion Plane (`feat/plane-1-ingestion`) - ✅ Complete
* **Target:** `crates/prism-ingest` & `crates/prism-common`
* **Modules:**
  1. `common`: Core types (`RawEvent`, `LogSource`, `IngestConfig`), including strict Base64/UTF-8 tagged IPC.
  2. `listener`: Amortized zero-copy buffering via `bytes::BytesMut` block allocation (`chunk_size`).
  3. `quic`: QUIC dual-ingestion stub utilizing `quinn` with self-signed rcgen certs.
  4. `dispatcher`: High-throughput TOCTOU-safe lock-free `flume` channel dispatching `RawEvent`.
* **Testing Gate:** Blasted 20,000 real raw syslog lines over UDP from 10 concurrent senders; asserted 100% packet arrival, payload verification, and rigorous backpressure testing.


### Phase 2: Integrity Plane (`feat/plane-4-integrity`) - ✅ Complete
* **Target:** `crates/prism-provenance`
* **Modules:**
  1. `hasher`: BLAKE3 SIMD in-flight hashing on the incoming `&[u8]` slice.
  2. `vault`: Apache Parquet writer with `zstd` compression batching raw payloads.
  3. `merkle`: 16-level Merkle tree generating 60-second root hashes (limit is per-tick, bounded to 65,536 leaves before forcing an immediate vault flush + ledger atomic write).
* **Testing Gate:** Feed real attack flow logs; verify Merkle root matches; intentionally mutate 1 byte in a Parquet record and assert the audit check immediately fails.

### Phase 3: Data Plane (`feat/plane-2-data-plane`) - ✅ Complete
* **Target:** `crates/prism-core`
* **Modules:**
  1. `router`: Microsecond heuristic byte-pattern vendor classifier using `memchr`.
  2. `vrl`: Datadog VRL execution engine with per-vendor `parse_regex`.
  3. `ocsf`: Class 4001 Network Activity schema serialization with provenance hash injected.
  4. `dlq`: Dead Letter Queue file sink (`/var/run/prism/dlq.log`) for unrecognized logs.
  5. `sink`: HTTP Bulk Exporter (`reqwest`) pushing to SIEM.
* **Testing Gate:** Ingest 50,000 mixed logs; verify OCSF 4001 + dlq.log verified.
### Phase 4: Control Plane (`feat/plane-3-control-plane`)
* **Target:** `prism-brain/` (Python)
* **Modules:**
  1. `watcher`: File watchdog detecting entries in `dlq.log`.
  2. `cluster`: Drain3 fixed-depth tree grouping raw logs into templates.
  3. `triage`: Open Jev (System 1) zero-shot classification for device type.
  4. `coder`: Ollama (System 2) Llama-3 generating VRL remap scripts and router signatures.
  5. `hitl`: Human-in-the-Loop review loop triggering hot-reload in `prism-core`.
* **Testing Gate:** Feed an alien log format (e.g. NGINX access log); verify Drain3 creates 1 template; verify Open Jev classifies as Web Proxy; verify Ollama produces valid VRL; verify hot-reload works without restarting Rust.

### Phase 5: Presentation & Observability (`feat/plane-5-presentation`)
* **Target:** `crates/prism-tui` & `docker-compose.yml`
* **Modules:**
  1. `tui`: Ratatui 4-pane terminal engine room (Live EPS, DLQ rate, Merkle ticker, HitL approval).
  2. `siem`: Docker Compose setup with Elasticsearch & Kibana reading PRISM's OCSF output.
* **Testing Gate:** Run full end-to-end pipeline with split-screen showing Ratatui TUI live stats and Kibana live threat maps simultaneously.

---

## 7. How to Resume in the New Chat

Start the new chat with:
> **"Read `HANDOFF_SESSION.md`. We are following strict bottom-to-top development on dedicated branches with no mocks. Checkout `feat/plane-1-ingestion` and let's begin Phase 1: Cargo Workspace setup and `prism-ingest` zero-copy UDP listener with real syslog test cases."**
