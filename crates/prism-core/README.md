# `prism-core`

The `prism-core` crate is the heart of the Data Plane (Plane 2) in the PRISM architecture. It executes the critical parsing, routing, and normalization routines required to translate chaotic multi-vendor logs into a unified structured format.

## Core Mechanisms

### 1. High-Speed Heuristic Router
Instead of using slow regular expressions, the `HeuristicRouter` uses SIMD-accelerated string searching (`memchr::memmem`).
- Achieves sub-microsecond routing (<3.07µs per route).
- Instantly categorizes logs by vendor (e.g., matching `logid="` for Fortinet, `%ASA-` for Cisco, and `,THREAT` for Palo Alto).

### 2. Vector Remap Language (VRL) Engine
For logs successfully routed, `prism-core` compiles and executes Datadog's Vector Remap Language (VRL) programs.
- Each vendor has a dedicated VRL program.
- Safely extracts fields (timestamps, source IPs, actions) without unsafe memory operations.

### 3. OCSF Normalization (`OcsfMapper`)
Parsed fields are rigorously mapped into the Open Cybersecurity Schema Framework (OCSF) JSON standard.
- Enforces Category 4, Class 4001 (Network Activity).
- Ensures downstream AI and SIEM tools receive uniformly structured data.

### 4. Dual Dead Letter Queue (DLQ)
Logs that fail routing or parsing (alien logs) are safely diverted to the DLQ.
- Writes simultaneously to a plaintext `/var/run/prism/dlq.log` and a `.jsonl` file.
- Guarantees zero lost data even for entirely unknown log formats.

### 5. `HttpSink` Exporter
Successfully normalized OCSF JSON events are batched and exported via NDJSON to Elasticsearch (or any compatible `_bulk` API sink), handling network retries and error detection robustly.

## How to Test
Test the Data Plane integration and routing speed:

```bash
cargo test -p prism-core
```
This suite includes `test_data_plane_routing`, validating 50,000 logs traversing the router, VRL, and OCSF mapper correctly.
