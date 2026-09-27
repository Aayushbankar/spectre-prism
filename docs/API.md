# PRISM REST API & Web Command Center Specification

The PRISM sovereign web console bridges operator actions and classified telemetry directly into the high-performance Rust data plane and Python AI Brain. This document provides complete endpoint documentation with real curl commands and JSON schemas.

---

## 🚀 Running the Frontend Dashboard

### Development Mode (with Live Backend Bridge)
```bash
cd frontend
npm install
npm run dev
```
The console will be accessible at `http://localhost:5173`. In development mode, Vite transparently proxies `/api/real/*` endpoints directly into the active PRISM daemon filesystem and UDP sockets.

### Production Build & Preview
```bash
cd frontend
npm run build
npm run preview -- --port 4173
```
Production build bundles TypeScript, Tailwind CSS, and Lucide icons into zero-dependency static assets in `dist/`.

### Direct View Deep-Linking
Deep-link directly to specific views or themes using URL query parameters:
- **SOC Command Center**: `http://localhost:5173/?view=command`
- **Pipeline Topology & Sankey**: `http://localhost:5173/?view=pipeline` (or `?view=flow`)
- **Engine Telemetry & SLAs**: `http://localhost:5173/?view=analytics`
- **Threat Vector Radar**: `http://localhost:5173/?view=tactical` (or `?view=threats`)
- **Byte Accounting & Vault**: `http://localhost:5173/?view=accounting`
- **Gatekeeper HitL Staging**: `http://localhost:5173/?view=gatekeeper`
- **Dead Letter Queue**: `http://localhost:5173/?view=dlq`
- **VRL Interactive Sandbox**: `http://localhost:5173/?view=vrl`
- **Themes**: `?theme=dark` (default), `?theme=cyber`, `?theme=light`

---

## 📡 API Endpoint Reference

### 1. `GET /api/real/status`
Retrieves live data plane performance metrics, packet processing counters, DLQ volume, and active AI status.

#### Example Request:
```bash
curl -s http://localhost:5173/api/real/status | jq .
```

#### Example Response:
```json
{
  "eps": 13290,
  "processed": 100000,
  "drops": 0,
  "dlq": 0,
  "telemetry": {
    "fortinet": 65000,
    "cisco": 25000,
    "paloalto": 10000,
    "latency_us": 13
  },
  "ledger_root": "5ade96bb788957a6e147cf7914bdcfb6038e2d427d11bf430b501ab1a129d20c",
  "ledger_count": 12,
  "ai_status": "online"
}
```

---

### 2. `GET /api/real/dlq`
Retrieves quarantined anomalous payloads currently awaiting autonomous AI clustering and template synthesis.

#### Example Request:
```bash
curl -s http://localhost:5173/api/real/dlq | jq .
```

#### Example Response:
```json
[
  {
    "id": "dlq_4f92a10b",
    "timestamp": "2026-09-27T12:57:41.104Z",
    "raw": "<13>Sep 27 12:57:41 firewall kernel: [DROPPED] IN=eth0 OUT= MAC=00:0c:29:.. SRC=192.168.1.100 DST=10.0.0.1 PROTO=TCP SPT=49152 DPT=443",
    "reason": "HEURISTIC_ROUTER_UNMATCHED",
    "hash": "c74829be34d58091a182faec081b3724c949e29a9972b21e89cf10928a57bf4a"
  }
]
```

---

### 3. `GET /api/real/rules`
Lists all candidate AI-synthesized rules pending operator authorization as well as active production rules.

#### Example Request:
```bash
curl -s http://localhost:5173/api/real/rules | jq .
```

#### Example Response:
```json
[
  {
    "id": "rule_firewall_8a8ab357",
    "device_type": "Firewall",
    "state": "pending",
    "vrl_code": ".class_uid = 4001\n.activity_id = 1\n.severity_id = 1\nparsed, err = parse_regex(.message, r'(?P<src>[0-9.]+)')\nif err == null { .src_endpoint.ip = parsed.src }",
    "created_at": "2026-09-27T12:57:41Z",
    "ocsf_class": 4001
  }
]
```

---

### 4. `POST /api/real/rules/approve`
Promotes a pending Gatekeeper rule to production `/rules/*.vrl`. Immediately triggers inotify hot-reload on the compiled Rust VRL engine and initiates automated re-parsing of the DLQ backlog.

#### Example Request:
```bash
curl -s -X POST http://localhost:5173/api/real/rules/approve \
  -H "Content-Type: application/json" \
  -d '{"rule_id": "rule_firewall_8a8ab357"}' | jq .
```

#### Example Response:
```json
{
  "status": "success",
  "rule_id": "rule_firewall_8a8ab357",
  "deployed_path": "/tmp/prism/rules/firewall_rule_firewall_8a8ab357.vrl",
  "reparsed_events": 10
}
```

---

### 5. `POST /api/real/stream/inject`
Injects synthetic or recorded perimeter log frames directly into the UDP Ingestion Plane socket (`127.0.0.1:15514`).

#### Example Request:
```bash
curl -s -X POST http://localhost:5173/api/real/stream/inject \
  -H "Content-Type: application/json" \
  -d '{
    "vendor": "fortinet",
    "count": 50,
    "raw_log": "<134>date=2024-01-15 time=08:23:41 devname=\"FGT-DC-01\" logid=\"0000000013\" type=\"traffic\" subtype=\"forward\" level=\"notice\" srcip=10.10.20.45 dstip=203.0.113.25 proto=6 action=\"accept\" sentbyte=15234 rcvdbyte=892451\n"
  }' | jq .
```

#### Example Response:
```json
{
  "status": "injected",
  "count": 50,
  "target": "127.0.0.1:15514"
}
```

---

### 6. `POST /api/real/vrl/test`
Compiles and executes candidate VRL bytecode against a test log payload in a sandboxed runtime, returning projected OCSF attributes and byte accounting statistics.

#### Example Request:
```bash
curl -s -X POST http://localhost:5173/api/real/vrl/test \
  -H "Content-Type: application/json" \
  -d '{
    "raw_payload": "<134>date=2024-01-15 time=08:23:41 devname=\"FGT-DC-01\" srcip=10.10.20.45 dstip=203.0.113.25 action=\"accept\"",
    "vrl_code": ".class_uid = 4001\n.src_endpoint.ip = \"10.10.20.45\"\n.dst_endpoint.ip = \"203.0.113.25\"\n.disposition = \"Allowed\""
  }' | jq .
```

#### Example Response:
```json
{
  "success": true,
  "ocsf_output": {
    "class_uid": 4001,
    "class_name": "Network Activity",
    "src_endpoint": { "ip": "10.10.20.45" },
    "dst_endpoint": { "ip": "203.0.113.25" },
    "disposition": "Allowed"
  },
  "execution_time_ns": 4200,
  "closure_ratio": 97.4
}
```

---

### 7. `POST /api/real/accounting`
Evaluates byte-level extraction completeness for a normalized event, categorizing raw bytes into `FIELD`, `LITERAL`, and `RESIDUE` according to PS-26156 Clause 3.1.

#### Example Request:
```bash
curl -s -X POST http://localhost:5173/api/real/accounting \
  -H "Content-Type: application/json" \
  -d '{
    "raw_length": 250,
    "field_bytes": 185,
    "literal_bytes": 55,
    "residue_bytes": 10
  }' | jq .
```

#### Example Response:
```json
{
  "closure_ratio": 0.96,
  "closure_percentage": "96.0%",
  "meets_guarantee": true,
  "threshold": 0.95,
  "breakdown": {
    "field_bytes": 185,
    "literal_bytes": 55,
    "residue_bytes": 10
  }
}
```

---

### 8. `GET /api/real/witness`
Retrieves Section 65B forensic provenance records, including the current RFC 6962 Merkle root and 2-of-3 Ed25519 cosignatures.

#### Example Request:
```bash
curl -s http://localhost:5173/api/real/witness | jq .
```

#### Example Response:
```json
{
  "merkle_root": "5ade96bb788957a6e147cf7914bdcfb6038e2d427d11bf430b501ab1a129d20c",
  "quorum_type": "2-of-3 Ed25519 Cosigning",
  "legally_admissible": true,
  "indian_evidence_act_section": "65B(4)",
  "witnesses": [
    {
      "witness_id": "witness-node-alpha",
      "public_key_fingerprint": "ed25519:a3f901bc",
      "signature": "3045022100e4a...9f01",
      "timestamp": "2026-09-27T12:57:41Z"
    },
    {
      "witness_id": "witness-node-beta",
      "public_key_fingerprint": "ed25519:88c21de4",
      "signature": "304502206bc12...88aa",
      "timestamp": "2026-09-27T12:57:41Z"
    }
  ]
}
```

---

## ⚡ Real-Time WebSocket Streaming Protocol

PRISM provides a low-latency, full-duplex WebSocket gateway for classified SOC consoles, real-time SIEM forwarders, and streaming telemetry dashboards.

### Endpoint URL
- **Vite Dev / Proxy Bridge**: `ws://localhost:5173/ws/telemetry`
- **Native PRISM Server**: `ws://127.0.0.1:8080/ws/telemetry`

---

### Subscription Channels

Clients can subscribe to one or more real-time channels:

| Channel | Frequency | Content Description |
|---|---|---|
| `telemetry` | 10 Hz (every 100ms) | Real-time EPS, p99 parse latency (µs), drops, buffer gauges |
| `events` | Event-driven (push) | Normalized OCSF Class 4001/3001 events with BLAKE3 provenance |
| `dlq` | Instantaneous | Quarantine alerts for unparsed perimeter payloads |
| `hitl` | Autonomous trigger | Gatekeeper candidate rule notifications awaiting operator approval |

---

### Client Subscription Handshake

Upon opening the WebSocket connection, the client sends a JSON subscription frame:

```json
{
  "action": "subscribe",
  "channels": ["telemetry", "events", "dlq", "hitl"],
  "filter": {
    "min_severity": "High",
    "vendors": ["Fortinet", "Cisco ASA", "Palo Alto"]
  }
}
```

#### Server Acknowledgment:
```json
{
  "type": "subscribed",
  "channels": ["telemetry", "events", "dlq", "hitl"],
  "session_id": "ws_sess_89f0a21b",
  "timestamp": 1727443800000
}
```

---

### Streaming Message Payloads

#### 1. Telemetry Frame (`channel: telemetry`)
```json
{
  "type": "telemetry_update",
  "timestamp": 1727443800100,
  "data": {
    "eps": 13290,
    "processed_total": 1054000,
    "dropped_total": 0,
    "dlq_active": 0,
    "latency_p99_us": 13.47,
    "buffer_utilization_pct": 24.2,
    "merkle_root": "5ade96bb788957a6e147cf7914bdcfb6038e2d427d11bf430b501ab1a129d20c",
    "vendor_rates": {
      "fortinet_eps": 5800,
      "cisco_eps": 4100,
      "paloalto_eps": 3390
    }
  }
}
```

#### 2. Normalized Event Frame (`channel: events`)
```json
{
  "type": "ocsf_event",
  "category_uid": 4,
  "class_uid": 4001,
  "class_name": "Network Activity",
  "activity_id": 1,
  "activity_name": "Allow",
  "severity_id": 2,
  "src_endpoint": { "ip": "198.51.100.42", "port": 52341 },
  "dst_endpoint": { "ip": "10.0.5.50", "port": 443 },
  "metadata": {
    "provenance_hash": "b3_5ade96bb788957a6e147cf7914bdcfb6",
    "merkle_leaf_index": 48201,
    "vault_chunk": "/vault/chunk_20260927_0042.parquet",
    "vrl_rule": "fortigate_ocsf4001.vrl",
    "sec_65b_certified": true
  }
}
```

#### 3. DLQ Quarantine Notification (`channel: dlq`)
```json
{
  "type": "dlq_alert",
  "dlq_id": "dlq_4f92a10b",
  "timestamp": "2026-09-27T12:57:41.104Z",
  "reason": "HEURISTIC_ROUTER_UNMATCHED",
  "raw_payload_preview": "<13>Sep 27 12:57:41 firewall kernel: [DROPPED] IN=eth0...",
  "clustering_status": "queued_for_drain3"
}
```

#### 4. Gatekeeper HitL Notification (`channel: hitl`)
```json
{
  "type": "gatekeeper_candidate",
  "rule_id": "rule_firewall_8a8ab357",
  "device_type": "Firewall",
  "suggested_ocsf_class": 4001,
  "confidence_score": 0.96,
  "status": "pending_approval",
  "hotkeys": { "approve": "a", "reject": "r", "preview": "p" }
}
```

---

### Heartbeat & Reconnection Protocol
- **Ping / Pong**: The server transmits `{ "type": "ping" }` every 5,000ms. The client must reply with `{ "type": "pong" }` within 2,500ms to keep the connection alive.
- **Auto-Reconnect**: Recommended exponential backoff (`initial: 500ms`, `max: 5000ms`, `factor: 1.5`) when running behind unstable field relays.

---

### Client Implementation Examples

#### JavaScript / TypeScript (Browser SOC Console)
```typescript
const socket = new WebSocket('ws://localhost:5173/ws/telemetry');

socket.onopen = () => {
  console.log('Connected to PRISM WebSocket Gateway');
  socket.send(JSON.stringify({
    action: 'subscribe',
    channels: ['telemetry', 'events', 'dlq', 'hitl']
  }));
};

socket.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.type === 'telemetry_update') {
    updateGauges(msg.data.eps, msg.data.latency_p99_us);
  } else if (msg.type === 'ocsf_event') {
    appendLedgerRow(msg);
  } else if (msg.type === 'gatekeeper_candidate') {
    notifyOperatorModal(msg);
  }
};
```

#### Python (`websockets` Client)
```python
import asyncio
import json
import websockets

async def stream_prism_telemetry():
    uri = "ws://127.0.0.1:8080/ws/telemetry"
    async with websockets.connect(uri) as ws:
        await ws.send(json.dumps({
            "action": "subscribe",
            "channels": ["telemetry", "dlq"]
        }))
        async for message in ws:
            payload = json.loads(message)
            print(f"[{payload.get('type')}] EPS: {payload.get('data', {}).get('eps')}")

if __name__ == "__main__":
    asyncio.run(stream_prism_telemetry())
```

