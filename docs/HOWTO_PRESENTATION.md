# HOW-TO: Presentation Plane

This guide explains how to spin up the UI components for the SIH live demo.

## 1. Elasticsearch and Kibana
Bring up the backing store using Docker Compose:
```bash
docker compose up -d --wait
```
This launches Elasticsearch 8.13.4 and Kibana.

## 2. Simulating Traffic
Push 50,000 events to the `_bulk` endpoint to populate the `prism-ocsf*` index. Kibana threat maps will automatically read this index.

## 3. Running the Terminal UI (TUI)
In a separate split-screen terminal, launch the Ratatui dashboard:
```bash
cargo run -p prism-tui
```
This will display the 4-pane Engine Room showing live EPS throughput, the DLQ table, Merkle commits, and pending HitL rules.
