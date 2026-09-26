# PRISM — Research and References

Full citation list supporting the architecture and design decisions in PRISM (SIH26156).

---

## 1. Algorithmic & Mathematical Foundations

- **[Drain, ICWS 2017]** He et al. — "Drain: An Online Log Parsing Approach with Fixed Depth Tree"
  - IEEE Xplore / DOI: https://doi.org/10.1109/ICWS.2017.13
  - Author PDF (LogPAI): https://arxiv.org/abs/1708.08581
  - Drain3 Implementation: https://github.com/logpai/Drain3
  - *Application:* Compresses high-volume unknown DLQ logs into static templates in near-linear time.

- **[LogCrisp, USENIX ATC 2025]** Wei et al. — "Fast Aggregated Analysis Enabling Two-Phase Pattern Extraction"
  - *Application:* Informs SIMD-accelerated header routing for fast format detection.

- **[KELP, arXiv:2601.00633]** Singh & Ramachandran — "Robust Online Log Parsing Through Evolutionary Grouping Trees"
  - arXiv: https://arxiv.org/abs/2601.00633
  - *Application:* Architectural basis for memory-efficient, zero-copy log ingestion.

- **[Thompson DFA, CACM 1968]** K. Thompson — "Regular Expression Search Algorithm"
  - DOI: https://doi.org/10.1145/363347.363387
  - Further reading: https://swtch.com/~rsc/regexp/regexp1.html
  - *Application:* Guarantees linear-time parsing, avoiding regex-based denial-of-service (ReDoS) vulnerabilities.

- **[simdjson, VLDB 2021]** Langdale & Lemire — "Parsing Gigabytes of JSON per Second"
  - DOI: https://doi.org/10.1007/s00778-020-00650-6
  - arXiv: https://arxiv.org/abs/1902.08318
  - *Application:* Informs high-throughput JSON log parsing strategy.

## 2. Generative AI & Parsing Synthesis

- **[DivLog, ICSE 2024]** Xu et al. — "Log Parsing with Prompt Enhanced In-Context Learning"
  - DOI: https://doi.org/10.1145/3597503.3639144
  - arXiv: https://arxiv.org/abs/2401.07661
  - *Application:* Validates using a local language model to draft parsing rules for unrecognized log formats.

## 3. Cryptographic Integrity & Legal Mandates

- **[BLAKE3, 2020]** O'Connor et al. — "BLAKE3: One function, fast everywhere"
  - IACR Cryptology ePrint 2020/463: https://eprint.iacr.org/2020/463
  - Official Repository: https://github.com/BLAKE3-team/BLAKE3
  - *Application:* Line-rate hashing plus a Merkle tree for tamper-evident log storage.

- **CERT-In Direction No. 20(3)/2022** — Mandates 180-day log retention for Indian entities.
  - Official document: https://www.cert-in.org.in/PDF/CERT-In_Directions_70B_28.04.2022.pdf
  - *Application:* Retention satisfied via Parquet + Zstd compressed storage.

- **Bharatiya Sakshya Adhiniyam, 2023 — Section 63** (successor to Section 65B, Indian Evidence Act 1872)
  - Reference: https://www.indiacode.nic.in/handle/123456789/2263
  - *Application:* Governs admissibility of hash-verified electronic records as legal evidence in India.

## 4. Open Standards & Schema Specifications

- **Open Cybersecurity Schema Framework (OCSF v1.9.0)**
  - Schema Browser: https://schema.ocsf.io/
  - Network Activity Class: https://schema.ocsf.io/1.3.0/classes/network_activity
  - GitHub: https://github.com/ocsf/ocsf-schema

- **Vector Remap Language (VRL, Datadog)**
  - Docs: https://vector.dev/docs/reference/vrl/
  - Playground: https://vrl.dev/
  - Source: https://github.com/vectordotdev/vector

- **Apache Parquet & Zstandard**
  - Parquet: https://parquet.apache.org/
  - Zstandard: https://github.com/facebook/zstd

- **rs_merkle (Rust Merkle Tree Library)**
  - Crates.io: https://crates.io/crates/rs_merkle
  - GitHub: https://github.com/antiguzun/rs_merkle

## 5. Benchmark Datasets

- **UNSW-NB15** (Australian Centre for Cyber Security)
  - Official Portal: https://research.unsw.edu.au/projects/unsw-nb15-dataset
  - Citation: Moustafa, N., & Slay, J. (2015), MilCIS 2015, DOI: https://doi.org/10.1109/MilCIS.2015.7348942

- **CIC-IDS-2017 & CSE-CIC-IDS2018** (Canadian Institute for Cybersecurity)
  - CIC-IDS-2017: https://www.unb.ca/cic/datasets/ids-2017.html
  - CSE-CIC-IDS2018 (AWS Open Data): https://registry.opendata.aws/cse-cic-ids2018

- **LogPAI Loghub-2.0 Benchmark**
  - GitHub: https://github.com/logpai/loghub

- **AIT Log Data Sets**
  - Zenodo Archive: https://zenodo.org/record/6475510

---

*Compiled for Team Spectre GPG — SIH26156, Universal Log Pre-processing Framework (PRISM).*
