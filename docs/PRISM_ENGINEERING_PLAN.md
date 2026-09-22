# PRISM Codebase Engineering Plan

**Project:** Programmable Routing & Intelligent Semantic Mapper (PRISM)
**Goal:** Transform the existing `spectre` EDR codebase into the PRISM Log Pre-processor.

## 1. Component Migration Strategy

| Legacy `spectre` Component | PRISM Implementation | Why it changes |
| :--- | :--- | :--- |
| **`main.rs` (eBPF hooks)** | **UDP/TCP Ingestion Loop** | PRISM is a network log pipeline, not a kernel process monitor. We will rip out `aya` and use `tokio::net::UdpSocket` listening on port 514 (Syslog). |
| **`tui.rs` (Process Trees)** | **Telemetry & AI Dashboard** | Re-purpose Ratatui to show Live EPS, Dead Letter Queue (DLQ) size, and a live stream of the AI generating VRL mapping rules. |
| **`pipeline.rs` (Stats)** | **Throughput Metrics** | Keep the metrics engine but track `events_ingested`, `bytes_processed`, `parse_failures`, and `ocsf_emitted`. |
| **`spectre-rules` (Sigma)** | **VRL Engine (`spectre-map`)** | Replace Sigma threat-hunting rules with Vector Remap Language (VRL) parsing rules. |
| **`spectre-graph` (Lineage)** | **Integrity Vault (`spectre-provenance`)** | Replace process ancestry with Cryptographic Provenance. Hash each raw log line with SHA-256 (Merkle trees) before parsing. |

---

## 2. Step-by-Step Execution Plan

### Step 1: Initialize Project PRISM
Create the foundational Rust structure, cherry-picking the necessary folders from the old EDR.
```bash
cargo new --bin prism-agent
cd prism-agent
# (We will copy tui.rs and pipeline.rs from spectre-agent/src/)
```

### Step 2: Update Dependencies (`Cargo.toml`)
Remove all EDR crates (`aya`, `petgraph`) and introduce parsing/cryptography crates:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
ratatui = "0.29"
crossterm = "0.28"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
vrl = "0.19"        # Vector Remap Language for Data Plane
sha2 = "0.10"       # Cryptography for Integrity Plane
hex = "0.4"
```

### Step 3: Write the Data Plane (UDP Ingestion & Integrity)
In `src/main.rs`, establish the high-speed Tokio UDP listener.
*   **Action 1:** Bind to `0.0.0.0:514`
*   **Action 2:** Read raw byte buffer.
*   **Action 3:** Immediately hash the buffer with `sha2` (The Integrity Plane).
*   **Action 4:** Pass to the VRL mapper.

### Step 4: Write the Control Plane (DLQ & AI Onboarding)
When the Data Plane fails to parse a log (because the VRL rule doesn't exist yet), route the raw log to a `DLQ.log` file.
*   *Note: In the hackathon demo, a separate Python script (`brain.py`) will run Drain3 clustering on this DLQ and prompt local Ollama to draft the new VRL config.*

### Step 5: Refactor the TUI
Modify the existing Ratatui dashboard. 
*   **Top:** Throughput (EPS) and System Memory (proving the Zero-Copy efficiency).
*   **Left Pane:** The Dead Letter Queue (showing unrecognized logs piling up).
*   **Right Pane:** The AI Console (showing the SLM drafting new VRL parsers in real-time).
*   **Bottom Pane:** The Integrity Stream (streaming the SHA-256 hashes of ingested logs).
