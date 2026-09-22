# Critical Academic Research Papers

The PRISM architecture directly implements the findings of these four breakthrough academic papers to solve NTRO's scaling bottlenecks.

## 1. DivLog: Log Parsing with Prompt Enhanced In-Context Learning
*   **Authors:** Junjielong Xu, Ruichun Yang, Yintong Huo, Chengyu Zhang, Pinjia He
*   **Venue:** IEEE/ACM 46th International Conference on Software Engineering (ICSE 2024)
*   **Summary:** Proves that LLMs can automatically infer log schemas and extract parameters using in-context learning without expensive fine-tuning. 
*   **Metrics:** Achieved 98.1% parsing accuracy across 16 datasets.
*   **PRISM Implementation:** Justifies our offline Control Plane. Instead of relying on manual Logstash Grok updates, our local Ollama SLM uses the DivLog methodology to auto-generate Vector Remap Language (VRL) parsing rules.

## 2. Drain: An Online Log Parsing Approach with Fixed Depth Tree
*   **Authors:** Pinjia He, Jieming Zhu, Zibin Zheng, Michael R. Lyu
*   **Venue:** IEEE International Conference on Web Services (ICWS 2017)
*   **Summary:** The foundational algorithm for log clustering. It uses a fixed-depth parse tree to group raw log messages into templates.
*   **Metrics:** Drastically outperforms regex clustering in both speed and accuracy.
*   **PRISM Implementation:** Implemented in `prism-brain/cluster.py`. When 100,000 unknown logs hit our Dead Letter Queue, Drain compresses them into 2 or 3 static templates before passing them to the LLM, preventing token-limit exhaustion.

## 3. LogCrisp: Fast Aggregated Analysis... Enabling Two-Phase Pattern Extraction
*   **Authors:** Junyu Wei, Guangyan Zhang, Junchao Chen, Qi Zhou
*   **Venue:** USENIX Annual Technical Conference (ATC 2025)
*   **Summary:** Demonstrates how to use AVX SIMD (Single Instruction, Multiple Data) instructions to vectorize log queries and ingestion, bypassing standard string manipulation.
*   **Metrics:** 3.8x ingestion speedup compared to standard engines.
*   **PRISM Implementation:** Validates the choice of a Rust-based Data Plane that can utilize vectorized SIMD operations for routing and identifying log structures at wire speed.

## 4. KELP: Robust Online Log Parsing Through Evolutionary Grouping Trees
*   **Authors:** Satyam Singh, Sai Niranjan Ramachandran
*   **Venue:** arXiv Pre-print (2026)
*   **Summary:** Introduces Zero-Copy Interning via Slab allocators to bypass object-oriented pointer indirection during parsing.
*   **Metrics:** Achieves near-perfect accuracy (0.956) in high-entropy sets with massive memory reductions.
*   **PRISM Implementation:** Validates our architectural constraint requiring the Rust Data Plane to operate with a zero-copy memory footprint (target <50MB RAM), directly solving the Elasticsearch/JVM memory bloat currently affecting NTRO.
