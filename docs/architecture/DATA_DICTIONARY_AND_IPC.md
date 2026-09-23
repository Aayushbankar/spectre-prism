# Data Dictionary & Inter-Process Communication (IPC)

**Project:** PRISM (SIH26156)

This document formalizes the data structures and the communication boundaries between the Rust (Hot Path) and Python (Offline Path) planes.

## 1. Inter-Process Communication (IPC) Contract

To maintain zero-copy speeds in the Data Plane and air-gapped security in the Control Plane, PRISM avoids REST APIs for internal module communication. It relies on **Asynchronous File-System IPC**.

### A. Rust -> Python (The Dead Letter Queue)
When `prism-core` (Rust) encounters an unknown log, it appends it to `dlq.log`.
* **Path:** `/var/run/prism/dlq.log` (test `/tmp/prism_dlq.log`)
* **Format:** JSON Lines (JSONL) for IPC, alongside a Plaintext replica for TUI.
* **Schema:** 
```json
{
  "raw_payload": "utf8:<UNRECOGNIZED_LOG_STRING>",
  "metadata": {
    "hash": "a1b2c3d4...",
    "timestamp": "2026-09-22T10:15:00Z",
    "source": { "Udp": "192.168.1.100:514" }
  }
}
```
* **Note:** `raw_payload` is strictly prefixed with `utf8:` or `b64:` to prevent heuristic base64 decoding corruption on binary syslogs.
* **Dual-Write Note:** `dlq.rs` writes plaintext `[TIMESTAMP] REASON PAYLOAD` for the Presentation TUI, and simultaneously writes `.jsonl` for the Python IPC.
* **Trigger:** `prism-brain` (Python) monitors the `.jsonl` file using `watchdog`.

### B. Python -> Rust (The Rule Hot-Reload)
When `prism-brain` successfully generates a new parser, it writes a `.vrl` script and a `.yaml` signature file.
* **Path:** `/etc/prism/rules/`
* **Trigger:** `prism-core` (Rust) uses the `notify` crate to detect the `Create` event in the directory. It dynamically loads the `.vrl` AST into memory without dropping network packets.

---

## 2. OCSF Data Dictionary (The Standard Output)

Regardless of the input vendor (Cisco, Fortinet, Check Point), PRISM guarantees output conforming to the **Open Cybersecurity Schema Framework (OCSF) v1.9.0 - Class 4001 (Network Activity)**.

| OCSF Field | Type | Required | Description / PRISM Mapping |
| :--- | :--- | :--- | :--- |
| `category_uid` | Integer | **Yes** | Hardcoded to `4` (Network Activity). |
| `class_uid` | Integer | **Yes** | Hardcoded to `4001` (Network Activity). |
| `activity_id` | Integer | **Yes** | Mapped by VRL. E.g., `1` (Allow), `2` (Deny), `3` (Reset). |
| `time` | Long | **Yes** | Epoch timestamp of the log generation. |
| `src_endpoint.ip` | String | **Yes** | Derived directly from VRL `.ip` extraction. |
| `severity_id` | Integer | **Yes** | Severity ID (e.g., 1 for Unknown, 3 for Low, 6 for High). |
| `severity` | String | **Yes** | String representation of severity (e.g., "High"). |
| `status_id` | Integer | **Yes** | Status ID (e.g., 1 for Success, 2 for Failure). |
| `confidence` | Integer | **Yes** | Confidence score of the parsing/classification. |
| `type_uid` | Integer | **Yes** | OCSF Event Type ID. |
| `observables` | Array[String] | **Yes** | List of extracted observables (e.g., IPs, Domains). |
| `src_endpoint.ip` | String | **Yes** | Extracted Source IP. |
| `src_endpoint.port` | Integer | No | Extracted Source Port. |
| `dst_endpoint.ip` | String | **Yes** | Extracted Destination IP. |
| `metadata.version` | String | **Yes** | PRISM Schema Version. |
| `metadata.provenance_hash` | String | **Yes** | **Custom PRISM Extension:** The BLAKE3 hash of the raw log to satisfy forensic traceability. |
| `metadata.vault_uri` | String | **Yes** | **Custom PRISM Extension:** The Parquet block ID where the raw log is stored. |
