# Team Task Breakdown & AI Prompt Matrix (6 Members)

**Project:** PRISM (SIH26156)
**Methodology:** Waterfall + Iterative Development

> **⚠️ NOTE TO TEAM SPECTRE & AI AGENTS:** 
> This document is structured as an **Agent-Optimized Prompt Matrix**. Because our team is utilizing AI coding assistants (Cursor, Copilot, Antigravity), tasks are broken down into precise micro-tasks. 
> *Workflow:* Copy the **"Agent Instructions"** block for your assigned task and paste it directly into your AI assistant along with the target file path.

---

## 1. Core Project (Data & Ingestion Plane)
**Owner:** Systems Engineer (Rust)
**Crates/Dependencies:** `tokio`, `bytes`, `memchr`, `vrl`, `reqwest`, `httptest`

### Micro-Task 1.1: Zero-Copy Network Listener
*   **Target File:** `prism-core/src/ingest/listener.rs`
*   **Agent Instructions:** Implement an asynchronous `tokio::net::UdpSocket` bound to port 514. Do not allocate new `String` or `Vec<u8>` for every packet. Instead, implement a zero-copy memory pool using amortized `bytes::BytesMut` blocks. When a packet arrives, yield a read-only slice (`&[u8]`) and pass it to a high-throughput `flume` channel. Use `memchr` for heuristic scanning.
*   **Acceptance Criteria:** Zero heap allocations per packet after initialization. Benchmarked at >50,000 EPS.

### Micro-Task 1.2: VRL Execution Engine
*   **Target File:** `prism-core/src/map/vrl_engine.rs`
*   **Agent Instructions:** Embed Datadog's `vrl` crate. Write a function that accepts a raw log string and a pre-compiled VRL AST program. Execute the VRL program safely to extract fields and return a `serde_json::Value`. Ensure abort-safety (no panics on malformed logs, return a `Result::Err` mapped to the DLQ).
*   **Acceptance Criteria:** Successfully executes a `fortinet.vrl` script against a mock log string without panicking.

### Micro-Task 1.3: Dynamic Router & Hot-Reload
*   **Target File:** `prism-core/src/router/hot_reload.rs`
*   **Agent Instructions:** Implement the `notify` crate to watch the `/etc/prism/rules/` directory for new `.vrl` files. Upon a file creation event, asynchronously compile the new VRL script into an AST and update a `parking_lot::RwLock` registry mapping vendor signatures to the AST.
*   **Acceptance Criteria:** Modifying or adding a `.vrl` file updates the parsing logic live without restarting the Rust binary.

---

## 2. Core Project (Integrity & Storage Plane)
**Owner:** Cryptography Engineer (Rust)
**Crates/Dependencies:** `blake3`, `parquet`, `arrow`, `zstd`, `rs-merkle`

### Micro-Task 2.1: In-Flight Hasher
*   **Target File:** `prism-core/src/integrity/hasher.rs`
*   **Agent Instructions:** Write a highly optimized function that takes the `&[u8]` slice from the UDP listener and computes a `blake3` hash. Return a tuple of `(&[u8], String)` representing the raw data and its hex hash. Use conditional compilation (`#[cfg]`) to allow swapping to `sha2` for FIPS compliance.
*   **Acceptance Criteria:** Hash computation takes less than 50 microseconds per payload.

### Micro-Task 2.2: Parquet Cold Vault
*   **Target File:** `prism-core/src/integrity/vault.rs`
*   **Agent Instructions:** Write an asynchronous batcher that collects the raw `&[u8]` payloads and their hashes. Once the batch hits 10,000 items (or 60 seconds elapses), write the batch to disk as an Apache Parquet file using the `parquet` and `arrow` crates. Enable `zstd` compression on the Parquet writer.
*   **Acceptance Criteria:** Parquet files are generated correctly and can be read back using a standard Python dataframe.

### Micro-Task 2.3: Merkle Tree Ticker
*   **Target File:** `prism-core/src/integrity/merkle.rs`
*   **Agent Instructions:** Implement a 16-level Merkle tree using `rs-merkle`. Every 60 seconds, take all the BLAKE3 hashes generated in that window, compute the Merkle Root, and append it to a tamper-evident `root_ledger.json` file.
*   **Acceptance Criteria:** Changing a single byte in an old Parquet log causes the Merkle Root validation to fail.

---

## 3. Core Project (Control Plane / AI)
**Owner:** AI Engineer (Python)
**Dependencies:** `drain3`, `transformers`, `ollama-python`, `watchdog`

### Micro-Task 3.1: DLQ Compressor (Drain3)
*   **Target File:** `prism-brain/cluster.py`
*   **Agent Instructions:** Write a Python daemon using `watchdog` to monitor `dlq.log` (JSONL format). Extract the raw payloads and feed them into the `drain3` clustering algorithm. Maintain a persistent state tree. When a new cluster template is formed, emit the template string to the Triage module.
*   **Acceptance Criteria:** Successfully compresses 50,000 raw identical logs into 1 template string.

### Micro-Task 3.2: System 1 Triage (Open Jev)
*   **Target File:** `prism-brain/triage.py`
*   **Agent Instructions:** Load the `open-jev-deberta-v3-large` model via HuggingFace `transformers`. Write a function that takes a Drain3 template and runs a zero-shot classification to categorize the device type (e.g., Firewall, Web, OS). Return the classification and the confidence score.
*   **Acceptance Criteria:** Execution takes <500ms on CPU and hallucinates 0% of the time.

### Micro-Task 3.3: System 2 Coder (Ollama)
*   **Target File:** `prism-brain/coder.py`
*   **Agent Instructions:** Use the `ollama` Python library. Construct a strict system prompt containing the OCSF schema rules. Pass the Drain3 template and Open Jev classification to a local `llama3` model to generate a Vector Remap Language (.vrl) script. Write the output to a `pending/` directory for HitL approval.
*   **Acceptance Criteria:** Outputs syntactically valid VRL code without markdown formatting blocks.

---

## 4. Output Part (SIEM Integration)
**Owner:** Integration Engineer
**Dependencies:** `reqwest`, `serde_json`, Docker

### Micro-Task 4.1: OCSF Struct Definition
*   **Target File:** `prism-core/src/sink/ocsf.rs`
*   **Agent Instructions:** Define Rust `struct` definitions mapping exactly to OCSF v1.9.0 Class 4001 (Network Activity). Implement `Serialize` via Serde. Ensure fields like `activity_id` and `metadata.provenance_hash` are present.
*   **Acceptance Criteria:** Successfully serializes to JSON matching the official OCSF standard.

### Micro-Task 4.2: HTTP Bulk Exporter
*   **Target File:** `prism-core/src/sink/exporter.rs`
*   **Agent Instructions:** Write an async `reqwest` client. Read finalized OCSF JSON objects from a channel, batch them into arrays of 500, and send them via HTTP POST to a configurable Elasticsearch `_bulk` API endpoint. Implement exponential backoff for retries.
*   **Acceptance Criteria:** Handles Elasticsearch backpressure (HTTP 429) without dropping logs.

### Micro-Task 4.3: Infrastructure Stack
*   **Target File:** `docker-compose.yml`
*   **Agent Instructions:** Write a compose file to spin up Elasticsearch 8.x (single-node) and Kibana 8.x. Include environment variables to disable X-Pack security for local testing ease.
*   **Acceptance Criteria:** Running `docker compose up` results in a healthy Kibana dashboard on port 5601.

---

## 5. Web Part (Dashboards & TUI)
**Owner:** UI/UX Engineer
**Dependencies:** `ratatui`, `crossterm`

### Micro-Task 5.1: TUI Engine Room
*   **Target File:** `prism-tui/src/ui.rs`
*   **Agent Instructions:** Use `ratatui` to build a 4-pane terminal interface: 1) Top-left: Live EPS Gauge, 2) Bottom-left: Real-time UDP packet hex dump, 3) Top-right: DLQ queue size bar chart, 4) Bottom-right: HitL Approval Log.
*   **Acceptance Criteria:** Renders cleanly at 60fps without flickering on standard Linux terminals.

### Micro-Task 5.2: HitL (Human-in-the-Loop) Intercept
*   **Target File:** `prism-tui/src/hitl.rs`
*   **Agent Instructions:** Implement a keyboard event listener (`crossterm::event`). When a file appears in the `pending/` rules directory, pause the TUI and display the drafted VRL code. Wait for the user to press `[Y]` to approve or `[N]` to reject. If approved, move the file to `/etc/prism/rules/`.
*   **Acceptance Criteria:** Safely intercepts and moves files based on keyboard input.

---

## 6. SIH Tasks (Hackathon Deliverables)
**Owner:** Product Owner / Analyst

### Micro-Task 6.1: Video Scripting
*   **Target File:** `docs/sih_materials/VIDEO_SCRIPT.md`
*   **Human Instructions:** Write a tight 120-second script. 0-20s: The legacy SIEM problem. 20-60s: Visual explanation of the 4-plane architecture. 60-120s: Screen recording showing the Ratatui TUI processing logs at Gbps speeds side-by-side with Kibana ingesting the OCSF data.

### Micro-Task 6.2: Pitch Deck Extraction
*   **Target File:** `docs/sih_materials/PITCH.pptx`
*   **Human Instructions:** Convert the 5-Slide core pitch mapped out in `teamwork_eval/JURY_PRESENTATION_DECK.md` into high-fidelity slides. Ensure Slide 4 explicitly calls out the Section 65B Indian Evidence Act compliance.

### Micro-Task 6.3: Q&A Defense Prep
*   **Target File:** `docs/sih_materials/QA_DEFENSE.md`
*   **Human Instructions:** Memorize the math for the 1.2T FLOP metrics and the Elasticsearch ILM cost savings created by the teamwork agents. Practice responding to the question: "Why didn't you put the AI in the inline hot-path?" (Answer: The Latency-Cost Trilemma).
