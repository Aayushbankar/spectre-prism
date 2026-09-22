# Functional & Non-Functional Requirements Specification (FRS/NFRS)

**Project:** PRISM (Programmable Routing & Intelligent Semantic Mapper)
**Problem Statement:** SIH26156 (NTRO - Universal Log Pre-processing Framework)

---

## 1. Functional Requirements Specification (FRS)

| Req ID | Requirement Description | Priority | Evaluation Criteria |
| :--- | :--- | :--- | :--- |
| **FRS-01** | **Universal Ingestion:** System must ingest raw telemetry (Syslog RFC 3164/5424, CEF, CSV) over UDP/TCP streams without dropping packets. | High | Ingestion loop binds to port 514 and captures raw string lines without regex blocking. |
| **FRS-02** | **Zero-Loss Storage (Integrity Vault):** System must store the exact raw byte stream of an ingested log before any parsing or truncation occurs. | High | Every raw log is written to an immutable disk vault (local or object store) alongside its cryptographic hash. |
| **FRS-03** | **Cryptographic Provenance:** System must compute a SHA-256 hash for every ingested log line to ensure forensic tamper-evidence. | High | Hash matches original payload. Root hashes are committed to a Merkle tree periodically. |
| **FRS-04** | **Semantic Mapping (Data Plane):** System must deterministically map incoming logs to the OCSF (Open Cybersecurity Schema Framework) JSON schema. | High | Raw strings are converted to valid `category_uid: 4` (Network Activity) JSON objects. |
| **FRS-05** | **Dead Letter Queue (DLQ):** Unrecognized logs (no matching parser) must be routed to a DLQ without crashing the pipeline. | High | Malformed/unknown logs are safely queued in `dlq.log` rather than being dropped. |
| **FRS-06** | **AI-Driven Parser Generation (Control Plane):** System must autonomously analyze the DLQ, cluster unknown formats, and draft new parsing rules (VRL). | Medium | System outputs a syntactically valid VRL script mapped to OCSF for the unknown format. |
| **FRS-07** | **Human-in-the-Loop (HitL) Gate:** Generated parsers must not execute until an administrator reviews and approves the code. | High | A TUI/CLI prompt allows admin to type 'Y' to commit the new parser to the Data Plane. |
| **FRS-08** | **Live Telemetry Dashboard:** Provide a Real-time Terminal UI showing EPS (Events Per Second), DLQ size, and parsing success rates. | High | Ratatui TUI refreshes at 10Hz displaying accurate pipeline metrics. |

---

## 2. Non-Functional Requirements Specification (NFRS)

| Req ID | Requirement Description | Priority | Metrics / Standards |
| :--- | :--- | :--- | :--- |
| **NFRS-01** | **Billion-Event Scale (Throughput):** System must support parsing at extreme network speeds without CPU saturation. | High | **Target:** > 50,000 EPS per CPU core. **Constraint:** Pipeline avoids inline regex compilation and memory reallocations. |
| **NFRS-02** | **Air-Gapped Deployment:** Entire framework (including AI Control Plane) must function without internet access. | High | **Constraint:** No cloud APIs (No OpenAI/AWS). AI must run via local quantized SLMs (Ollama/Llama3). |
| **NFRS-03** | **Low Memory Footprint (Zero-Copy):** Pipeline must utilize zero-copy memory patterns to minimize RAM usage. | High | **Target:** Core Rust ingestion process remains under 50MB RSS during peak load. |
| **NFRS-04** | **Data Traceability (CERT-In 180-Day Rule):** Output logs must natively reference their raw origin to support 180-day forensic audits. | High | OCSF output contains `_batch_uri` and offset linking directly to the compressed raw vault. |
| **NFRS-05** | **Extensibility:** The parser engine must be loosely coupled, allowing new rules to be hot-reloaded without pipeline restart. | Medium | Modifying a VRL config file reloads the parsing engine seamlessly within 500ms. |
