# PRISM Kibana Dashboard Requirements

This document defines the exact data payloads PRISM sends to Elasticsearch and the required Kibana visualizations for the PRISM "Executive View" dashboard.

## 1. The Data Payload (What the Backend Sends)

PRISM ingests raw logs from Cisco, Fortinet, and Palo Alto firewalls, normalizes them, and pushes them to Elasticsearch using the OCSF v1.9.0 (Network Activity) schema via our Rust Data Plane HTTP Bulk Exporter. 

Every single event arriving in Elasticsearch will be a JSON document that looks exactly like this:

```json
{
  "category_uid": 4,
  "class_uid": 4001,
  "activity_id": 2,
  "time": 1727271900,
  "src_endpoint": {
    "ip": "203.0.113.45"
  },
  "dst_endpoint": {
    "ip": "10.0.5.50"
  },
  "severity_id": 6,
  "severity": "High",
  "metadata": {
    "provenance_hash": "8a9b7c6d5e4f3a2b1c...",
    "vault_uri": "/vault/batch-942.parquet"
  }
}
```

### Field Definitions:
* **`time`**: Epoch timestamp of the event.
* **`activity_id`**: The firewall action. `1` (Allow), `2` (Deny), `3` (Reset).
* **`src_endpoint.ip`**: The IP address of the source.
* **`dst_endpoint.ip`**: The destination IP address.
* **`severity` / `severity_id`**: Threat level (e.g., "High" / 6, "Low" / 3, "Unknown" / 1).
* **`metadata.provenance_hash`**: The BLAKE3 cryptographic hash of the original raw log.
* **`metadata.vault_uri`**: Reference to the cold storage Parquet file.

---

## 2. Required Dashboard Visualizations

You must build the following Kibana panels mapping exactly to the backend fields provided above.

### A. Global Threat Map
* **Visualization:** Coordinate Map / Region Map
* **Input Field:** `src_endpoint.ip`
* **How it works:** Elasticsearch will geolocate the `src_endpoint.ip`. Plot these coordinates on the map.
* **Example:** IP `203.0.113.45` plots to a specific latitude/longitude, represented by a red dot if blocked, or a heatmap cluster.

### B. Traffic Action Breakdown
* **Visualization:** Donut Chart or Pie Chart
* **Input Field:** `activity_id`
* **How it works:** Group the events by `activity_id`.
* **Example Map:** 
  * `1` -> "Allowed" (Color: Green)
  * `2` -> "Denied" (Color: Red)
  * `3` -> "Reset" (Color: Yellow)

### C. Live Ingestion Throughput (EPS)
* **Visualization:** Time-Series Line Chart or Area Chart
* **Input Field:** `time` (X-axis) and Event Count (Y-axis)
* **How it works:** A standard EPS (Events Per Second) graph. Group the timestamp by 1-second intervals.
* **Example:** At exactly `1727271900`, the chart should spike to show `50,000` events hitting the system simultaneously during our load test.

### D. Forensic Event Ledger
* **Visualization:** Data Table
* **Input Fields:** `time`, `severity`, `src_endpoint.ip`, `dst_endpoint.ip`, `activity_id`, `metadata.provenance_hash`
* **How it works:** A raw table showing the most recent high-severity alerts. 
* **Requirement:** The `metadata.provenance_hash` MUST be visible. This hexadecimal string proves PRISM's compliance with Section 65B of the Indian Evidence Act (cryptographic traceability) and links the Elasticsearch UI back to the immutable vault.
* **Example Row:** 
  * Time: `Oct 14, 2026 @ 10:15:00`
  * Severity: `High`
  * Source: `203.0.113.45`
  * Action: `2` (Denied)
  * Hash: `8a9b7c6d5e4f3a2b1c...`

## 3. Demo Environment Constraints
* **Refresh Rate:** Set the Kibana dashboard to auto-refresh every 1 second.
* **Theme:** Use the Kibana Dark Theme to match the Ratatui terminal UI running on the primary screen.
