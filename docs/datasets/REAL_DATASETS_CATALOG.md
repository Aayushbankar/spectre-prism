# Real Dataset Catalog for framework — Perimeter Logs → OCSF, Drain3, 50k EPS, Air-Gapped SIH Demo

**Project:** PRISM (SIH26156) / framework data plane
**Companion:** `docs/datasets/DATASET_PLAN.md` (strategy), `docs/references/{LOG_FORMAT_STANDARDS,OCSF_SCHEMA,PERIMETER_DEVICES}.md`
**Machine manifest:** `datasets/manifest.json`
**Verification date:** 2026-09-23 (live URL + size + license checked via web search; see Status column)
**Rule:** targets are *real, downloadable, offline-bundlable* corpora. No live-API-only sources.

Target pipeline: perimeter syslog (Fortinet KV / Cisco ASA `%ASA-` / Palo Alto CSV / Zeek / Suricata EVE)
→ VRL parse → OCSF (4001 Network Activity, 4003 DNS, 3002 Auth, 2004 Detection Finding)
→ Drain3 template mining → DLQ (`UNMAPPED` → `dlq.log` → `prism-brain` → hot-reload VRL)
→ 50k+ EPS replay → air-gapped 2-min demo.

---

## 1. Catalog table (verified 2026-09-23)

| # | Name | Live URL (verified) | Size (compressed) | Format | License + offline-bundlable? | Use in PRISM (volume / parser / VRL / DLQ / demo-fit) | Verdict |
|---|---|---|---|---|---|---|---|
| 1 | UNSW-NB15 (ACCS) | `https://research.unsw.edu.au/projects/unsw-nb15-dataset` (canonical) · blobs via `https://cloudstor.aarnet.edu.au/plus/index.php/s/2DhnLGDdEECo4ys` · DOI `10.26190/5d7ac5b1e8485` | CSVs ~2 GB (2,540,044 records, 49 feats); raw PCAP ~100 GB. Status: LIVE | CSV + PCAP + Argus/Bro logs | Academic-research-only, **commercial use prohibited** (Moustafa/Slay). Offline: **YES for CSV slice** (bundle 4× CSV ≈ 2 GB); NO for full 100 GB PCAP | Volume: flow→syslog synthesis for 50k EPS replay (9 attack classes). Parser: weak (flow features, not raw syslog). VRL: synthesize `srcip/dstip/proto/state` → OCSF 4001. DLQ: no. Demo-fit: medium (replay gauge, not vendor logs) | **USE (CSV slice)** |
| 2 | CIC-IDS-2017 (CIC/UNB) | `https://www.unb.ca/cic/datasets/ids-2017.html` (canonical) · direct `http://205.174.165.80/CICDataset/CIC-IDS-2017/Dataset/{MachineLearningCSV,GeneratedLabelledFlows}.zip` · HF mirror `c01dsnap/CIC-IDS2017` (~885 MB parquet, 2.83M rows). Status: LIVE | ML-CSV ~hundreds MB; 8× daily CSVs ~2.8M flows; full PCAPs multi-GB (~50 GB+). Status: LIVE | CSV + PCAP | CIC research use, citation required (Sharafaldin et al. ICISSP 2018); no OSI license file. Offline: **YES for CSV slice** | Volume: BEST cost/benefit EPS stress (Benign + BruteForce/DoS/Heartbleed/Botnet/DDoS/Web/Infiltration). VRL: flow 5-tuple + flags → OCSF 4001. Parser: no ground-truth templates. Demo-fit: high (attack-mix replay) | **USE (CSV slice)** |
| 3 | CSE-CIC-IDS2018 (CSE+CIC, AWS Open Data) | `https://registry.opendata.aws/cse-cic-ids2018/` · bucket `s3://cse-cic-ids2018` (`aws s3 ls --no-sign-request s3://cse-cic-ids2018/`, region `ca-central-1`) · docs `https://www.unb.ca/cic/datasets/ids-2018.html`. Status: LIVE | **~400–450 GB total** (PCAP + per-host logs + CICFlowMeter-V3 CSVs with 80 feats); Kaggle cleaned CSV ~6 GB. Status: LIVE | CSV + PCAP + host/event logs | CIC research use, citation required. Offline: **PARTIAL — CSV slice only** (full 400 GB+ not air-gap-bundlable) | Volume: largest realistic enterprise (50 attackers, 420 PCs + 30 servers, 7 scenarios). VRL: flow→4001. Parser: no. Demo-fit: medium (needs slicing; infiltration-heavy vs 2017) | **SLICE (CSV sample only)** |
| 4 | Loghub (LogPAI, ISSRE'23) | `https://github.com/logpai/loghub` · Zenodo `https://zenodo.org/records/3227177` (v7, **5.1 GB**). Status: LIVE | 5.1 GB (19 sets: HDFS 16 GB / Thunderbird 29.6 GB / Windows 26 GB raw; Linux 2.25 MB, Apache 4.9 MB, OpenSSH 70 MB slices) | Raw `.log` + 2k annotated samples + templates | Free for **research/academic**, must cite + link repo. Offline: **YES, fully bundlable** | Parser: Drain3 accuracy harness (2k labeled lines/set). VRL: **Linux + Apache + OpenSSH map directly to syslog regex/VRL** → OCSF. Volume: low (correctness, not EPS). Demo-fit: high (parser precision numbers for jury) | **USE (parser benchmark)** |
| 5 | Loghub-2.0 (LogPAI, ISSTA'24) | `https://github.com/logpai/loghub-2.0` · Zenodo `https://zenodo.org/records/8275861` (v1, **965.6 MB**, 14 zips). Status: LIVE | 965.6 MB (e.g. HDFS 321.9 MB, Thunderbird 293 MB, Spark 230.7 MB, BGL 86.1 MB; Linux 428 kB, Apache 578 kB, OpenSSH 8.2 MB) | Large-scale annotated logs + templates (14 sets) | Same LogPAI research/academic terms. Offline: **YES, fully bundlable (fits USB)** | Parser: **PRIMARY Drain3 benchmark** (large-scale, harder than v1). VRL: same syslog-adjacent sets. Volume: medium. Demo-fit: high | **USE (primary parser benchmark)** |
| 6 | AIT Log Data Set v2 (AIT-LDSv2) | `https://zenodo.org/records/5789064` (v2.0, Feb 2022). Successor V2.1 on OpenAIRE (`10.5281/zenodo.19483937`). Status: LIVE. ⚠️ Old ID `zenodo.org/records/6475510` (v1.1, cited in DATASET_PLAN §2C) is **DEAD/superseded** — do not use | Per-scenario unpacked **14–39 GB** (8 scenarios × 4–6 days; `_no-pcaps.zip` variants much smaller); total ~200 GB+ with pcaps | Syslog + Apache/DNS/VPN/audit/Suricata/pcap + line-number ground-truth labels | Open (cite Landauer et al. TDSC; record lists CC terms; V2.1 OpenAIRE shows CC BY-NC-SA) — verify record license tab before redistribution. Offline: **YES via `_no-pcaps.zip` slice** | **BEST perimeter-log fit**: multi-host firewall/syslog/VPN/DNS + attack timelines → VRL→OCSF 4001/4003/3002/4014. Parser: Drain3 on real syslog. DLQ: attack-phase labels validate routing. Demo-fit: very high (slice `russellmitchell` 14 GB unpacked, low scan volume) | **USE (top pick — `_no-pcaps` slice)** |
| 7 | AIT Alert Data Set (AIT-ADS) | `https://zenodo.org/records/8263181` (v1, Aug 2023). Status: LIVE | Small (per-scenario `_aminer.json` + `_wazuh.json` + `labels.csv`; **2,655,821 alerts**: 2.29M Wazuh / 307k Suricata / 56k AMiner) | JSON alerts (heterogeneous per-IDS) + attack-phase labels | Open, cite Landauer et al. Offline: **YES, fully bundlable** | DLQ/AI-plane: heterogeneous JSON is ideal **alien-format / aggregation test** (Suricata+Wazuh+AMiner → OCSF 2004). Parser: template-merge demo. Demo-fit: high (alert-correlation story) | **USE (DLQ/alert-plane)** |
| 8 | SecRepo (secrepo.com) | `https://www.secrepo.com/` (JSON-driven link manifest) · mirror `https://github.com/sooshie/secrepo`. Status: LIVE | Small samples (e.g. Squid 24 MB comp / ~200 MB; conn log 522 MB comp / 3 GB ~22M flows; auth.log ~86k lines; honeypot ~994k JSON) | Snort/Suricata/Zeek/iptables/ModSecurity/proxy/auth samples | **CC-BY-4.0** (site header; per-link exceptions noted). Offline: **YES** | VRL: spot-checks (iptables→4001, Zeek→4001, ModSecurity→4002/2004). DLQ: alien seeds (honeypot JSON, proxy). Volume: low. Demo-fit: low-medium (fragmented, unlabelled) | **SLICE (VRL spot-checks)** |
| 9 | OTRF Security-Datasets | `https://github.com/OTRF/Security-Datasets` (1.8k★) · docs `https://securitydatasets.com`. Status: LIVE | Small per-attack zips (KB–100s MB; clone selectively) | Windows/Sysmon + Zeek/Bro + ATT&CK-mapped logs | **MIT** (repo LICENSE; note: README also mentions GPL-3.0 for contributions — treat code GPL, data MIT, verify per-folder) | **Best demo-fit**: per-technique replay (Atomic Red Team ↔ ATT&CK) → OCSF 2004 + DLQ alien JSON → TUI story. VRL: Zeek/Sysmon→OCSF. Volume: low. Offline: **YES** | **USE (demo/story)** |
| 10 | DARPA OpTC (FiveDirections) | `https://github.com/FiveDirections/OpTC-data` (manifest; data on Google Drive) · IEEE DataPort `OpTC.tar.gz` **287.59 GB** (doi `10.21227/edq8-nk52`). Status: LIVE | **~1 TB compressed JSON** (eCAR + eCAR-Bro + Bro, 1000 hosts × 2 weeks) | eCAR JSON + Bro logs + red-team ground truth PDF | **Public domain / Distribution-A unlimited** (DARPA release). Offline: **NO (full)** — slice only | Volume: extreme scale test only. VRL: eCAR→OCSF mapping cost HIGH (custom schema). Parser: no. Demo-fit: poor (fictional IPs, heavy). Use Bro slice →4001 if needed | **SLICE-or-AVOID (Bro slice only; never full)** |
| 11 | LANL Cyber-Security Events (cyber1 + auth + 2017 unified) | `https://csr.lanl.gov/data/cyber1/` (58 days, **1.648B events, ~12 GB comp / 11 GB gz / 89 GB txt**) · `https://csr.lanl.gov/data/auth` (708M auth, **2.3 GB bz2** single file) · `https://csr.lanl.gov/data/2017` (90-day netflow + Win host JSON). Status: LIVE (email-gated `data-fence`, free) | 2.3–12 GB per set | CSV-ish (`auth/proc/flow/dns/redteam.txt.gz`) + JSON host events | **CC0 / public domain** (LANL waiver). Offline: **YES (best-licensed volume set)** | **Top EPS pick**: auth→OCSF 3002, netflow→4001, DNS→4003, redteam.txt = ground truth. 50k EPS replay proven (de-identified, unified U/C IDs). Demo-fit: high (red-team window replay) | **USE (top volume pick)** |
| 12 | MACCDC via Netresec | `https://www.netresec.com/?page=MACCDC` (2010/2011/2012 share links) · index `https://www.netresec.com/?page=PcapFiles`. Status: LIVE | Multi-GB PCAP per year (competition captures) | PCAP only (no parsed logs) | Competition/research use; no OSI license — mirror terms only. Offline: **YES for a small slice** | PCAP→(Zeek/Suricata)→syslog replay for demo color. VRL: indirect (must parse first). Volume: medium. Demo-fit: medium (needs slicing to MBs) | **SLICE (small pcap slice only)** |
| 13 | IoT-23 (Stratosphere Lab) | `https://www.stratosphereips.org/datasets-iot23` · full `https://mcfp.felk.cvut.cz/publicDatasets/IoT-23-Dataset/iot_23_datasets_full.tar.gz` (**21 GB**) · small `.../iot_23_datasets_small.tar.gz` (**8.8 GB**, labels only). Status: LIVE | 21 GB full / 8.8 GB small (23 scenarios, 325M labeled flows, 760M+ packets) | PCAP + **Zeek `conn.log.labeled`** (TSV + 2 label cols) | Free research (Avast-funded; cite García et al. 2020). Offline: **YES (small tar)** | Zeek conn→OCSF 4001 (Mirai/Okiru/Torii + benign Echo/lamp/lock). Parser: Drain3 on conn logs. DLQ: label columns test routing. Demo-fit: medium (IoT/OT angle, not enterprise firewall) | **SLICE (small tar / 1–2 scenarios)** |
| 14 | NSL-KDD | Canonical `https://www.unb.ca/cic/datasets/nsl.html` — **DEAD: page states "this dataset is no longer available"**. Mirrors (unverified history): Kaggle `hassan06/nslkdd`, IEEE DataPort `NSL-KDD.zip` **5.5 MB** (doi `10.21227/425a-3e55`), PLOS figshare `pone.0271436.s003.zip` **6.29 MB**, Zenodo `3768048`. Status: **CANONICAL DEAD / mirrors live** | ~5–6 MB (125,973 train + 22,544 test, 41 feats) | ARFF/TXT (no pcaps, no syslog) | Redistribute-with-citation (Tavallaee et al. 2009) | No raw logs → **no VRL/Drain3/OCSF value**; 1999 lineage, obsolete for 50k EPS or perimeter demo. Baseline-compat only | **AVOID (legacy only)** |

### Dead / superseded IDs (do not fetch)

- **Zenodo `6475510`** — AIT LDS v1.1, cited in `DATASET_PLAN.md` §2C. **Dead/superseded.** Replace with **V2.0 `5789064`** (+ V2.1 `19483937`) and **AIT-ADS `8263181`**.
- **UNB NSL-KDD canonical page** — dataset withdrawn ("no longer available"). Any NSL-KDD use must cite a mirror + pin hash; prefer dropping it.
- **Kaggle "Cybersecurity Threat Detection Logs" (~6M rows)** cited in DATASET_PLAN §2C — synthetic, unlabeled-history, no canonical publisher; **not cataloged** (AVOID for jury rigor).

---

## 2. Verdicts roll-up

**USE (bundle / benchmark / demo):**
1. **AIT-LDSv2 (`5789064`, `_no-pcaps` slice)** — only corpus with real multi-host firewall+syslog+VPN+DNS+Suricata and attack timelines → VRL→OCSF + Drain3 + DLQ in one set.
2. **LANL cyber1/auth (`csr.lanl.gov`, CC0)** — cheapest legal 50k-EPS volume (auth 2.3 GB, full 12 GB) with red-team ground truth.
3. **Loghub-2.0 (`8275861`, 965 MB)** — primary Drain3/parse-accuracy benchmark (citeable ISSRE/ISSTA numbers).
4. **OTRF Security-Datasets (MIT)** — per-technique ATT&CK replay → best 2-minute demo story + alien-JSON DLQ seeds.
5. Supporting USE: UNSW-NB15 CSVs, CIC-IDS-2017 ML-CSV, AIT-ADS JSON, Loghub-v1.

**SLICE (small sample only, never full):** CSE-CIC-IDS2018 (CSV sample; 400 GB+ full is unbunnable), OpTC (Bro slice; 1 TB full impossible), IoT-23 small tar / 1 scenario, MACCDC single pcap (MBs), SecRepo spot samples.

**AVOID:** NSL-KDD (withdrawn + no-log lineage), full OpTC / full IDS2018 (air-gap impossible), Kaggle synthetic threat-logs (history), full UNSW 100 GB PCAP (CSV suffices).

---

## 3. PRISM mapping (how each USE pick drives the pipeline)

| Pick | Volume (50k EPS) | Parser (Drain3) | VRL → OCSF | DLQ / brain | Demo slice (air-gap) |
|---|---|---|---|---|---|
| AIT-LDSv2 | syslog replay at throttle | Drain3 on syslog/Audit/DNS; template count vs labels | syslog/VPN→4001+4014, DNS→4003, auth→3002 | miblockeled/out-of-window lines → `dlq.log` | `russellmitchell _no-pcaps` (14 GB unpacked → cut 50–200 MB window around attack 2022-01-24 03:01–04:39) |
| LANL | **primary EPS source**: `auth.txt.gz` + `flow.txt.gz` replay | N/A (pre-structured) | auth→3002, flow→4001, dns→4003 | malformed/de-id edge lines → DLQ | 1× auth day (~250 MB bz2) + `redteam.txt` window |
| Loghub-2.0 | N/A | **primary benchmark**: Linux/Apache/OpenSSH/HDFS sets, template F1 | Linux/Apache/OpenSSH regexes validate VRL fns | novel-template lines → DLQ seeds | Linux (428 kB) + Apache (578 kB) + OpenSSH (8.2 MB) zips whole |
| OTRF | low-rate story replay | small template sets | Zeek→4001, Sysmon→2004 | JSON variants → alien-format DLQ | 2–3 technique folders (entire, MBs) |
| CIC-IDS-2017 ML-CSV | secondary EPS/attack-mix | N/A | 5-tuple+flags→4001 | N/A | 1 day CSV (e.g. Friday DDoS) |
| UNSW-NB15 CSV | tertiary EPS/diversity (9 classes) | N/A | flow feats→4001 | N/A | `UNSW-NB15_1.csv` subset |
| AIT-ADS | N/A | alert-merge (AMiner/Wazuh/Suricata) | alerts→2004 | **primary alert-DLQ corpus** | per-scenario JSONs whole |

VRL targets (from `PERIMETER_DEVICES.md` + `OCSF_SCHEMA.md`): Fortinet KV (`srcip/srcport/dstip/dstport/action/sentbyte/rcvdbyte/transip`) → 4001 (`activity_id` 1/2/6, `action_id`/`disposition_id`); Cisco `%ASA-6-302014` → 4001 Close; Palo Alto positional CSV → 4001; Zeek conn → 4001; DNS → 4003; VPN/auth → 3002/4014; IDS alerts → 2004.

---

## 4. Fetch plan (offline bundle order, smallest-first)

1. Loghub-2.0 Linux+Apache+OpenSSH zips (~10 MB) → Drain3 harness up.
2. OTRF 2–3 technique folders (MBs) → demo story + DLQ seeds.
3. AIT-ADS per-scenario JSONs → alert-plane DLQ.
4. AIT-LDSv2 `russellmitchell_no-pcaps.zip` → cut attack window (50–200 MB) → VRL→OCSF fixtures.
5. LANL `lanl-auth-dataset-1-00.bz2` (215 MB) + `redteam.txt.gz` → EPS replay.
6. CIC-IDS-2017 `MachineLearningCSV.zip` 1-day slice → attack-mix EPS.
7. UNSW-NB15 1 CSV → diversity EPS.
8. (Optional) IoT-23 1 scenario conn.log.labeled / MACCDC single pcap (MBs) → OT color.

Verify each with checksums (`checksums.txt` / Zenodo md5 / AWS ETag) and pin versions in `datasets/manifest.json`.
