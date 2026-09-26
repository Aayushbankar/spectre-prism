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
- No redistribution of raw corpora: corpora can be fetched via `./tools/fetch_datasets.sh`.
- PRISM zero-copy architecture only processes and extracts cryptographic provenance, never persisting raw corpora beyond encrypted, immutable Vault retention with ZSTD chunking.
