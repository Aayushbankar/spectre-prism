# Team Task Breakdown (6 Members)

**Project:** PRISM (Universal Log Pre-processing Framework - SIH26156)
**Methodology:** Waterfall + Iterative Development

The workload is distributed among a 6-member team, explicitly categorized by module to prevent overlap and ensure specialized focus.

---

## 1. Core Project (Data & Ingestion Planes)
*Focus: Wire-speed network programming and exact-match parsing.*

**Member 1: Systems Engineer (Rust Data Plane)**
*   **Task 1:** Write the Tokio UDP/QUIC listener to ingest raw packets (`prism-ingest`).
*   **Task 2:** Implement the Zero-Copy Slab Allocator memory pool.
*   **Task 3:** Embed the Datadog `vrl` (Vector Remap Language) Rust crate into the mapper.
*   **Task 4:** Write the `notify` file-watcher to hot-reload new VRL scripts automatically.

## 2. Core Project (Integrity Plane)
*Focus: Cryptography, forensic traceability, and cheap cold storage.*

**Member 2: Cryptography Engineer (Rust Storage Plane)**
*   **Task 1:** Implement the `blake3` SIMD hashing algorithm on the incoming byte buffers.
*   **Task 2:** Write the Apache Parquet compression sink for the Cold Vault (`prism-provenance`).
*   **Task 3:** Build the 16-level Merkle Tree ticker to generate minute-by-minute Root Hashes for tamper evidence.
*   **Task 4:** Ensure the BLAKE3 hash and Parquet URI are appended to the Data Plane's output.

## 3. Core Project (Control Plane - AI & ML)
*Focus: Automating parser generation via offline AI.*

**Member 3: AI Engineer (Python Control Plane)**
*   **Task 1:** Build the `drain3` script to read `dlq.log` and compress raw logs into static templates.
*   **Task 2:** Implement the Open Jev (System 1 AI) via HuggingFace to deterministically classify the templates.
*   **Task 3:** Connect the Ollama (System 2 AI) Python bindings to draft VRL code based on the templates.
*   **Task 4:** Write the local rule-validation loop to ensure the drafted VRL is type-safe before human approval.

## 4. Output Part (SIEM Sink & Taxonomies)
*Focus: Connecting PRISM to the outside world and ensuring OCSF compliance.*

**Member 4: Integration Engineer (Data Routing)**
*   **Task 1:** Map the exact output from the VRL engine to the OCSF 4001 (Network Activity) JSON schema.
*   **Task 2:** Write the HTTP Bulk Exporter (`reqwest` in Rust) to batch and push the finalized JSON.
*   **Task 3:** Write the `docker-compose.yml` stack to spin up the local Elasticsearch datastore and Kibana instance.
*   **Task 4:** Ensure the DLQ routing switch gracefully handles failed parses without dropping data.

## 5. Web / Presentation Part (Dashboards)
*Focus: Real-time telemetry and executive visualization.*

**Member 5: UI/UX Engineer (Terminal & Web)**
*   **Task 1:** Build the `ratatui` (Rust) Terminal UI for the "Engine Room" display.
*   **Task 2:** Wire the Tokio `mpsc` channels to display live EPS (Events Per Second) and DLQ accumulation rates on the TUI.
*   **Task 3:** Build the "HitL (Human-in-the-Loop)" prompt on the TUI for the admin to approve AI-drafted scripts.
*   **Task 4:** Configure the Kibana/Grafana web dashboard with Geo-IP maps and pie charts to consume the Elasticsearch output.

## 6. SIH-Related Tasks (Hackathon Deliverables)
*Focus: Winning the jury pitch, compliance, and presentation.*

**Member 6: Product Owner (Pitch & Documentation)**
*   **Task 1:** Script and record the 2-Minute Demo Video demonstrating the dual-dashboard strategy.
*   **Task 2:** Build the 5-Slide Core Pitch Presentation exactly matching the PS evaluation criteria.
*   **Task 3:** Maintain and polish all Architecture Markdown documents (Whitepaper, FRS/NFRS, Technical Defense).
*   **Task 4:** Formulate the mathematical proofs for the Section 65B Indian Evidence Act compliance and 1.2T FLOP metrics for the Q&A appendix.
