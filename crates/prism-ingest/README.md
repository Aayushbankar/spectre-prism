# `prism-ingest`

The `prism-ingest` crate is the entry point for raw log data in the PRISM framework. It provides high-throughput network ingestion.

## Key Features

- **`UdpListener`**: High-performance UDP socket ingestion.
- **10MiB Chunking**: Batches data in memory to minimize I/O overhead.
- **Socket Tuning**: Leverages `SO_RCVBUF 8MiB` to prevent packet drops under heavy load.
- **Dispatcher**: Lock-free `flume` dispatcher with 50,000 channel capacity for dual routing to the Data and Integrity planes.
- **Zero Allocation**: Employs a zero per-packet allocation design using an amortized buffer.

## Benchmarks
Achieves over 325,000 EPS on standard hardware with zero drops.

## Testing
To run the ingestion tests:
```bash
cargo test -p prism-ingest
```
