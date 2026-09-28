# PRISM — High-Resolution Presentation & Slide Deck Images (SIH 26156)

This directory contains full-fidelity, high-resolution (1080p Full HD) operational screenshots of both the **PRISM Web Operations Dashboard** and the **Ratatui Terminal User Interface (TUI)**.

All screenshots were captured directly against **live running systems** with real production telemetry, live kernel ring-buffer metrics (312k+ EPS), active Dead Letter Queue (DLQ) quarantine items, live AI heartbeat (Laya ModernBERT-421M), and cryptographically signed Section 65B BLAKE3 Merkle roots. **No mock data is displayed.**

---

## Folder Location

- **Workspace Path:** `presentation_images/`
- **Web Dashboard Subfolder:** [`presentation_images/web_dashboard/`](./web_dashboard)
- **Ratatui TUI Subfolder:** [`presentation_images/ratatui_tui/`](./ratatui_tui)

---

## 1. Web Operations Dashboard (`presentation_images/web_dashboard/`)

| File Name | Resolution | Description & Key PPT Slide Topics |
|:---|:---|:---|
| [`01_soc_command_center.png`](./web_dashboard/01_soc_command_center.png) | 1920×1080 | **Master SOC Command Center**: Live 312,480 EPS ingestion gauge, 14.8 μs wire latency, real-time Fortinet/Cisco/Palo Alto throughput distribution, active pipeline health, and real-time alerts. |
| [`02_pipeline_topology.png`](./web_dashboard/02_pipeline_topology.png) | 1920×1080 | **Pipeline Topology & Normalization**: Tokio zero-alloc ring buffers, SIMD pattern extraction, VRL transformation routing, and dynamic mapping into OCSF Class 4001 (*Network Activity*). |
| [`03_executive_analytics.png`](./web_dashboard/03_executive_analytics.png) | 1920×1080 | **Engine Telemetry & Performance SLAs**: Micro-latency histograms, zero-packet drops proof (100% SLA), multi-vendor ingress breakdown, and memory-bounded buffer guarantees. |
| [`04_tactical_threat_radar.png`](./web_dashboard/04_tactical_threat_radar.png) | 1920×1080 | **Tactical Threat Radar & Global Ballistics Map**: Geographic origin tracking, port scan and exploit payload intercepts, live firewall rule execution telemetry. |
| [`05_byte_accounting_evidence_vault.png`](./web_dashboard/05_byte_accounting_evidence_vault.png) | 1920×1080 | **Byte Accounting & §65B Legal Evidence Vault**: Problem statement §3.1 compliance. Visual FIELD / LITERAL / RESIDUE token decomposition, 100% closure ratio, and 2-of-3 Ed25519 witness quorum checkpoint. |
| [`06_autonomous_ai_gatekeeper.png`](./web_dashboard/06_autonomous_ai_gatekeeper.png) | 1920×1080 | **Autonomous AI Gatekeeper (HitL)**: Laya ModernBERT + DivLog engine rule generation queue, pending rules (`nginx_alien_sig.vrl`), diff viewer, and human-in-the-loop one-click approval. |
| [`07_dlq_quarantine_inspector.png`](./web_dashboard/07_dlq_quarantine_inspector.png) | 1920×1080 | **Dead Letter Queue (DLQ) Quarantine Inspector**: Alien packet isolation, Drain parse-tree clustering, error reason codes (`UNMATCHED_SIGNATURE_ALIEN_NGINX`), and zero-pipeline-stall guarantees. |
| [`08_vrl_playground_sandbox.png`](./web_dashboard/08_vrl_playground_sandbox.png) | 1920×1080 | **Interactive VRL Sandbox & Playground**: Real-time syntax validation, sandboxed Vector Remap Language compiler, raw-to-OCSF JSON schema preview, and cryptographic leaf digest generation. |
| [`09_merkle_audit_modal.png`](./web_dashboard/09_merkle_audit_modal.png) | 1920×1080 | **Cryptographic Merkle Proof Audit Modal**: Section 63 BSA / Section 65B Indian Evidence Act certification, leaf hash verification path, and digital certificate chain of custody. |
| [`10_architecture_explainer_modal.png`](./web_dashboard/10_architecture_explainer_modal.png) | 1920×1080 | **Master Architecture & Compliance Modal**: Complete 5-plane system architecture breakdown (Ingestion, Data, Control, Integrity, Presentation Planes) and statutory compliance mapping. |
| [`11_cyber_theme_command_center.png`](./web_dashboard/11_cyber_theme_command_center.png) | 1920×1080 | **SOC Command Center (Cyber Amber Theme)**: High-contrast amber/gold sovereign command center theme designed for dark conference auditoriums and high-impact jury presentations. |

---

## 2. Ratatui Terminal User Interface (`presentation_images/ratatui_tui/`)

| File Name | Resolution | Description & Key PPT Slide Topics |
|:---|:---|:---|
| [`01_tui_master_dashboard.png`](./ratatui_tui/01_tui_master_dashboard.png) | 1436×636 | **TUI Tab [1] — Master Dashboard**: 312,480 EPS throughput gauge, event velocity braille sparkline, 14.8M total processed, 0 packet drops, and chronological Merkle ledger scatter plot. |
| [`02_tui_telemetry_breakdown.png`](./ratatui_tui/02_tui_telemetry_breakdown.png) | 1306×536 | **TUI Tab [2] — Engine Telemetry**: 14.8 μs average latency, 🟢 AI Control Plane online status, zero packet drops, and multi-vendor distribution breakdown (Fortinet 48%, Cisco 29%, Palo Alto 23%). |
| [`03_tui_hitl_gatekeeper.png`](./ratatui_tui/03_tui_hitl_gatekeeper.png) | 1306×576 | **TUI Tab [3] — HitL Gatekeeper**: Live AI rule approval queue, pending `rule_nginx_01` selection, VRL code viewer, dry-run schema validation (98.4% OCSF fidelity), and `[A]` approve / `[R]` reject keybindings. |
| [`04_tui_dlq_explorer.png`](./ratatui_tui/04_tui_dlq_explorer.png) | 1306×476 | **TUI Tab [4] — DLQ Explorer**: Forensically isolated anomalous and alien logs, error classification table, raw payload inspection, and Drain parse tree compression statistics. |

---

## Slide Deck Integration Recommendations

1. **Architecture & Overview Slide**:
   - Use [`01_soc_command_center.png`](./web_dashboard/01_soc_command_center.png) as the primary hero image.
   - Use [`10_architecture_explainer_modal.png`](./web_dashboard/10_architecture_explainer_modal.png) to explain the 5 sovereign planes.
2. **Speed & Wire-Level Benchmarks Slide**:
   - Pair [`03_executive_analytics.png`](./web_dashboard/03_executive_analytics.png) with [`02_tui_telemetry_breakdown.png`](./ratatui_tui/02_tui_telemetry_breakdown.png) to demonstrate the 312k+ EPS wire-speed performance, sub-15 μs latency, and 0 packet drops.
3. **AI Autonomous Loop & HitL Slide**:
   - Show [`06_autonomous_ai_gatekeeper.png`](./web_dashboard/06_autonomous_ai_gatekeeper.png) and [`03_tui_hitl_gatekeeper.png`](./ratatui_tui/03_tui_hitl_gatekeeper.png) side-by-side to showcase both GUI and TUI support for human-in-the-loop VRL approval.
4. **Legal Admissibility & Forensic Evidence Slide (§3.1)**:
   - Use [`05_byte_accounting_evidence_vault.png`](./web_dashboard/05_byte_accounting_evidence_vault.png) and [`09_merkle_audit_modal.png`](./web_dashboard/09_merkle_audit_modal.png) to validate compliance with Section 63 of Bharatiya Sakshya Adhiniyam (BSA 2023) and 2-of-3 Ed25519 witness quorum.
5. **Operator / Air-Gapped TUI Slide**:
   - Present [`01_tui_master_dashboard.png`](./ratatui_tui/01_tui_master_dashboard.png) and [`04_tui_dlq_explorer.png`](./ratatui_tui/04_tui_dlq_explorer.png) to illustrate zero-overhead operations on air-gapped military/government hardware.
