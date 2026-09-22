# System Component Architecture

**Project:** PRISM (Programmable Routing & Intelligent Semantic Mapper)

This document details the internal module structure required to fulfill the 3-plane architecture design.

## 1. High-Level Workflow
1. **Ingestion:** Raw network packet arrives -> `prism-ingest`.
2. **Provenance:** Packet is hashed -> `prism-integrity`.
3. **Routing:** Packet is checked against known schemas -> `prism-map`.
   - *Match:* Converted to OCSF -> output to sink.
   - *Miss:* Sent to Dead Letter Queue (DLQ).
4. **Learning (Async):** `prism-brain` reads DLQ -> clusters -> drafts new parser -> requests admin approval.
5. **Observability:** `prism-tui` queries metrics -> renders dashboard.

---

## 2. Component Breakdown

### A. Data Plane (`prism-core`)
**Language:** Rust
*   **`ingest.rs`:** 
    *   Listens on UDP/TCP (Port 514).
    *   Implements zero-copy byte buffering.
*   **`mapper.rs`:**
    *   Embeds the Vector Remap Language (VRL) execution engine.
    *   Caches compiled VRL programs.
    *   Takes `&str` (raw log), executes VRL program, returns `serde_json::Value` (OCSF output).
*   **`router.rs`:**
    *   Heuristics engine (determines if log is Fortinet, Cisco, etc., by checking first few bytes).
    *   Handles the fallback routing to DLQ when heuristic fails.

### B. Integrity Plane (`prism-provenance`)
**Language:** Rust
*   **`hasher.rs`:**
    *   Implements `sha2::Sha256`.
    *   Calculates hash of the raw byte slice.
*   **`vault.rs`:**
    *   Writes raw strings + hashes to immutable compressed Parquet or Zstd logs.
    *   Maintains the Merkle Tree root.

### C. Control Plane (`prism-brain`)
**Language:** Python (for AI/ML libraries)
*   **`cluster.py`:**
    *   Implements **Drain3** (Fixed-depth tree log clustering).
    *   Reads `dlq.log`, extracts the static templates (e.g., `<IP> failed login at <TIME>`), and masks variables.
*   **`infer.py`:**
    *   Uses `ollama` Python bindings.
    *   Loads local SLM (Llama-3-8B-Instruct).
    *   Prompts the SLM with the clustered template and the OCSF schema definition, asking for a VRL script.
*   **`gatekeeper.py`:**
    *   Waits for user approval of the LLM-generated VRL script. If approved, writes to the `rules/` directory, triggering a hot-reload in `prism-core`.

### D. Presentation Plane (`prism-tui`)
**Language:** Rust (Ratatui)
*   **`ui.rs`:**
    *   Renders the 4-pane terminal dashboard.
    *   Communicates with `prism-core` via Tokio `mpsc` channels to fetch live EPS (Events Per Second) and DLQ accumulation rates.
