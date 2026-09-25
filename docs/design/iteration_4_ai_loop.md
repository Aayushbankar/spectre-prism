# Design Document: Iteration 4 - Autonomous AI Hot-Reloading Loop

## 1. Objective
Achieve a zero-downtime, self-healing parser loop. When PRISM encounters an unknown log format, it must route it to the Dead Letter Queue (DLQ), trigger an AI agent to write a new VRL (Vector Remap Language) parser, validate it for safety and data fidelity, and atomically hot-reload the Rust ingestion engine—all without dropping packets or restarting the server. Target: 0 to mapped in < 60s.

## 2. Python Control Plane (The Brain)
**Status:** Found existing foundations in `prism-brain/`.
*   **Watcher:** We will leverage the existing `prism-brain/watcher/watcher.py`, which correctly utilizes the `watchdog` library to monitor the DLQ file for `Alien Raw Bytes`.
*   **Triage & Coder:** `triage.py` (using `facebook/bart-large-mnli`) will classify the device type, and `coder.py` (using `Llama-3`) will generate the candidate `.vrl` script. 

## 3. Rust Engine Hot-Reloading (Zero Downtime)
**Status:** `crates/prism-core/src/vrl.rs` currently hardcodes VRL arrays at startup. 
*   **Methodology:** We will refactor `VrlEngine` to load scripts dynamically from a designated `rules/` directory.
*   **Concurrency Model:** The compiled VRL programs will be stored inside an `Arc<RwLock<HashMap<Vendor, Program>>>`. The high-throughput data plane threads will hold a `read` lock to process millions of events concurrently without locking each other.
*   **File Watching:** We will use the Rust `notify` crate to spawn a background Tokio task. When a new `.vrl` file is dropped into the `rules/` directory, the watcher will compile it in the background. Only if compilation succeeds will it briefly acquire a `write` lock to swap the new parser into memory, ensuring zero downtime and zero panics.

## 4. Gatekeeper Safety Validation (The Defense)
**Status:** `prism-brain/hitl/gatekeeper.py` currently blindly writes AI-generated code to disk. This is a critical security and stability risk.
*   **Methodology:** We cannot trust LLM-generated code blindly. We will build a dry-run interface in the Rust binary (e.g., `prism --dry-run-vrl <script.vrl> --payload <raw_log>`). 
*   **Closure Metrics:** Gatekeeper will invoke this dry-run command. The Rust engine will execute the script in a sandbox, parse the fields, and pipe the result through the `accounting.rs` module we built in Iteration 2. 
*   **The Gate:** The deployment will **only** be approved and moved to the active `rules/` directory if the Byte-Accounting `closure_ratio` is >= 90% (meaning the AI actually parsed the log completely, rather than just extracting an IP and dropping the rest).

## 5. Pros and Cons
**Pros:** 
*   `Arc<RwLock>` provides near-zero overhead for the data plane (reads are virtually free).
*   Byte-accounting closure guarantees that hallucinations cannot corrupt the pipeline.
*   Reusing `watcher.py` saves implementation time.

**Cons:** 
*   Spawning a subprocess for Gatekeeper dry-runs adds latency (~50ms), but since this happens out-of-band on the control plane and not the data plane, it is entirely acceptable.
