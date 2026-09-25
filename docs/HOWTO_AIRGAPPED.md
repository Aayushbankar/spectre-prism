# HOW-TO: Air-gapped Deployment

This runbook describes how to deploy PRISM in a completely disconnected NTRO environment.

## 1. Fetching Dependencies (Online Machine)
First, download all required Python wheels and model weights:
```bash
pip download -r requirements.txt --platform manylinux2014_x86_64 --only-binary=:all:
wget https://huggingface.co/lmstudio-community/Meta-Llama-3-8B-Instruct-GGUF/resolve/main/Meta-Llama-3-8B-Instruct-Q4_K_M.gguf
```
Save the offline container images:
```bash
docker pull docker.elastic.co/elasticsearch/elasticsearch:8.13.4
docker save docker.elastic.co/elasticsearch/elasticsearch:8.13.4 -o elastic.tar
```

## 2. Transfer
Transfer all `.whl`, `.gguf`, and `.tar` files to the air-gapped machine via a secure USB drive.

## 3. Offline Installation (Air-gapped Machine)
Load the container images using Podman (daemonless):
```bash
podman load -i elastic.tar
```
Install the Python dependencies from the local directory:
```bash
pip install --no-index --find-links=/path/to/wheels -r requirements.txt
```
Start the local stack:
```bash
docker compose up -d
cargo run
```
