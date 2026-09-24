# Reproducibility & Prerequisites

This document outlines the deployment configurations for various target platforms, guaranteeing PRISM can run in air-gapped SIH evaluations, high-end GPU workstations, and standard laptops for development.

## Per-Device Matrix

| Device | OS | CPU/GPU | RAM | Install | Config | Air-gapped | Container |
|---|---|---|---|---|---|---|---|
| Laptop CPU-only (your device) | Arch Linux 7.1.8 | CPU 8-core no GPU | 16GB | `pip install -r requirements.txt` | `config.yaml` device:cpu coder.enabled:false | Download wheels + `pip download` | `docker compose up` |
| GPU Workstation | Ubuntu 22.04 | NVIDIA 4090 | 32GB | `pip install -r requirements.txt[all]` | `device:gpu` | same + `ollama pull llama3` | `docker compose up --gpus all` |
| Air-gapped NTRO | RHEL 9 | CPU | 64GB | `pip download` + transfer | `device:cpu` `air-gapped:true` | `podman load` | `podman compose up` |

### Core Toolchain
* **Rust:** `rustc 1.97.1` (Cargo Workspace)
* **Python:** `3.12` (Control Plane)
* **LLM (Optional):** `Ollama` for local autonomous rule generation
* **Packages:** `watchdog`, `drain3` (Base requirements)

### Verification
Ensure your dependencies are successfully linked by running:
```bash
pip show drain3
```
This should output the package metadata if successfully installed.

### Edge AI Installation
To run the L3 Control Plane offline with ML extraction, compile and run `llama-server` and `laya`:
```bash
# llama-server
cmake -DGGML_NATIVE=ON .
make llama-server
/home/legion/.local/bin/llama-server --model /tmp/models/llama3.gguf

# laya
pip install laya==0.3.16
huggingface-cli download convaiinnovations/laya
```
