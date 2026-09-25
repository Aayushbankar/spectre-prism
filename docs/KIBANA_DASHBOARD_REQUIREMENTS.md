# Kibana Dashboard Design Requirements (PRISM "Executive View")

**Project:** PRISM (SIH26156)  
**Target:** Kibana Dashboards (Elasticsearch 8.x)  
**Role:** Executive SIEM View for Dual-Dashboard Hackathon Demo  

## 1. Overview & Objective
For our final SIH demo, PRISM utilizes a dual-screen presentation. Screen 1 is the "Engine Room" (Ratatui TUI showing raw Rust performance). Screen 2 is the **"Executive View" (Kibana Dashboard)**. 

Your objective is to build this Kibana Dashboard. The dashboard must visually prove that PRISM successfully ingests heterogeneous, proprietary firewall logs (Cisco, Fortinet, Palo Alto) and normalizes them into a single, unified format that standard SIEM tools can instantly understand. This directly satisfies PRISM's Requirement (f) Unified Visibility and (g) SIEM Integration.

## 2. Data Source & Schema
PRISM's Rust Data Plane automatically batches and pushes logs to Elasticsearch via the HTTP Bulk Exporter. 
You will be querying an Elasticsearch index (e.g., `prism-logs-*`).

**Crucial:** All data arriving in Elasticsearch strictly conforms to the **Open Cybersecurity Schema Framework (OCSF) v1.9.0 - Class 4001 (Network Activity)**. 

### Key OCSF Fields to Utilize:
*   `time` (Long): The epoch timestamp of the log. **Use this as your primary `@timestamp` for time-series charts.**
*   `activity_id` (Integer): The action taken by the firewall. `1` = Allow, `2` = Deny, `3` = Reset.
*   `src_endpoint.ip` (String): Source IP address.
*   `dst_endpoint.ip` (String): Destination IP address.
*   `severity` (String) / `severity_id` (Integer): Log severity (e.g., "High" / 6).
*   `metadata.provenance_hash` (String): PRISM's custom forensic extension. The BLAKE3 hash of the original raw log.

## 3. Required Dashboard Panels

Please implement the following panels in the Kibana Dashboard:

### Panel 1: Global Threat Map (Geo-IP)
*   **Type:** Coordinate Map / Region Map
*   **Metric:** Count of events.
*   **Bucket:** Geo-Hash on `src_endpoint.ip` (Requires an ingest pipeline or Logstash GeoIP filter if not enriched prior, or simply assume PRISM will enrich it/use dummy coordinates for the demo).
*   **Purpose:** Visual eye-candy for the judges showing where attacks are originating.

### Panel 2: Traffic Disposition (Allowed vs. Blocked)
*   **Type:** Pie Chart or Donut Chart
*   **Metric:** Count
*   **Bucket:** Terms aggregation on `activity_id` or mapped string aliases (Allow, Deny, Reset).
*   **Colors:** `1` (Allow) = Green, `2` (Deny) = Red, `3` (Reset) = Yellow.
*   **Purpose:** Proves PRISM can extract firewall actions from completely different vendors and map them to a single metric.

### Panel 3: Ingestion Throughput Over Time (EPS)
*   **Type:** Line Chart / Area Chart (Time Series)
*   **Y-Axis:** Count of Events
*   **X-Axis:** Date Histogram on `time` (per second or per 5 seconds).
*   **Purpose:** Visually mirrors the high EPS count shown on the Ratatui TUI, proving that the normalized data is arriving at the SIEM in real-time.

### Panel 4: High Severity Alerts & Forensic Ledger
*   **Type:** Data Table
*   **Columns:** 
    *   `time`
    *   `severity`
    *   `src_endpoint.ip`
    *   `dst_endpoint.ip`
    *   `activity_id`
    *   `metadata.provenance_hash`
*   **Filter:** `severity_id >= 4` (or filter by "High").
*   **Purpose:** Highlights PRISM's unique **Section 65B Evidentiary Compliance**. The presence of the `provenance_hash` in the SIEM alert allows an analyst to tie the normalized alert back to the exact cryptographically signed raw log in PRISM's Cold Vault.

## 4. Demo Execution Notes
During the presentation:
1.  We will blast 50,000 mixed logs into PRISM.
2.  The Kibana dashboard should auto-refresh every 1-2 seconds.
3.  The dashboard should instantly populate with the unified metrics, despite the source data being messy and proprietary. 
4.  Ensure the dashboard has a clean, dark-mode theme to match the terminal aesthetic of the TUI.
