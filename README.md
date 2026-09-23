# PRISM 🔮
**Programmable Routing & Intelligent Semantic Mapper**

*Team Spectre | Smart India Hackathon (SIH26156) | NTRO - Universal Log Pre-processing Framework*

## 📖 Overview
PRISM is a high-performance, next-generation log pre-processing framework designed to handle billions of events per day. It solves the Latency-Cost Trilemma by decoupling throughput from intelligence using a **4-Plane Architecture**:
- **Data & Ingestion Plane (Rust):** Gbps zero-copy network ingestion and deterministic parsing (VRL).
- **Control Plane (Python/AI):** Air-gapped offline AI (Drain3 + Open Jev + Ollama) for auto-generating parsers for unknown logs.
- **Integrity Plane (Rust):** BLAKE3 hashing and Merkle-tree Parquet vaults for 180-day forensic court compliance.
- **Presentation Plane (Rust/Web):** Terminal UI "Engine Room" and Elasticsearch SIEM integrations.

## 📚 Team Documentation & Architecture
Before writing any code, please read the official architecture blueprints. This is a strict Waterfall + Iterative project.

1. **[PRISM Whitepaper & Architecture](docs/PRISM_WHITEPAPER.md)** *(Start Here)*
2. **[Component Module Design](docs/architecture/COMPONENT_DESIGN.md)**
3. **[Data Dictionary & IPC Contracts](docs/architecture/DATA_DICTIONARY_AND_IPC.md)**
4. **[Team Task Allocation](docs/TEAM_TASK_BREAKDOWN.md)**
5. **[Teamwork AI Architectural Evaluation](teamwork_eval/ARCHITECTURAL_COMPARISON.md)**

## ⚙️ Prerequisites
To develop and run PRISM locally, ensure you have the following installed:
- **Rust** (`cargo`, `rustc`)
- **Python 3.12+** (with `pip`)
- **Docker & Docker Compose** (for Elasticsearch/Kibana sink testing)
- **Ollama** (for local System 2 AI execution)

## 🚀 Getting Started
```bash
# Clone the repository
git clone https://github.com/Aayushbankar/spectre-prism.git
cd spectre-prism
```
*(Note: codebase directories `prism-common`, `prism-ingest`, `prism-provenance`, `prism-core` initialized. `prism-tui` deferred)*

## 🛠️ Work Allocation
Check the WhatsApp group poll to claim your module category. Once confirmed, refer to the [Team Task Breakdown](docs/TEAM_TASK_BREAKDOWN.md) for your exact deliverables and begin development in your designated module.
