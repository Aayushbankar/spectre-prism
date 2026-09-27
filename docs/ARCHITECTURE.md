# PRISM System Architecture Specification

PRISM is a high-throughput, sovereign SIEM ingestion and universal log normalization engine engineered in Rust and Python for national defense and critical infrastructure networks (SIH PS-26156).

---

## 🏛️ 1. High-Level Five-Plane Architecture

<p align="center">
  <img src="images/architecture_diagram.png" alt="PRISM Five-Plane Architecture" width="100%">
</p>

<details>
<summary><b>🔍 View Architecture Diagram Mermaid Source</b></summary>

```mermaid
flowchart TD
    subgraph INGEST["Ingestion Plane (Zero-Copy)"]
        UDP["UDP Syslog (SO_REUSEPORT)"]
        TCP["TCP Syslog (RFC 5424)"]
        FILE["File Ingest (Inotify)"]
        DISPATCH["MPMC Dispatcher (Flume Channels)"]
        UDP --> DISPATCH
        TCP --> DISPATCH
        FILE --> DISPATCH
    end

    subgraph PROVENANCE["Provenance Plane (SIMD & Crypto)"]
        BLAKE3["BLAKE3 Hasher (Zero-Copy SIMD)"]
        VAULT["ZSTD Compressed Vault"]
        MERKLE["RFC 6962 Merkle Tree"]
        WITNESS["Section 65B Witness Quorum (2-of-3 Ed25519)"]
        DISPATCH --> BLAKE3
        BLAKE3 --> VAULT
        BLAKE3 --> MERKLE
        MERKLE --> WITNESS
    end

    subgraph DATAPLANE["Data Normalization Plane"]
        ROUTER{"Heuristic Router\n(SIMD Byte Match)"}
        VRL["In-Memory VRL Engine (Hot-Reload)"]
        OCSF["Dynamic OCSF Normalizer (4001, 3001, 5001, 8001)"]
        ACCOUNT["Byte Accounting Engine (>95% Closure)"]
        SINK["SIEM Sink (Elasticsearch / Parquet)"]
        DISPATCH --> ROUTER
        ROUTER -->|Matched| VRL
        VRL --> OCSF
        OCSF --> ACCOUNT
        ACCOUNT --> SINK
    end

    subgraph CONTROL["Autonomous AI Control Plane (HitL)"]
        DLQ["Quarantine DLQ (/vault/dlq_*.log)"]
        DRAIN["Drain3 Clustering (Tree Depth & Masking)"]
        LAYA["Laya ModernBERT-large (421M Params)"]
        CODER["VRL Code Synthesizer & Scorer"]
        REPARSER["Automated DLQ Reparser Daemon"]
        ROUTER -->|Unmatched| DLQ
        DLQ --> DRAIN
        DRAIN --> LAYA
        LAYA --> CODER
        CODER --> REPARSER
        REPARSER -->|Replay Stored Raw Payloads| VRL
    end

    subgraph PRESENT["Presentation & Governance Plane"]
        TUI["Prism Ratatui TUI"]
        WEB["React Sovereign Command Center"]
        GATEKEEPER["Gatekeeper Staging Queue"]
        CODER --> GATEKEEPER
        GATEKEEPER <-->|"Review & Approve (Key 'a')"| TUI
        GATEKEEPER <-->|Review & Approve| WEB
        GATEKEEPER -->|Promote to /rules| VRL
    end
```
</details>

---

## 🔄 2. End-to-End Data Flow with Numbered Steps

<p align="center">
  <img src="images/data_flow_diagram.png" alt="PRISM End-to-End Data Flow" width="100%">
</p>

<details>
<summary><b>🔍 View Data Flow Diagram Mermaid Source</b></summary>

```mermaid
flowchart TD
    classDef step fill:#1e293b,stroke:#3b82f6,stroke-width:2px,color:#f8fafc;
    classDef branch fill:#0f172a,stroke:#f59e0b,stroke-width:2px,color:#fbbf24;
    classDef success fill:#064e3b,stroke:#10b981,stroke-width:2px,color:#a7f3d0;
    classDef alert fill:#450a0a,stroke:#ef4444,stroke-width:2px,color:#fca5a5;

    S1["Step 1: Wire-Speed Ingest\nUDP 514 / TCP / File Tailer\nZero-Copy BytesMut Ring Buffer"]:::step
    S2["Step 2: Cryptographic Provenance\nSIMD BLAKE3 Hash & Immutable ZSTD Vault"]:::step
    S3["Step 3: Forensic Ledger & Witness\nRFC 6962 Merkle Batching & 2-of-3 Ed25519 Quorum"]:::step
    
    ROUTER{"Step 4: SIMD Heuristic Router\nPattern Match (devname, %ASA, CSV)"}:::branch
    
    VRL["Step 5A: In-Memory VRL Engine\nDynamic Rule Bytecode Execution"]:::step
    OCSF["Step 6A: Dynamic OCSF Normalizer\nClass UIDs: 4001, 3001, 5001, 8001"]:::step
    ACCOUNT["Step 7A: Byte-Level Accounting\nClosure Ratio > 95% Verified"]:::step
    SINK["Step 8A: High-Throughput SIEM Sink\nElasticsearch Bulk / Parquet / Kafka"]:::success

    DLQ["Step 5B: Quarantine DLQ\n/vault/dlq_<uuid>.log with Provenance"]:::alert
    DRAIN["Step 6B: Drain3 Template Clustering\nToken Depth Trees & Dynamic Masking"]:::step
    LAYA["Step 7B: Laya ModernBERT (421M)\nZero-Shot Classification & Schema Discovery"]:::step
    CODER["Step 8B: VRL Code Synthesizer\nCompile-Safe Rule & Dry-Run Scorer"]:::step
    GATEKEEPER["Step 9: Gatekeeper HitL Review Queue\nTUI Keyboard Shortcut 'a' or Web Approval"]:::branch
    HOTRELOAD["Step 10: Inotify Hot-Reload & Reparse\nVrlEngine Hot-Reloads -> Flushes DLQ to OCSF"]:::success

    S1 -->|"Zero-Copy Chunks"| S2
    S2 -->|"Leaf Hashes"| S3
    S1 -->|"Raw Byte Slices"| ROUTER

    ROUTER -->|"Fast-Path (Matched)"| VRL
    VRL --> OCSF
    OCSF --> ACCOUNT
    ACCOUNT --> SINK

    ROUTER -->|"Slow-Path (Unknown)"| DLQ
    DLQ --> DRAIN
    DRAIN --> LAYA
    LAYA --> CODER
    CODER --> GATEKEEPER
    GATEKEEPER -->|"Atomic Promote to /rules"| HOTRELOAD
    HOTRELOAD -->|"Reparse Stored DLQ Logs"| VRL
```
</details>

### Step-by-Step Data Flow Execution
1. **Wire-Speed Ingest**: Multiple UDP sockets bind via `SO_REUSEPORT` on port 514 across all CPU cores. Amortized `BytesMut` buffer pools receive raw packet streams with zero heap allocations per packet.
2. **Cryptographic Provenance**: Every raw event payload is immediately hashed using hardware-accelerated **BLAKE3 SIMD** (>3 GB/s) and serialized into compressed ZSTD cold storage blocks.
3. **Forensic Ledger & Quorum**: RFC 6962 Merkle Trees batch 1,000–10,000 leaf hashes per checkpoint. A **2-of-3 Ed25519 Witness Quorum** cryptographically cosigns every Merkle root to satisfy Indian Evidence Act Section 65B legal admissibility.
4. **SIMD Heuristic Router**: A multi-stage `memchr` byte-slice scanner matches vendor signatures (`devname=`, `%ASA-`, `pan_log`) in **~3.07 µs**.
5. **Data Plane (Fast-Path)**:
   - Matched logs execute in-memory compiled **Vector Remap Language (VRL)** bytecode.
   - Dynamic OCSF normalization populates class UIDs: `4001` (Network Activity), `3001` (Authentication), `5001` (File Activity), and `8001` (System Health).
   - **Byte Accounting Engine** verifies `closure_ratio > 0.95` ensuring zero data loss before dispatching to SIEM (Elasticsearch Bulk API, Kafka, Parquet).
6. **Autonomous AI Control Plane (Slow-Path)**:
   - Unrecognized traffic is quarantined to the Dead Letter Queue (`/vault/dlq_<uuid>.log`) preserving raw provenance hashes.
   - **Drain3** clusters unknown logs into parse trees with dynamic wildcards `<*>`.
   - **Laya ModernBERT-large** (421M params) classifies device semantics, infers fields, and assigns target OCSF classes.
   - **VRL Synthesizer** generates compile-safe VRL programs verified by sandbox dry-run execution against real sample payloads.
7. **Gatekeeper & Human-in-the-Loop**:
   - Synthesized rules enter the Gatekeeper Staging Queue (`pending`).
   - Security operators inspect the proposed VRL bytecode, accuracy score, and sample diff inside `prism-tui` or the Sovereign SOC Web Dashboard.
   - Pressing **`a`** atomically promotes the rule to `/rules/*.vrl`.
8. **Inotify Hot-Reload & Reparsing**:
   - `VrlEngine` detects new rules via Linux inotify and atomically swaps the rule pointer.
   - The DLQ Reparser scans historical quarantined logs, converts them to OCSF, and flushes the backlog to SIEM without dropping a single event or requiring a process restart.

---

## 🌐 3. Production Deployment Topology

<p align="center">
  <img src="images/deployment_topology.png" alt="PRISM Production Deployment Topology" width="100%">
</p>

<details>
<summary><b>🔍 View Deployment Topology Mermaid Source</b></summary>

```mermaid
flowchart TB
    subgraph SOURCES["Perimeter & Defense Infrastructure"]
        FW1["FortiGate Core Firewalls"]
        FW2["Cisco ASA VPN Gateways"]
        FW3["Palo Alto Next-Gen Firewalls"]
        EP["Linux / Windows Endpoints (Inotify)"]
        FW1 -->|"UDP 514"| NIC
        FW2 -->|"UDP 514"| NIC
        FW3 -->|"TCP 514"| NIC
        EP -->|"Direct / Inotify"| NIC
    end

    subgraph HOST["PRISM Air-Gapped Appliance (Bare-Metal / Distroless)"]
        NIC["NIC / Kernel Socket Ring (SO_REUSEPORT / 64MB Buffer)"]

        subgraph RUST["PRISM Data Plane (Rust - Zero-Copy)"]
            INGEST["Ingest Workers (Amortized Buffer Pool)"]
            ROUTER["SIMD Heuristic Router (memchr)"]
            VRL_ENG["VRL Normalizer (Hot-Reload)"]
            REPARSER["Automated DLQ Reparser"]
            INGEST --> ROUTER
            ROUTER -->|"Fast Path"| VRL_ENG
            REPARSER -->|"Backlog Replay"| VRL_ENG
        end

        subgraph PROV["Provenance & Cryptographic Engine"]
            SIMD_HASH["SIMD BLAKE3 Hasher"]
            VAULT["ZSTD Compressed Vault"]
            MERKLE_TREE["RFC 6962 Merkle Tree Engine"]
            WITNESS_QUORUM["Section 65B Witness (2-of-3 Ed25519)"]
            INGEST --> SIMD_HASH
            SIMD_HASH --> VAULT
            SIMD_HASH --> MERKLE_TREE
            MERKLE_TREE --> WITNESS_QUORUM
        end

        subgraph PYTHON["PRISM AI Brain (Python Control Plane)"]
            DLQ_STORE["Quarantined DLQ Spool"]
            DRAIN3["Drain3 Tree Clustering"]
            LAYA_BERT["Laya ModernBERT-large (421M Weights)"]
            VRL_SYNTH["VRL Synthesis & Scorer"]
            GATEKEEPER["Gatekeeper Staging Queue"]
            ROUTER -->|"Slow Path (Unknown)"| DLQ_STORE
            DLQ_STORE --> DRAIN3
            DRAIN3 --> LAYA_BERT
            LAYA_BERT --> VRL_SYNTH
            VRL_SYNTH --> GATEKEEPER
            GATEKEEPER -->|"Atomic Write /rules"| VRL_ENG
            GATEKEEPER -->|"Signal Reparse"| REPARSER
        end

        NIC --> INGEST
    end

    subgraph GOVERNANCE["Governance & Presentation Layer"]
        TUI["Prism Ratatui TUI (Local/SSH Console)"]
        WEB["React Sovereign SOC Command Center"]
        GATEKEEPER <-->|"Review & Approve (Key 'a')"| TUI
        GATEKEEPER <-->|"Visual Review & Diff"| WEB
    end

    subgraph EGRESS["SIEM & Forensic Archives"]
        ES["Elasticsearch / OpenSearch Cluster"]
        KAFKA["Enterprise Kafka Broker"]
        PARQUET["Immutable Parquet Cold Storage"]
        AUDIT["Section 65B Certified Forensic Ledger"]
        VRL_ENG -->|"HTTP Bulk Sink"| ES
        VRL_ENG -->|"Event Stream"| KAFKA
        VAULT --> PARQUET
        WITNESS_QUORUM --> AUDIT
    end
```
</details>

---

## ⏱️ 4. Latency Budget & Performance Invariants

| Component | Target Latency | Measured Latency | Verification Harness |
|---|---|---|---|
| Ingest Socket Read | < 5 µs | **~1.8 µs** | `prism-ingest` micro-bench |
| BLAKE3 SIMD Hash | < 2 µs | **~0.4 µs** | `prism-provenance` test |
| Heuristic Router | < 5 µs | **~3.07 µs** | `bench_router_heuristic` (1,000,000 runs) |
| In-Memory VRL Transform | < 15 µs | **~8.2 µs** | `prism-core` VRL engine benchmark |
| **Total Pipeline (p99)** | **< 25 µs** | **~13.47 µs** | **Full Data Plane Ingestion to OCSF** |
| DLQ Hot-Reload Delay | < 50 ms | **< 12 ms** | Inotify atomic pointer swap |
