# PRISM (SIH26156) - Master Development & Victory Plan

This document serves as the master blueprint for developing PRISM into a winning submission for the NTRO Smart India Hackathon (SIH26156) problem statement. It tracks what has been completed, what is currently in progress, and the exact steps required to finish the project.

## 🎯 The Ultimate Goal
To build a highly performant, air-gapped, Rust-based Universal Log Pre-processing Framework that ingests massive volumes of raw logs, intelligently normalizes them to OCSF, proves zero data loss via cryptographic accounting, and requires zero human intervention for new log formats.

---

## ✅ Iteration 1: Foundation & Truth (COMPLETED)
**Goal:** Establish a baseline of truth by auditing the initial prototype and identifying architectural gaps.
* **[x] Task 1.1:** Full repository audit (Buildability, Claims vs. Reality, Test Integrity).
* **[x] Task 1.2:** Identified fatal flaws: No runnable binary, mocked AI, hardcoded OCSF fields, synthetic data.
* **[x] Task 1.3:** Generated `AUDIT_REPORT.md` to track technical debt and prioritize remediation.

---

## ✅ Iteration 2: Structural Integrity & Differentiators (COMPLETED)
**Goal:** Build the core Rust engine and implement the high-value features that separate PRISM from standard log forwarders.
* **[x] Task 2.1 (The Binary):** Built `prism-bin` (`main.rs`) to wire UDP ingestion to the dispatcher and data plane.
* **[x] Task 2.2 (Dynamic Parsing):** Rewrote VRL scripts to natively extract multiple fields for Cisco, Fortinet, and Palo Alto.
* **[x] Task 2.3 (OCSF Mapping):** Mapped extracted VRL fields dynamically into the OCSF schema, eliminating hardcoded constants.
* **[x] Task 2.4 (Byte-Accounting):** Implemented `accounting.rs` to tag every byte (`Field`, `Literal`, `Residue`) to mathematically prove zero data loss.
* **[x] Task 2.5 (Witness Cosigning):** Implemented 2-of-3 `ed25519` cryptographic signing in `witness.rs` to make the vault tamper-proof.
* **[x] Task 2.6 (Real Data):** Replaced synthetic IP generators with real vendor log samples in `data/samples/`.

---

## 🏃 Iteration 3: Ingestion Completeness (NEXT)
**Goal:** Fulfill the "Universal" ingestion requirement by supporting enterprise-grade protocols beyond UDP.
* **[ ] Task 3.1 (TCP Listener):** Implement a `TcpListener` in `crates/prism-ingest/src/listener.rs`. Must handle connection pooling, stream framing (newline delimited), and backpressure.
* **[ ] Task 3.2 (File Tailer):** Implement a file-tailing module that reads actively appended log files (e.g., `/var/log/syslog`) and streams lines to the dispatcher.
* **[ ] Task 3.3 (Unified Ingestion config):** Update `main.rs` to allow the user to specify multiple ingestion endpoints via CLI or a `.toml` config file (e.g., listen on UDP 514, TCP 514, and tail `/var/log/auth.log` simultaneously).

---

## 🏃 Iteration 4: The Autonomous AI Loop (The Killer Demo)
**Goal:** Deliver on the "intelligent" promise by making the system write its own parsing rules for unknown logs without restarting.
* **[ ] Task 4.1 (DLQ Watcher):** Write a Rust thread (or refine the Python `watcher.py`) that monitors the Dead Letter Queue for recurring unknown log formats (using the existing `Drain3` clustering).
* **[ ] Task 4.2 (Air-gapped LLM Integration):** Integrate a local, lightweight GGUF model (e.g., Llama 3 8B Instruct) via `llama.cpp` to analyze the clustered unknown log and draft a VRL script mapping it to OCSF.
* **[ ] Task 4.3 (Hot-Reloading):** Enhance `VrlEngine` to watch a rules directory. When the AI writes a new `.vrl` file, Rust automatically compiles it and adds it to the routing table without dropping a single packet.
* **[ ] Task 4.4 (The E2E Test):** Create an integration test where an unknown log is sent, fails to parse, the AI writes the rule, and 5 seconds later, the exact same log is successfully parsed.

---

## 🏃 Iteration 5: "Show, Don't Tell" (Dashboards & Visualization)
**Goal:** Give hackathon judges visual proof of scale and success.
* **[ ] Task 5.1 (Elasticsearch Sink):** Finalize the `HttpSink` to robustly push OCSF JSON to a live Elasticsearch cluster. Implement bulk retries and error handling.
* **[ ] Task 5.2 (Dockerized UI Stack):** Create a `docker-compose.yml` that boots Elasticsearch and Kibana with pre-configured indices.
* **[ ] Task 5.3 (The Winning Dashboard):** Design and export a Kibana dashboard showing:
    * Live EPS (Events Per Second).
    * Vendor breakdown pie charts.
    * Byte-accounting closure gauge (Target: >95%).
    * Threat maps based on OCSF `src_endpoint.ip`.
* **[ ] Task 5.4 (TUI Polish):** Polish the `prism-tui` Ratatui terminal interface for a high-tech "hacker" visual during the live pitch.

---

## 🏃 Iteration 6: Stress Testing & Air-Gap Proof
**Goal:** Generate the undeniable numbers required for the presentation.
* **[ ] Task 6.1 (Load Generator):** Build a Rust script (`tools/load_gen`) that blasts millions of real logs into the TCP/UDP ports as fast as possible.
* **[ ] Task 6.2 (Benchmarking):** Run the load generator and document the maximum EPS achieved on a standard laptop CPU. Record CPU/Memory utilization using `htop`.
* **[ ] Task 6.3 (Air-Gap Validation):** Write a strict test script that disables all external networking on the host machine and proves the entire pipeline (including the AI loop) still works flawlessly.

---

## 🏆 Iteration 7: Presentation & Pitch Polish
**Goal:** Translate engineering success into hackathon points.
* **[ ] Task 7.1 (Pitch Deck Update):** Update the presentation deck with real EPS numbers, screenshots of the dashboard, and diagrams of the Witness Cosigning architecture.
* **[ ] Task 7.2 (Live Demo Script):** Write a minute-by-minute script for the live demo:
    1. Start PRISM.
    2. Send 50,000 logs (Show speed).
    3. Open Dashboard (Show visualization).
    4. Send an unknown log (Show AI Auto-onboarding).
    5. Show the Vault (Prove Witness signatures).
* **[ ] Task 7.3 (Jury Q&A Prep):** Prepare technical defenses for expected judge questions (e.g., "How do you handle regex ReDoS?", "What happens if the vault disk fills up?").

---
*Generated by Antigravity Autonomous Agent.*
