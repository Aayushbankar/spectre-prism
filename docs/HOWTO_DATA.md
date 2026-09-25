# HOW-TO: Data Plane

This guide explains how PRISM normalizes diverse logs into the OCSF schema.

## 1. Vector Remap Language (VRL)
PRISM uses VRL to write highly performant parsing scripts per-vendor.
Rules are stored in `/etc/prism/rules/`.

## 2. Running the Data Plane Pipeline
To test the routing (SIMD `memchr`) and VRL mapping:
```bash
cargo test -p prism-core test_data_plane_routing
```
This converts a raw vendor string into a standard OCSF JSON structure (e.g., Category 4, Class 4001 for Network Activity).

## 3. Dead Letter Queue (DLQ)
When the `HeuristicRouter` encounters an alien log that has no matching rule, it dual-writes the log to the DLQ:
- `/var/run/prism/dlq.log`
- `/var/run/prism/dlq.jsonl`

These files are monitored by the Control Plane to generate new rules.
