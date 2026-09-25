# `prism-ingest`

The `prism-ingest` crate forms the Ingestion Plane (Plane 1) of the PRISM framework. It is responsible for absorbing massive volumes of log data with zero drops and near-zero latency, functioning as the high-speed gateway into the system.

## Architectural Highlights

### 1. Zero Per-Packet Allocation
Instead of allocating memory for every incoming log line, `prism-ingest` pre-allocates large `BytesMut` chunks (10MiB). As packets arrive, they are sliced from this continuous block. This amortizes the allocation cost (approximately 1 allocation per 70,000 logs), ensuring CPU time is spent on I/O rather than memory management.

### 2. Lock-free Dispatching (`flume`)
Logs are instantly dispatched to both the Data Plane and the Integrity Plane via a dual-write mechanism using `flume` lock-free channels. The channel capacity is tuned to 50,000 events to absorb sudden spikes in log traffic.

### 3. High-Performance `UdpListener`
The UDP listener is heavily tuned for raw throughput:
- Utilizes `SO_RCVBUF 8MiB` socket buffer sizes to prevent kernel-level packet drops under heavy load.
- Operates asynchronously using `tokio`, capable of handling thousands of concurrent senders.

## Benchmarks
In heterogeneous concurrent testing (`test_udp_ingest_heterogeneous_concurrent`):
- Sustains bursts of 100,000 messages with 0 stall and 0 drops.
- Consistently validates throughput exceeding 50,000 Events Per Second (EPS).

## How to Test
You can run the ingest-specific test harness using:

```bash
cargo test -p prism-ingest
```

This will run tests like `test_drop_under_pressure`, `test_udp_ingest_heterogeneous_concurrent`, and `test_capacity_invariance_100k` to validate the backpressure mechanisms and zero-drop guarantees.
