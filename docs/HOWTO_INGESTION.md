# HOW-TO: Ingestion Plane

This guide explains how to send logs to the PRISM ingestion plane and verify reception.

## Prerequisites
- PRISM must be running.
- You need `nc` (netcat) installed.

## 1. Sending Test Logs
Use netcat to send a standard syslog UDP packet to the local ingestion port (514):

```bash
echo "<34>1 2026-09-24T10:00:00Z mymachine su - ID47 - 'su root' failed" | nc -u 127.0.0.1 514
```

## 2. Tuning Socket Buffers
To ensure zero drops under heavy load, tune the receive buffer:
```bash
sudo sysctl -w net.core.rmem_max=8388608
sudo sysctl -w net.core.rmem_default=8388608
```
The `prism-ingest` crate uses `SO_RCVBUF 8MiB`.

## 3. Verifying Concurrency
The ingestion plane uses a `flume` lock-free channel with a capacity of 50,000. It dual-routes messages to the Data Plane and the Integrity Plane seamlessly.
Monitor the EPS (Events Per Second) in the Ratatui TUI or check the raw `flume` metrics.
