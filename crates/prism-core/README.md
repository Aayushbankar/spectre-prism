# `prism-core`

The `prism-core` crate forms the Data Plane of the PRISM framework, handling routing, normalization, and export.

## Key Features

- **`HeuristicRouter`**: SIMD-accelerated log routing using `memchr`, achieving under 3.07µs per log.
- **Vector Remap Language (VRL)**: Embedded VRL execution engine for per-vendor (Fortinet, Cisco ASA, Palo Alto) mapping scripts.
- **`OcsfMapper`**: Maps parsed logs into the OCSF category 4 class 4001 schema.
- **Dead Letter Queue (DLQ)**: Dual-writes unparseable or alien logs to `/var/run/prism/dlq.log` and `.jsonl` for offline analysis.
- **`HttpSink`**: Exports normalized OCSF JSON to Elasticsearch using the `_bulk` NDJSON API, complete with error detection.

## Testing
To run the routing and transformation tests:
```bash
cargo test -p prism-core
```
