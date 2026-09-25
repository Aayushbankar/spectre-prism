# HOW-TO: Ingestion Plane (Plane 1)

This guide details the steps to configure, test, and tune the PRISM Ingestion Plane.

## 1. Sending Test Logs via UDP
The Ingestion Plane listens on UDP port `514` by default. You can test it using `nc` (netcat):

```bash
echo "<34>1 2026-09-24T10:00:00Z mymachine su - ID47 - 'su root' failed" | nc -u 127.0.0.1 514
```

## 2. Tuning Socket Buffers
To prevent kernel-level UDP drops during massive bursts (e.g., >300k EPS), you must tune the Linux socket buffer sizes.

Run the following as root:
```bash
sudo sysctl -w net.core.rmem_max=8388608
sudo sysctl -w net.core.rmem_default=8388608
```

The `prism-ingest` crate explicitly asks for an 8MiB buffer (`SO_RCVBUF 8MiB`) using the `socket2` library. If the OS maximum is lower, the OS will silently truncate the requested buffer size, leading to drops under load.

## 3. Validating Concurrency and Zero Drops
The `prism-ingest` crate employs a lock-free `flume` channel sized at 50,000 to manage backpressure.

You can validate this behavior by running the dedicated ingest test suite:
```bash
cargo test -p prism-ingest test_udp_ingest_heterogeneous_concurrent -- --nocapture
```
This test concurrently spawns 10 senders that blast 2,000 messages each, asserting that absolutely zero drops occur at the ingest channel.

## 4. Monitoring
Once running, check the Ratatui TUI (Pane 1) to see the live EPS metric. It calculates the throughput directly from the `flume` channel drain rate.
