# HOW-TO: Presentation Plane (Plane 5)

This runbook guides you through running the SIH Live Demo for PRISM.

## 1. Starting Elasticsearch and Kibana
The Presentation Plane outputs normalized OCSF logs into an Elasticsearch `_bulk` NDJSON API.

Start the observability stack via Docker Compose:
```bash
docker compose up -d --wait
```
This boots Elasticsearch 8.13.4 and Kibana in the background.

## 2. Generating Demo Logs
Once ES is running, push 50,000 heterogeneous events into the ingestion plane to populate the indices:
```bash
./scripts/generate_traffic.sh --count 50000 --rate 5000
```
*(Alternatively, run `cargo test test_data_plane_routing` to push the data programmaticially).*

## 3. Configuring Kibana
1. Navigate to `http://localhost:5601`.
2. Go to **Stack Management > Data Views**.
3. Create a data view for the pattern: `prism-ocsf-*`.
4. Open the **Threat Map** dashboard. You will see geographic IP mapping rendered from the OCSF Network Activity (Class 4001) normalized logs.

## 4. Running the Ratatui Dashboard (TUI)
For an edge-node, zero-GUI presentation, run the 4-Pane Engine Room TUI in a split terminal:

```bash
cargo run -p prism-tui
```

This interface directly polls `/var/run/prism/dlq.log` and the `prism-provenance` ticker asynchronously.
- **Pane 1**: Live EPS (Events Per Second) Gauge.
- **Pane 2**: Live DLQ Table.
- **Pane 3**: Live Merkle Root Ticker (updates every 500ms).
- **Pane 4**: Human-in-the-Loop (HitL) Gatekeeper pending approvals.
