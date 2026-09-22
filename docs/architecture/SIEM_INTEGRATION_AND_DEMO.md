# SIEM Integration & Dual-Dashboard Demo Plan

## 1. Addressing PS Requirement (g): Efficient SIEM Integration
PRISM does not trap data; it is a pre-processor. Once `prism-core` maps raw logs into OCSF JSON, it must export them to industry-standard dashboards.

### The Export Sink
We will implement an **HTTP Bulk Exporter** in the Rust Data Plane. 
* As logs are mapped to OCSF, they are batched.
* The batch is flushed via HTTP POST directly to an Elasticsearch/OpenSearch indexing API or a Splunk HTTP Event Collector (HEC).
* For Data Lakes, a secondary sink can write batched OCSF JSON into compressed Apache Parquet files.

## 2. Addressing PS Requirement (k): Containerization
The entire PRISM stack will be packaged using `docker-compose`. This ensures platform independence and solves the demo deployment constraint.

The stack will include:
1. `prism-core` (Rust - Ingestion/Mapping)
2. `prism-brain` (Python/Ollama - AI Control Plane)
3. `elasticsearch` (Industry SIEM datastore)
4. `kibana` (Industry Dashboard)

## 3. The Dual-Dashboard Demo Strategy (For the Hackathon)
To prove both the engineering depth and the commercial viability of PRISM, the demo will use a dual-screen approach:

### Screen 1: The "Engine Room" (Ratatui TUI)
* **What it shows:** The raw engineering power.
* **Visuals:** Live EPS meters (hitting 50k+), the Dead Letter Queue catching unknown logs, and the AI Terminal auto-drafting VRL scripts.
* **Proves:** Scalability, AI Onboarding, Zero-Copy speed.

### Screen 2: The "Executive View" (Kibana / Grafana)
* **What it shows:** The business value of normalized OCSF data.
* **Visuals:** A standard Kibana dashboard showing Geo-IP maps of attackers, pie charts of Allowed vs. Dropped firewall traffic, and trend lines.
* **Proves:** Unified visibility (Req f), SIEM integration (Req g), and that the OCSF output works perfectly with standard industry tools.
