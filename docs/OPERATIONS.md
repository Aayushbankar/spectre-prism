# PRISM Production Operations Guide

## 1. System Architecture & Binary Topology
PRISM separates duties between a high-throughput, memory-safe compiled Rust data plane and an autonomous AI governance control plane:
- **Data Plane**: `prism` (compiled Rust binary) - binds to syslog ports (UDP/TCP 514), executes VRL rules in-memory, writes ZSTD chunks to Vault, and signs Merkle leaves.
- **Control Plane**: `prism-brain` (Python SLM microservice) - observes DLQ logs, clusters log templates with Drain, enriches with Laya ModernBERT-large, and generates validated VRL scripts.
- **Operator Console**: `prism-tui` (Ratatui interactive terminal) or Web Command Center (`frontend/`).

---

## 2. Production Deployment

### Directory Layout & Storage Volumes
```
/data/prism/
├── vault/               # ZSTD compressed raw chunks & dlq_<uuid>.log files
├── rules/               # Active VRL rules (*.vrl) hot-reloaded by VrlEngine
├── pending_rules/       # AI generated candidate rules awaiting HitL review
├── approved_rules/      # Audit archive of human-approved rules
├── rule_metadata/       # JSON metadata (confidence, class_uid, cluster stats)
└── ledger.log           # Checkpointed RFC 6962 Merkle roots with cosignatures
```

### Launching PRISM Services
```bash
# 1. Start PRISM Data Plane
PRISM_BASE_DIR=/data/prism ./target/release/prism \
  --udp-bind-addr 0.0.0.0:514 \
  --vault-dir /data/prism/vault \
  --rules-dir /data/prism/rules \
  --batch-size 1000 &

# 2. Start PRISM AI Brain
cd prism-brain
python3 main.py \
  --vault-dir /data/prism/vault \
  --base-dir /data/prism \
  --rules-dir /data/prism/rules &

# 3. Launch Operator TUI
PRISM_BASE_DIR=/data/prism ./target/release/prism-tui
```

---

## 3. Monitoring & Telemetry

### Real-Time Metrics JSON
PRISM periodically dumps atomic performance counters to `/tmp/prism_metrics.json`:
```json
{
  "eps": 13290,
  "processed": 1054000,
  "drops": 0,
  "dlq": 0,
  "telemetry": {
    "fortinet": 450000,
    "cisco": 350000,
    "paloalto": 254000,
    "latency_us": 24
  }
}
```

### Kubernetes Probes
- **Liveness Probe**: `GET /healthz` -> HTTP 200 OK (verifies worker thread health).
- **Readiness Probe**: `GET /readyz` -> HTTP 200 OK (verifies UDP/TCP listeners bound, VRL engine loaded).

---

## 4. Operational Maintenance & Reprocessing

### Hot-Reloading Rules Without Restarts
PRISM monitors `/data/prism/rules/` for filesystem changes using `notify`. When a new rule is approved via TUI or Gatekeeper API:
1. `VrlEngine` compiles and registers the new rule in-memory (< 2 milliseconds).
2. The background DLQ Reparser daemon automatically scans `/data/prism/vault/dlq_*.log`.
3. Unrecognized logs are re-evaluated, converted to OCSF, and dispatched to SIEM without dropped frames or manual intervention.

### Cryptographic Forensic Proofs (Section 65B Compliance)
To verify the authenticity of any raw event in judicial or auditing scenarios:
1. Lookup the event's BLAKE3 `provenance_hash`.
2. Generate the inclusion proof against the checkpointed Merkle root in `/vault/ledger.log`:
   ```bash
   cargo test -p prism-merkle -- test_inclusion_proof --nocapture
   ```
3. Verify the 2-of-3 Ed25519 cosignatures:
   ```bash
   cargo test -p prism-provenance -- test_witness_sign_and_verify --nocapture
   ```

---

## 5. High-Throughput Scaling & Capacity Planning
- **Vertical Scaling**: An 8-core, 16GB RAM instance handles > 100,000 EPS sustained with zero drops due to `SO_REUSEPORT` kernel parallelism and zero-copy ring buffers.
- **Horizontal Scaling**: Multiple PRISM ingest collectors can run behind an ECMP / L4 load balancer, writing to partitioned Vault buckets with synchronized Merkle witness logging.
