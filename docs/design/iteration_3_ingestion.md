# Design Document: Iteration 3 - Ingestion Completeness

## Overview
This document outlines the design decisions and research for implementing highly concurrent, scalable TCP ingestion and robust File Tailing in Rust within our Tokio-based PRISM pipeline.

## 1. TCP Ingestion

### Options Evaluated
1. **Raw `std::net::TcpListener` + Thread Pool:**
   - *Pros:* Simple, no async overhead.
   - *Cons:* Does not scale well to 10k+ concurrent connections due to OS thread limits. Blocking I/O would stall the dispatcher.
2. **`tokio::net::TcpListener` + `tokio_util::codec::FramedRead`:**
   - *Pros:* Handles framing (newline-delimited) automatically. Clean abstraction.
   - *Cons:* Requires adding `tokio-util` dependency.
3. **`tokio::net::TcpListener` + `BufReader::lines()`:**
   - *Pros:* Native to Tokio. Simple, no extra dependencies. Highly performant for line-delimited syslog.
   - *Cons:* Slightly more manual error handling for broken lines compared to `FramedRead`.

### Selected Approach: `tokio::net::TcpListener` with `BufReader::lines()`
- We will spawn a Tokio task for each accepted connection.
- A `tokio::io::BufReader` will wrap the TCP stream, and `.lines()` will be used to extract discrete log events.
- **Backpressure:** By using the existing `flume` bounded channels in `prism-ingest`, `send_async()` will automatically yield and pause reading from the TCP socket if the dispatcher pipeline backs up. This naturally applies TCP backpressure via the window size.

## 2. File Tailing

### Options Evaluated
1. **Raw Polling (`std::fs::File` + `tokio::time::sleep`):**
   - *Pros:* Zero dependencies.
   - *Cons:* CPU intensive. Misses edge cases like file truncation (e.g., `logrotate`).
2. **`notify` Crate (inotify / fsevents):**
   - *Pros:* Standard for file system events in Rust. Highly efficient.
   - *Cons:* Very low-level. Requires manual implementation of file position tracking, handling rotations, truncations, and deleted files.
3. **`linemux` Crate:**
   - *Pros:* Built specifically for async line tailing. Natively integrates with Tokio. Handles log rotations and truncations out-of-the-box.
   - *Cons:* Adds an external dependency.

### Selected Approach: `linemux`
- For a hackathon timeline and robust production behavior, reinventing file tailing over `notify` is risky. `linemux::MuxedLines` allows us to simply add files to a watcher and stream lines as an async iterator.
- It inherently supports dynamic file creation and rotation, which is critical for real-world `/var/log/*` scenarios.
- The stream will pipe lines directly into the existing `flume` channel, sharing the same backpressure mechanisms as TCP/UDP.

## 3. Unified Ingestion Config

- We will modify `main.rs` to parse a structured `.toml` configuration (or robust CLI arguments) allowing the user to define a heterogeneous list of inputs.
- E.g., `inputs = [{ type = "tcp", port = 514 }, { type = "file", path = "/var/log/syslog" }]`.
- `main.rs` will spawn the respective listener/tailer tasks, all feeding into the same central dispatcher `flume` channel.
