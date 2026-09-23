# Data Dictionary & Inter-Process Communication (IPC)

**Project:** PRISM (SIH26156)

This document formalizes the data structures and the communication boundaries between the Rust (Hot Path) and Python (Offline Path) planes.

## 1. Inter-Process Communication (IPC) Contract

To maintain zero-copy speeds in the Data Plane and air-gapped security in the Control Plane, PRISM avoids REST APIs for internal module communication. It relies on **Asynchronous File-System IPC**.

### A. Rust -> Python (The Dead Letter Queue)
When `prism-core` (Rust) encounters an unknown log, it appends it to `dlq.log`.
* **Path:** `/var/run/prism/dlq.log`
* **Format:** JSON Lines (JSONL)
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
* **Note**: `raw_payload` is strictly prefixed with `utf8:` or `b64:` to prevent heuristic base64 decoding corruption on binary syslogs.
* **Trigger:** `prism-brain` (Python) monitors this file using `watchdog`.

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
| `src_endpoint.ip` | String | **Yes** | Extracted Source IP. |
| `src_endpoint.port` | Integer | No | Extracted Source Port. |
| `dst_endpoint.ip` | String | **Yes** | Extracted Destination IP. |
| `metadata.provenance_hash` | String | **Yes** | **Custom PRISM Extension:** The BLAKE3 hash of the raw log to satisfy forensic traceability. |
| `metadata.vault_uri` | String | **Yes** | **Custom PRISM Extension:** The Parquet block ID where the raw log is stored. |
