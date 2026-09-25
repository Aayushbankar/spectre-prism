# HOW-TO: Data Plane (Plane 2)

This guide covers the routing, mapping, and Dead Letter Queue (DLQ) mechanics within the PRISM Data Plane.

## 1. How the Heuristic Router Works
The router sits before the VRL engine. It uses highly optimized SIMD `memchr` to find vendor-specific substrings (e.g., `%ASA-` for Cisco).

You can run the routing benchmark to observe the sub-microsecond latency:
```bash
cargo test -p prism-core bench_router_heuristic -- --nocapture
```
This tests routing 1 million logs, proving latency stays well under 5µs.

## 2. Managing VRL Rules
Parsing rules are written in Vector Remap Language (VRL).
- Rules reside in memory but are hot-reloaded from `/etc/prism/rules`.
- They map specific vendor formats directly into OCSF Category 4, Class 4001 (Network Activity).

## 3. Simulating Alien Logs
Logs that the Router doesn't recognize are classified as `Unknown` and sent to the DLQ.

To simulate an alien log:
```bash
echo "I am an entirely unrecognized log format!" | nc -u 127.0.0.1 514
```

## 4. Checking the DLQ
The DLQ writes simultaneously to two formats. Verify them using:

```bash
cat /var/run/prism/dlq.log
cat /var/run/prism/dlq.jsonl
```
The `.jsonl` version is actively monitored by the Python Control Plane (`prism-brain`) using `inotify`.

## 5. End-to-End Routing Test
To test the complete flow from routing to OCSF normalization:
```bash
cargo test -p prism-core test_data_plane_routing
```
This test ensures recognized logs are mapped correctly and unknown logs are routed strictly to the DLQ.
