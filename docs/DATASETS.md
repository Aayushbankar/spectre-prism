# Dataset Disclaimer

## Perimeter Dataset Sources
All corpora are PUBLICLY AVAILABLE research datasets:
- **Honeynet Project** (Scan of the Month 30, 34, Dragon NIDS) - [honeynet.org](https://honeynet.org)
- **Loghub 2.0** (Apache, OpenSSH, Linux, Proxifier, Blue Coat, Squid) - Zenodo 8196385 ([doi:10.5281/zenodo.8196385](https://doi.org/10.5281/zenodo.8196385))
- **MACCDC 2012** (Zeek conn.log) - [secrepo.com](https://www.secrepo.com)
- **LogPAI Benchmark** (HDFS, BGL, OpenStack, Spark, Windows, Zookeeper) - [github.com/logpai/loghub](https://github.com/logpai/loghub)

## No Private/Sensitive Data
- All IPs are RFC 5737 documentation addresses (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.100.0/24`) or private RFC 1918 subnets (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`).
- All hostnames are synthetic (`FGT-DC-01`, `FW-CORE-01`, `SRV-EDGE-02`, etc.).
- Absolutely no real credentials, passwords, tokens, PII, or proprietary intelligence logs are contained within or produced by PRISM.

## Usage & Compliance
- Research and educational use only in accordance with SIH PS-26156 evaluation parameters.
- No redistribution of raw corpora: corpora can be fetched via `./fetch_datasets.sh` (or `./tools/fetch_datasets.sh`).
- PRISM zero-copy architecture only processes and extracts cryptographic provenance, never persisting raw corpora beyond encrypted, immutable Vault retention with ZSTD chunking.

---

## 📥 Automated Fetch Script (`fetch_datasets.sh`)

PRISM provides an automated, idempotent bash utility to retrieve and stage all public research corpora:

```bash
# Standard usage (fetches into ./realdata):
./fetch_datasets.sh

# Specify custom target directory:
./fetch_datasets.sh --dir /data/perimeter_logs

# Fetch all available full research corpora:
./fetch_datasets.sh --all
```

### Command-Line Arguments

| Flag | Description | Default |
|---|---|---|
| `--dir <path>` | Target directory for downloaded log files | `realdata` |
| `--all` | Retrieve all full corpora including large tarballs | Off |
| `--no-bluecoat` | Skip downloading Blue Coat proxy logs | Off |
| `--no-full-zeek` | Skip full-size Zeek conn.log.gz download | Off |

### Offline / Air-Gapped Operation

In air-gapped or offline evaluation environments, `fetch_datasets.sh` automatically checks for local cached assets in `data/real_corpora/` before making network requests. If offline and no local cache is found, synthetic RFC 5737 compliant test fixtures are automatically generated, ensuring pipelines never block.

### Corpora Inventory & Origin Mapping

| Target Log File | Upstream Source / DOI | Device / Format | Role in PRISM Evaluation |
|---|---|---|---|
| `iptables.log` | Loghub 2.0 (Linux Syslog) | Linux Netfilter / Kernel | Firewall drop slow-path & DLQ triage |
| `snort.log` | Loghub 2.0 (HDFS/Alerts) | Snort NIDS Alert | Network Intrusion detection parsing |
| `dragon-nids.log`| Loghub 2.0 (OpenStack) | Enterasys Dragon NIDS | Alert signature clustering |
| `apache-access.log`| Loghub 2.0 (Apache) | Apache HTTP Server CLF | Web server access normalization |
| `OpenSSH.full.log`| Zenodo 8196385 | OpenSSH Auth Log | Authentication Class 3001 |
| `zeek-conn.log` | MACCDC 2012 (SecRepo) | Zeek / Bro Network Flow | Flow & session activity parsing |
| `squid-access.log`| Loghub 2.0 (Proxifier) | Squid Proxy Log | Egress proxy connection activity |
| `cisco_asa.log` | Synthetic Perimeter Corpus | Cisco ASA Syslog | Class 4001 Fast-path wire-speed parse |
| `fortinet_fortigate.log`| Synthetic Perimeter Corpus | FortiGate Key-Value Syslog| Class 4001 Fast-path wire-speed parse |
| `paloalto_threat.log`| Synthetic Perimeter Corpus | Palo Alto PAN-OS CSV | Class 4001 Threat log wire-speed parse |

