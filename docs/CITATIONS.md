# PRISM Citations & References

This document provides a detailed academic and technical bibliography for all algorithms, models, datasets, and libraries used within the PRISM framework for SIH PS 26156.

## Academic Papers & Algorithms

1. **Drain (He et al., ICWS 2017)**
   - *Title*: Drain: An Online Log Parsing Approach with Fixed Depth Tree.
   - *Authors*: Pinjia He, Jieming Zhu, Zibin Zheng, and Michael R. Lyu.
   - *Conference*: 2017 IEEE International Conference on Web Services (ICWS).
   - *Summary*: Drain uses a fixed-depth parse tree to extract log templates in linear time `O(n)`. We use the `drain3` implementation for ultra-fast clustering of alien logs in the Control Plane.

2. **Laya (arXiv:2503.23303, 2025)**
   - *Title*: Laya: A ModernBERT Approach to High-Speed Log Classification.
   - *Authors*: ConvAI Innovations, et al.
   - *Summary*: Laya leverages ModernBERT 421M (Apache 2.0) to achieve semantic triage of log templates, scoring 0.766 against baseline 0.727 accuracy, operating under tight latencies.

3. **LLaMA.cpp (Gerganov et al.)**
   - *Repository*: https://github.com/ggerganov/llama.cpp
   - *Summary*: We use `llama-server` with Q4 GGUF quantized models to generate VRL parsing logic on CPU-only edge devices without relying on cloud APIs.

## Technologies & Libraries

4. **Zstandard (Yann Collet / Meta)**
   - *Repository*: https://github.com/facebook/zstd
   - *Summary*: ZSTD provides real-time compression for our cold-storage Parquet vault, maximizing disk I/O throughput.

5. **Apache Parquet (Apache Software Foundation)**
   - *Summary*: Parquet 2.0 format is used in the Integrity Plane to store logs in columnar format, enabling efficient querying and minimizing storage footprint.

6. **rs_merkle (Rust Merkle Tree library)**
   - *Repository*: https://crates.io/crates/rs_merkle
   - *Summary*: Used to construct cryptographic proofs of log immutability (1<<16 capacity) for Section 65B compliance.

7. **Open Cybersecurity Schema Framework (OCSF v1.9.0)**
   - *Specification*: https://github.com/ocsf/ocsf-schema
   - *Summary*: The unified schema PRISM normalizes all disparate vendor logs into, specifically utilizing Category 4 (Network), Class 4001 (Network Activity).

8. **Vector Remap Language (Datadog)**
   - *Specification*: https://vector.dev/docs/reference/vrl/
   - *Summary*: VRL provides a safe, performant execution engine for mapping raw strings to OCSF JSON.

9. **Ratatui (Rust Terminal UI)**
   - *Repository*: https://github.com/ratatui-org/ratatui
   - *Summary*: A framework for building rich terminal user interfaces, used to construct the 4-pane Engine Room TUI in `prism-tui`.

## Datasets

10. **Cybersecurity Datasets**
    - *Sources*: UNSW-NB15, CIC-IDS-2017, Loghub-2.0, Zenodo AIT (records/6475510).
    - *Summary*: The evaluation and test harnesses utilize heterogeneous vendor logs (Fortinet, Cisco ASA, Palo Alto) drawn from these standard cybersecurity research datasets to validate the 50k EPS throughput.
