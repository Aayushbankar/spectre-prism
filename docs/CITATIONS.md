# Citations and References

This project leverages several academic papers, datasets, and open-source tools. We gratefully acknowledge the following works:

1. **Drain (He et al., ICWS 2017)**: He, P., Zhu, J., Zheng, Z., & Lyu, M. R. (2017). "Drain: An Online Log Parsing Approach with Fixed Depth Tree". *IEEE International Conference on Web Services (ICWS)*. Used for O(n) streaming log template mining.
2. **Laya (arXiv:2503.23303)**: ModernBERT-based Log Analysis (2025). Employed as a System 1 classifier for high-speed anomaly detection.
3. **LLaMA.cpp**: Gerganov, G. et al. C++ port of LLaMA for CPU/GPU inference. Used with Q4 GGUF models for rule generation. [GitHub](https://github.com/ggerganov/llama.cpp)
4. **Zstandard (ZSTD)**: Collet, Y., Meta. Real-time data compression algorithm. Used in the cold vault. [Zstandard](https://facebook.github.io/zstd/)
5. **Apache Parquet**: Apache Software Foundation. Columnar storage format. Used for the 180-day retention logs. [Parquet](https://parquet.apache.org/)
6. **rs_merkle**: Rust Merkle Tree library used for Section 65B compliance auditing. [Crates.io](https://crates.io/crates/rs_merkle)
7. **Open Cybersecurity Schema Framework (OCSF v1.9.0)**: Standardized schema for cybersecurity events. [OCSF](https://schema.ocsf.io/)
8. **Vector Remap Language (VRL)**: Datadog. A safe, performant expression language for data transformation. [VRL](https://vector.dev/docs/reference/vrl/)
9. **Ratatui**: Rust Terminal UI library used for the Presentation Plane. [Ratatui](https://ratatui.rs/)
10. **Datasets**: Tested against UNSW-NB15, CIC-IDS-2017, Loghub-2.0, and Zenodo AIT (records/6475510) to validate real-world robustness.
