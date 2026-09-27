# PRISM Detailed Engineering & Production Deployment Guide

## 1. Production Deployment Topology

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

    subgraph PRESENTATION["Presentation & Operator Control"]
        TUI["Prism Ratatui TUI (Local/SSH Console)"]
        WEB["React Sovereign SOC Command Center"]
        GATEKEEPER <-->|"Approve / Reject ('a')"| TUI
        GATEKEEPER <-->|"Approve / Reject"| WEB
    end

    subgraph DOWNSTREAM["Downstream SIEM & Storage Infrastructure"]
        ES["Elasticsearch / OpenSearch Cluster"]
        KAFKA["Enterprise Kafka Broker"]
        PARQUET["Immutable Parquet Cold Storage"]
        AUDIT["Section 65B Certified Forensic Ledger"]
        VRL_ENG -->|"Bulk OCSF JSON"| ES
        VRL_ENG -->|"Stream Events"| KAFKA
        VAULT --> PARQUET
        WITNESS_QUORUM --> AUDIT
    end
```
</details>

### Single-Node Bare-Metal Topology (ADR-01)
```
[Perimeter Firewalls / Gateways (Fortinet, Cisco ASA, Palo Alto, Linux Syslog)]
       │  UDP/TCP port 514 (Kernel SO_REUSEPORT)
       ▼
┌────────────────────────────────────────────────────────────────────────┐
│  PRISM High-Performance Host (8+ Cores, 16GB+ RAM, NVMe Storage)       │
│                                                                        │
│  ┌─────────────────────────────┐      ┌──────────────────────────────┐ │
│  │     prism (Rust Binary)     │      │   prism-brain (Python SLM)   │ │
│  │   • Ingestion Workers (514) │      │   • Drain3 Tree Clustering   │ │
│  │   • SIMD Heuristic Router   │◄────►│   • Laya ModernBERT (421M)   │ │
│  │   • VRL Normalizer (Hot)    │      │   • VRL Synthesizer & Scorer │ │
│  │   • BLAKE3 Merkle Vault     │      │   • Gatekeeper HitL Queue    │ │
│  │   • Automated DLQ Reparser  │      │   • Inotify Watcher Loop     │ │
│  └──────────────┬──────────────┘      └──────────────┬───────────────┘ │
│                 │                                    │                 │
│                 └──────────────────┬─────────────────┘                 │
│                                    ▼                                   │
│                         /data/prism (NVMe / SSD)                       │
│                         ├── /vault (ZSTD Compressed Chunks)            │
│                         ├── /rules (*.vrl Dynamic Rules)               │
│                         ├── /rule_metadata (*.json States)             │
│                         └── /vault/ledger.log (Merkle 65B Checkpoints) │
└────────────────────────────────────┬───────────────────────────────────┘
                                     │ HTTP Bulk Sink (OCSF JSON)
                                     ▼
                [Elasticsearch / OpenSearch / Kafka / Parquet]
```

---

## 2. End-to-End Data Pipeline Architecture

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

    S1 --> S2
    S2 --> S3
    S1 --> ROUTER

    VRL["Step 5a: Fast Path - In-Memory VRL\nHot-Reload Bytecode Execution"]:::step
    OCSF["Step 6a: Dynamic OCSF Normalization\nClasses 4001, 3001, 5001, 8001"]:::step
    ACCOUNT["Step 7a: Byte Accounting Verification\nClosure Ratio > 95% Guarantee"]:::success
    SINK["Step 8a: High-Speed Egress Sink\nElasticsearch Bulk / Parquet Vault"]:::step

    ROUTER -->|Matched Signature| VRL
    VRL --> OCSF
    OCSF --> ACCOUNT
    ACCOUNT --> SINK

    DLQ["Step 5b: Slow Path - DLQ Isolation\nQuarantine Raw Payload (/vault/dlq_*.log)"]:::alert
    DRAIN["Step 6b: Autonomous Drain3 Clustering\nTree Depth & Parameter Masking"]:::step
    LAYA["Step 7b: Laya ModernBERT-large (421M)\nSchema Classification & VRL Synthesis"]:::step
    GATEKEEPER["Step 8b: Gatekeeper Staging Queue\nDry-Run Validation & Candidate Rules"]:::branch
    HITL["Step 9b: Operator HitL Authorization\nRatatui TUI / React SOC (Key 'a')"]:::success
    REPARSE["Step 10b: Hot-Reload & DLQ Reparsing\nInotify Reload + Backlog Replay into OCSF"]:::step

    ROUTER -->|Unmatched / Anomaly| DLQ
    DLQ --> DRAIN
    DRAIN --> LAYA
    LAYA --> GATEKEEPER
    GATEKEEPER <-->|Audit & Approve| HITL
    HITL -->|Promote Rule| REPARSE
    REPARSE -->|Replay Quarantined Logs| VRL
```
</details>

---

## 3. Linux Kernel Optimization & Socket Tuning

To guarantee lossless ingestion at >100,000 EPS without buffer overflow, apply the following kernel socket parameters:

```bash
# /etc/sysctl.d/99-prism.conf
# Increase max receive socket buffer sizes to 64MB
net.core.rmem_max = 67108864
net.core.rmem_default = 33554432

# Increase network device backlog queue
net.core.netdev_max_backlog = 500000

# Increase maximum file descriptors
fs.file-max = 2097152

# Disable slow start on idling connections
net.ipv4.tcp_slow_start_after_idle = 0
```
Apply with:
```bash
sudo sysctl --system
```

---

## 4. Air-Gapped High-Security Operations

PRISM is purpose-built to operate in strictly air-gapped environments without any external internet connectivity.

### Prerequisites Checklist:
1. **Model Cache**: Pre-download Laya weights to `/opt/models/convaiinnovations/laya`.
2. **Container Image**: Export distroless image via `docker save prism:latest | gzip > prism-airgap.tar.gz`.
3. **Execution**:
   ```bash
   docker run --network none \
     -v /data/vault:/tmp/prism/vault \
     -v /data/rules:/tmp/prism/rules \
     -p 514:514/udp \
     prism:latest
   ```

---

## 5. Kubernetes Production Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prism-collector
  namespace: sovereign-soc
spec:
  replicas: 3
  selector:
    matchLabels:
      app: prism
  template:
    metadata:
      labels:
        app: prism
    spec:
      containers:
      - name: prism-data-plane
        image: prism:latest
        imagePullPolicy: IfNotPresent
        resources:
          limits:
            cpu: "4"
            memory: 8Gi
          requests:
            cpu: "2"
            memory: 4Gi
        ports:
        - containerPort: 514
          protocol: UDP
          name: syslog-udp
        volumeMounts:
        - name: vault-storage
          mountPath: /data/prism/vault
        - name: rules-storage
          mountPath: /data/prism/rules
      volumes:
      - name: vault-storage
        persistentVolumeClaim:
          claimName: prism-vault-pvc
      - name: rules-storage
        configMap:
          name: prism-rules-config
```
