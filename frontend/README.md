# PRISM Frontend Dashboard · SIH PS 26156

A high-performance, real-time observability and telemetry dashboard engineered for the **PRISM Universal Log Parsing Framework** (Smart India Hackathon 2026, Problem Statement 26156).

This dashboard seamlessly adapts and synthesizes all three reference designs into a unified, responsive interface:

---

## 🎨 Implemented Design Visualizations

### 1. Ingestion & Traffic Flow (`ideainso3.jpeg`)
* **Executive Natural Language Summary**: Dynamic AI narrative highlighting total event volume (2.66M+), active sensor nodes (1,420), payload volume (14.59 MB/s), 5xx backend server errors (87), and OCSF Class 4001 compliance.
* **Health Score & SLI Rings**: Overall Ingestion Health Gauge (**94 / 100 GOOD**) with circular SVG indicators for **Success Rate (92.3%)**, **Availability (96.7%)**, **Not Found / DLQ (4.4%)**, and **Error Budget (92.3%)**.
* **Stat Cards Strip**: Total Requests (Cyan accent), Data Transferred to Vault (Blue accent), Unique Origin Nodes (Purple accent), and Error Rate (Rose accent).
* **Interactive SVG Sankey Diagram**:
  * **Column 1 (Origin Regions)**: China (`CN`), India (`IN`), United States (`US`), Indonesia (`ID`), Brazil (`BR`), Pakistan (`PK`), Bangladesh (`BD`), Nigeria (`NG`), and other aggregated sectors.
  * **Column 2 (Engine Planes)**: Palo Alto VRL Pipe, FortiGate Parser, Cisco ASA Normalizer, Drain3 AI Miner, and Merkle Vault 65B.
  * **Column 3 (Status Egress)**: `200 OK · OCSF Class 4001` (Green/Cyan ribbon), `404 Blocked / Denied` (Amber ribbon), `503 DLQ / HitL Triage` (Magenta ribbon).
  * Interactive hover highlighting, volume tooltips, and cubic Bézier flow ribbons.

### 2. Executive Analytics & Operations (`dashboard2.jpeg`)
* **Dual Theme Engine**: Full support for **Dark Theme**, **Light Theme**, and **Tactical Cyber Theme** with an instant toggle.
* **KPI Metrics with Sparklines**:
  * **Ingest Throughput (EPS)**: 856k EPS (-12% downward trend sparkline).
  * **Sensor Nodes Online**: 523 nodes (+23% upward trend sparkline).
  * **Parse Latency (p99)**: 9.56 µs (+8% trend sparkline).
  * **Protocol Breakdown Donut**: UDP Syslog (49%), TCP/TLS (36%), NetFlow v9 (15%).
* **Real-Time Buffer Progress Bars**: Ingest Chunks, Drain3 Mining Trees, VRL Transforms, and DLQ Quarantined Events with glowing live gauges.
* **Smooth Spline Area Chart**: Interactive monthly time-series showing Raw Ingested EPS vs Normalized OCSF EPS with hover tooltip (`680k EPS Aug 16`).
* **Firewall Vendor Breakdown**: Multi-segment Donut Chart featuring Palo Alto (45%), Fortinet (20%), Cisco ASA (15%), AWS VPC Flow (11%), and Suricata IDS (9%).
* **Anomaly Latency Bins**: Rounded vertical bar chart with highlighted peak bar (`12.51 µs`).
* **Top Rules & Endpoints**: Ranked visual progress list for top VRL rules and OCSF destinations.

### 3. Tactical Command & Threat Intelligence (`dashboard_1.jpeg`)
* **Amber / Gold Cyber HUD**: High-contrast dark cyber theme optimized for SOC command centers.
* **Interactive Global Threat Vector Map**:
  * Vector world map with geolocated threat beacons (China, Russia, USA, India, Germany, Brazil, Indonesia, Australia).
  * Animated ballistic trajectory arcs connecting threat origin coordinates to the central PRISM Defense Gateway in India.
  * Sector details panel with click-to-filter capability.
* **Telemetry & Threat Histograms**: Multi-series frequency histogram with trajectory curves and dual-threshold telemetry envelopes.
* **Threat Spectrum Donut**: AI triage breakdown (Brute Force 40%, Port Sweep 25%, DNS Amplification 20%, C2 Beaconing 15%).
* **Orbital Node Topology**: Central Tokio MPSC channel core with satellite parser engines.
* **Forensic Event Ledger**:
  * Full OCSF v1.9 Network Activity Class 4001 tabular log stream.
  * **Section 65B Indian Evidence Act Compliance**: Displays 256-bit **BLAKE3 cryptographic provenance hashes** linking directly to Parquet 2.0 immutable cold vault batches.
  * **Interactive Proof Verification Modal**: Allows security operators to inspect and verify BLAKE3 integrity, Parquet chunk URI, and Merkle leaf indexes in real-time.

---

## 🚀 Running the Dashboard

```bash
cd frontend
npm install
npm run dev
```

To build for production:
```bash
npm run build
npm run preview
```
