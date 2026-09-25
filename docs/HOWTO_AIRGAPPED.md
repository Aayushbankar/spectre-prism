# HOW-TO: Air-Gapped Deployment

PRISM is designed explicitly for NTRO and highly secure SIH environments where no internet connection is available (Section 65B compliance).

## 1. Preparation on an Internet-Connected Machine
Before moving to the air-gapped site, download all necessary packages and container images.

### Python Dependencies (Wheels)
Download all wheels matching the target OS (`manylinux` for typical Linux servers):
```bash
pip download --platform manylinux2014_x86_64 --only-binary=:all: -r requirements.txt -d ./offline-wheels
```

### Docker/Podman Images
Pull the required Elasticsearch and Kibana images, then export them to a tarball:
```bash
docker pull docker.elastic.co/elasticsearch/elasticsearch:8.13.4
docker pull docker.elastic.co/kibana/kibana:8.13.4
docker save -o prism-images.tar docker.elastic.co/elasticsearch/elasticsearch:8.13.4 docker.elastic.co/kibana/kibana:8.13.4
```

### LLM Weights
Download the quantized Llama GGUF models:
```bash
wget https://huggingface.co/TheBloke/Llama-2-7B-Chat-GGUF/resolve/main/llama-2-7b-chat.Q4_K_M.gguf -O ./models/llama-2-7b.gguf
```

## 2. Transfer via Secure Media
Transfer the `./offline-wheels`, `prism-images.tar`, `./models`, and the compiled PRISM binary (via `cargo build --release`) using a secure USB or CD.

## 3. Installation on Air-Gapped Server

### Install Python Packages
Install directly from the local directory without querying PyPI:
```bash
pip install --no-index --find-links=./offline-wheels -r requirements.txt
```

### Load Container Images (Podman/Docker)
```bash
podman load -i prism-images.tar
```

### Run the Stack
Boot the background services and start PRISM:
```bash
docker compose up -d
cargo run --release
```

All functionalities—including Drain3 clustering, Laya triage, and Llama.cpp VRL generation—will execute completely disconnected from the internet.
