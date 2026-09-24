# EPS Benchmark & Air-Gap Deployment Plan for PRISM

**Refs:** `docs/requirements/FRS_NFRS.md` (NFRS-01 50k EPS/core, FRS-08 10Hz TUI, NFRS-02 air-gapped),
`docs/datasets/DATASET_PLAN.md`, `HANDOFF_SESSION.md`, `scripts/fetch_small.sh`, `scripts/fetch_large.sh`

---

## 1. EPS Measurement Protocol

### 1.1 Pipeline under test

```text
pcap file --(tcpreplay)--> Suricata / Zeek --(EVE JSON / conn.log)-->
  syslog shim --(UDP 127.0.0.1:514)--> PRISM ingest -> router -> VRL ->
  OCSF 4001 JSON out + Parquet vault + TUI EPS gauge
```

Steps:

1. **Replay:** `tcpreplay -i lo -M <mbps> --loop N <capture.pcap>` replays a
   CIC-IDS-2017 / UNSW-NB15 day-capture at a controlled rate.
2. **Detect:** Suricata (`eve.json`, `event_type: alert|flow`) or Zeek
   (`conn.log`) converts packets to log lines. One flow/alert = one syslog line.
3. **Ship:** a thin shim (`tools/streamer.py` or `socat`) forwards each
   EVE/conn line as one RFC 3164/5424 UDP datagram to `127.0.0.1:514`.
4. **Measure (PRISM side):** ingest timestamps every event at socket recv;
   OCSF serializer timestamps at emit. TUI samples the counter at 10 Hz (FRS-08).
5. **Run matrix:** 10k / 50k / 100k target EPS, 60 s each, 1 core pinned
   (`taskset -c 0`), then N-core scaling run.

### 1.2 Metrics (per run)

| Metric | How captured | Pass bar (NFRS-01) |
| :--- | :--- | :--- |
| Sustained EPS | OCSF lines emitted / wall time | **>= 50,000 EPS on 1 core** |
| Loss | sent (tcpreplay count) − received (vault count) − DLQ count | **0 drops** at 50k EPS |
| p50 / p99 ingest→emit latency | HDR histogram of per-event deltas | p99 < 5 ms (whitepaper target) |
| RSS peak | `/usr/bin/time -v` + TUI sampler, `VmHWM` | **< 50 MB** for Rust ingest+core (NFRS-03) |
| TUI refresh | measured frame interval | **10 Hz ± 1 Hz** (FRS-08) |
| Hash throughput | BLAKE3 GB/s in `prism-provenance` | must exceed line-rate (see §2) |

Report template per run: `eps=<n>, loss=<n>, p50=<ms>, p99=<ms>, rss=<MB>, tui_hz=<hz>`.

### 1.3 Minimal repro commands (connected staging host)

```bash
scripts/fetch_small.sh --dry-run        # verify slice URLs, download nothing
scripts/fetch_small.sh                  # connected staging host only
# replay example (tune -M until TUI reads 50k EPS):
tcpreplay -i lo -M 400 --loop 10 data/full/cic2017/Monday-workingHours.pcap &
suricata -r data/full/cic2017/Monday-workingHours.pcap -l /tmp/eve &
cargo run -p prism-tui  # observe EPS gauge at 10Hz
```

---

## 2. Throughput Math (1 KB/log = 50 MB/s)

Assumptions: mean syslog line ≈ **1,000 bytes** (≈1 KB) on the wire;
NFRS-01 target = **50,000 events/s/core**.

| Quantity | Math | Result |
| :--- | :--- | :--- |
| Goodput per core | 50,000 ev/s × 1,000 B | **50 MB/s (400 Mb/s)** |
| Per event budget (1 core) | 1 s / 50,000 | **20 µs/event** — no inline regex compile, no per-event alloc |
| UDP datagrams | 1 event = 1 datagram → 50k pps | fits loopback; needs `SO_RCVBUF ≥ 8 MiB` (HANDOFF contract) |
| 1500 B MTU framing | 50 MB/s / 1500 B | ≈ 35k frames/s on wire for full pcaps |
| 4-core box | 4 × 50 MB/s | **200 MB/s (~1.6 Gb/s)**, still under one SATA SSD |
| Vault write (zstd ~4:1) | 50 MB/s raw → ~12.5 MB/s Parquet | 180-day CERT-In retention = capacity-planning input, not demo blocker |
| Hash cost (BLAKE3 ~5 GB/s/core) | 50 MB/s ÷ 5 GB/s | **~1% of one core** — hash is not the bottleneck |
| TUI sampling (10 Hz) | 50,000 / 10 | **5,000 events per frame** per core — counter-only, never per-event UI work |

Conclusion: 50k EPS/core is NIC- and allocator-bound, not hash- or disk-bound.
Any run reporting < 50k EPS with RSS > 50 MB points at allocation/regex, per NFRS-01/03.

---

## 3. Two-Minute Demo Slice Spec

Total budget: **120 s** video. Replay at ~175 events/s average so the TUI,
DLQ injection, and HitL approval all fit on screen. All inputs < 500 MB
(`scripts/fetch_small.sh` only — never `fetch_large.sh` on the demo box).

| # | Artifact | Lines | Source script output |
| :--- | :--- | :--- | :--- |
| D1 | `golden-1000.log` | 1,000 | hand-picked Fortinet + Cisco ASA + Palo Alto lines from `DATASET_PLAN.md §3` (exact VRL/OCSF assertions) |
| D2 | `iot23-10k.log` | 10,000 | `data/demo/iot23/iot23-s22-10k.log` (scenario 22 slice; scenario 21 as spare) |
| D3 | `ait-10k.log` | 10,000 | 10k-line head of `data/demo/ait/ait-training-slice.tar.gz` extract |
| — | **Total** | **21,000** | 21,000 ÷ 120 s ≈ **175 EPS avg**, burst to 5k EPS for 10 s to flex the gauge |

Build:

```bash
scripts/fetch_small.sh --data-dir ./data/demo
head -n 10000 data/demo/iot23/iot23-s22-10k.log > /tmp/iot23-10k.log
tar -xzf data/demo/ait/ait-training-slice.tar.gz -O | head -n 10000 > /tmp/ait-10k.log
cat data/golden-1000.log /tmp/iot23-10k.log /tmp/ait-10k.log > data/demo-21k.log
wc -l data/demo-21k.log  # must print 21000
```

Demo timeline (120 s): 0–20 s golden replay + OCSF asserts; 20–70 s IoT/AIT
burst with TUI EPS gauge; 70–100 s alien-format injection → DLQ → Drain3 →
Ollama VRL draft; 100–120 s HitL `[Y]` approval + hot-reload, zero drops.

---

## 4. Offline (Air-Gapped) Bundle Procedure — USB Sneakernet

NFRS-02: the eval/final host has **no internet**. Everything below is prepared
on a connected staging host, checksummed, carried over USB, and re-verified.

### 4.1 Pack (staging, connected)

```bash
scripts/fetch_small.sh --data-dir ./data/demo   # demo slices only
sha256sum data/demo/*/*.log data/demo/*/*.gz data/demo/*/*.json > SHA256SUMS
cargo fetch            # populate $CARGO_HOME registry cache
cargo vendor           # vendor/ dir for --offline build
pip download -r prism-brain/requirements.txt -d ./wheels
ollama pull llama3:8b  # Q4_Instruct ≈ 4.7GB blob; record digest from `ollama list`
tar -czf prism-offline-bundle.tar.gz data/demo SHA256SUMS vendor wheels
sha256sum prism-offline-bundle.tar.gz > prism-offline-bundle.sha256
```

### 4.2 Carry & verify (air-gap host)

```bash
sha256sum -c prism-offline-bundle.sha256
tar -xzf prism-offline-bundle.tar.gz
sha256sum -c SHA256SUMS            # every dataset slice re-verified
cargo build --release --offline    # uses vendor/, no network
pip install --no-index --find-links ./wheels -r prism-brain/requirements.txt
ollama create prism-vrl -f Ollama.Modelfile  # imports the 4.7GB Q4 blob from USB
```

Carry `cargo`/`pip` mirrors and the Ollama blob on the **same USB** as the data;
the air-gap host must never need `curl`/registry access.

### 4.3 Ollama model note

- Pinned model: **`llama3:8b-instruct Q4` ≈ 4.7 GB** on disk. Pull once on
  staging (`ollama pull llama3:8b`), export the blob, verify digest on the
  air-gap host before `ollama create`.
- Runs CPU-only via llama.cpp backend — no GPU required for VRL drafting
  (slow but acceptable: DLQ batches are small, one draft per template).

### 4.4 CPU / RAM fallback — the 3.1 GB-RAM demo box

`llama3:8b Q4` needs ~5–6 GB resident with context — it will **OOM on a
3.1 GB box**. Ordered fallbacks:

1. **Preferred:** `llama3.2:3b-instruct Q4` (~2 GB resident) — still local,
   still NFRS-02 compliant; slightly weaker VRL drafts, HitL catches errors.
2. **Minimal:** `tinyllama:1.1b Q4` (~0.7 GB) or VRL template-fill only
   (no SLM call) — Drain3 + Open Jev still run; System-2 step is stubbed
   with a pre-approved rule file on USB.
3. **Operational:** cap Ollama context (`num_ctx 1024`), `OLLAMA_NUM_PARALLEL=1`,
   add 4 GB swap file, and run `prism-brain` with `--no-llm` flag documented
   in the demo script so the pipeline degrades to Drain3+Jev instead of crashing.

Never attempt `fetch_large.sh` or full IDS2018 on this box — disk and RAM
will both exhaust. Demo slices (§3) only.

---

## 5. License Clearance Table

| Dataset | Source | License / terms | Eval use | Redistribution | Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Loghub / Loghub-2.0 | logpai/loghub (GitHub) | research-use, cite ISSRE'23 paper | ✅ benchmark Drain3 | ⚠️ link, don't re-host | keep 2k slices + citation |
| IoT-23 | Stratosphere IPS Lab | CC BY-NC-SA–style academic terms (verify on index page) | ✅ academic eval | ❌ no commercial re-host | slices only, attribute |
| LANL cyber1 (auth) | Los Alamos CSR | US-gov research release, cite LANL paper | ✅ head-100k eval | ⚠️ link, don't mirror full | head excerpt + citation |
| AIT ADS | Zenodo 6475510 | CC BY 4.0 (verify per-file on record) | ✅ 96MB slice | ✅ with attribution | keep license file in bundle |
| OTRF attack_data (1 technique) | splunk/attack_data (GitHub) | Apache-2.0 / research-use per repo | ✅ single-T1059 JSON | ✅ per Apache-2.0 | keep NOTICE |
| SecRepo samples | secrepo.com | mixed vendor EULAs, sample-only | ✅ weird/tunnel samp. | ⚠️ samples only | do not mirror full captures |
| CIC-IDS-2017 / IDS2018 | UNB CIC / AWS Open Data | research-use, cite CIC papers | ✅ staging EPS runs | ❌ fetch at eval, don't bundle | never ship full pcaps on USB |
| UNSW-NB15 | UNSW ACCS / Cloudstor | research-use, cite Moustafa & Slay | ✅ staging EPS runs | ❌ same as above | same as above |
| llama3:8b weights | Meta / Ollama | Llama 3 Community License | ✅ local inference | ⚠️ follow Meta terms | ship blob on USB, no re-upload |

> Re-verify each license URL at bundle time — dataset hosts relicense.
> Checksums in `scripts/fetch_small.sh` (`SHA256_TBD_PIN_ME`) must be pinned
> on the staging host before any jury/eval run.

---

## 6. Open TODOs

- [ ] Pin all `SHA256_TBD_PIN_ME` digests after one verified connected fetch.
- [ ] Re-pin IoT-23 capture filenames + SecRepo paths (hosts drift; 404 = re-check index).
- [ ] Record `ollama list` digest for the exact `llama3:8b` Q4 blob shipped on USB.
- [ ] Run §1 matrix on staging and paste `eps/loss/p50/p99/rss/tui_hz` table here.
