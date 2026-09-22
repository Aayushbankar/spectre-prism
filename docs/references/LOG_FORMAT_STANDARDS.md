# Log Format Standards & Specifications Reference

> **Purpose:** Verified RFC numbers, specs, and tools relevant to SIH26156.
> **Source:** Researched from IETF RFC database, vendor documentation, official project sites.
> **Rule:** Nothing in this file is opinion or interpretation. Every entry has a named source.

---

## 1. Syslog Standards (IETF RFCs)

### Format Specifications

- **RFC 3164 — The BSD Syslog Protocol** (August 2001, Informational)
  Documents the legacy Unix BSD syslog wire format. Defines PRI (Facility + Severity), Header (TIMESTAMP in `Mmm dd hh:mm:ss` — **no year, no timezone**), HOSTNAME, and MSG body. No guaranteed delivery.

- **RFC 5424 — The Syslog Protocol** (March 2009, Standards Track — obsoletes RFC 3164)
  Modern extensible syslog. Strict header: PRI, VERSION, ISO 8601 microsecond TIMESTAMP, HOSTNAME, APP-NAME, PROCID, MSGID. Adds STRUCTURED-DATA (key-value SD-ELEMENTs). UTF-8 payload.

### Transport Specifications

- **RFC 5426 — Syslog over UDP** (March 2009, Standards Track)
  UDP port 514. Covers MTU handling, message truncation behavior, congestion avoidance.

- **RFC 5425 — TLS Transport for Syslog** (March 2009, Standards Track)
  Secure syslog over TCP+TLS on port 6514. Mutual cert auth, encryption, octet-counting framing.

- **RFC 6587 — Syslog over TCP** (April 2012, Informational)
  Historical non-crypto TCP practices. Two framing methods: octet-counting and non-transparent delimiter (LF/NUL).

- **RFC 6012 — DTLS Transport for Syslog** (October 2010, Standards Track)
  Secure datagram transport via DTLS over UDP. Confidentiality + replay protection without TCP state.

### Cryptographic & Management

- **RFC 5848 — Signed Syslog Messages** (May 2010, Standards Track)
  Digital signature framework using SD Elements. Signature Blocks + Certificate Blocks for verifying integrity, origin authenticity, and detecting missing/tampered entries.

- **RFC 3195 — Reliable Delivery for Syslog** (November 2001, Standards Track)
  Reliable delivery over BEEP framework. Sequenced, lossless delivery with optional SASL/TLS.

- **RFC 5427 — Textual Conventions for Syslog Management** (March 2009, Standards Track)
  SNMP MIB textual conventions for syslog facilities, severities, SD parameter names.

---

## 2. CEF (Common Event Format)

- **Creator:** ArcSight, Inc. (acquired by HP 2010 → Micro Focus 2017 → OpenText 2023)
- **RFC Status:** **No RFC exists.** Open proprietary vendor specification.
- **Document:** "ArcSight Common Event Format (CEF) Implementation Standard"
- **Structure:**
  ```
  CEF:Version|Device Vendor|Device Product|Device Version|Device Event Class ID|Name|Severity|[Extension]
  ```
  7-field pipe-delimited header + space-delimited `key=value` Extension pairs.
  Typically transported inside Syslog envelopes (RFC 3164 or 5424).

---

## 3. LEEF (Log Event Extended Format)

- **Creator:** IBM Security, for IBM QRadar SIEM.
- **RFC Status:** **No RFC exists.** Proprietary vendor specification.
- **Document:** "IBM Security QRadar Log Event Extended Format (LEEF) Implementation Guide"
- **LEEF 1.0:**
  ```
  LEEF:1.0|Vendor|Product|Version|EventID|key=value<tab>key2=value2
  ```
  5 pipe-delimited header fields + tab-delimited key-value attributes.
- **LEEF 2.0:**
  ```
  LEEF:2.0|Vendor|Product|Version|EventID|Delimiter|key=value<del>key2=value2
  ```
  Adds 6th header field for custom delimiter.

---

## 4. Normalization / Taxonomy Standards

### IETF RFCs

- **RFC 5674 — Alarms in Syslog** (Oct 2009) — Taxonomy for alarm/fault reporting inside RFC 5424 structured syslog. Maps ITU-T X.733 severities to syslog.
- **RFC 4765 — IDMEF** (March 2007, Experimental) — XML schema with 33 classes, 108 fields for normalizing IDS events.
- **RFC 7970 — IODEF-v2** (Nov 2016) — XML model for incident response exchange between CSIRTs.
- **RFC 7011 & 7012 — IPFIX** (Sep 2013) — Network flow log telemetry standard (successor to NetFlow v9). IANA information element registry.

### Open Industry Standards (Non-RFC)

- **OCSF (Open Cybersecurity Schema Framework)**
  - Linux Foundation. Launched August 2022 by AWS, Splunk, CrowdStrike, Broadcom, others.
  - De facto open standard for vendor-agnostic security log taxonomy. JSON schema classes, categories, attribute dictionaries.

- **ECS (Elastic Common Schema)**
  - Created by Elastic (2019). Donated to CNCF/OpenTelemetry in 2023.
  - Field naming convention standard (`source.ip`, `destination.port`, `event.action`, `user.name`).

- **OpenTelemetry Semantic Conventions**
  - CNCF. Standardizes log data models (Resource, Scope, Attributes, Body, Timestamp, SeverityNumber).

- **MITRE CEE (Common Event Expression)**
  - **Discontinued/Archived (2013).** Early taxonomy initiative. Concepts influenced OCSF.

---

## 5. VRL (Vector Remap Language)

- **What:** Domain-specific, expression-oriented language for transforming observability data (logs, metrics, traces).
- **Key Properties:**
  - Memory-safe, abort-safe. Fallible functions require explicit handling (`!` or `??`).
  - Compile-time type checking/inference.
  - Written in Rust, executes natively (no interpreter overhead).
  - Built-in parsers: `parse_syslog`, `parse_cef`, `parse_common_log`, `parse_json`, `parse_regex`, `parse_groks`.
- **Docs:** https://vector.dev/docs/reference/vrl/
- **Functions:** https://vector.dev/docs/reference/vrl/functions/
- **Playground:** https://vrl.dev/

---

## 6. Vector (Log Pipeline Tool)

- **What:** Open-source, high-performance observability data pipeline. Collects, transforms, routes logs/metrics/traces.
- **Built in:** Rust. No GC pauses. Minimal memory/CPU footprint.
- **Pipeline model:** DAG of Sources → Transforms → Sinks.
- **Creator:** Timber Technologies (2019). **Acquired by Datadog (Feb 2021).**
- **License:** MPL-2.0 (Mozilla Public License 2.0).
- **Status:** Active, production-grade. Powers Datadog Observability Pipelines.
- **Site:** https://vector.dev/
- **Source:** https://github.com/vectordotdev/vector
