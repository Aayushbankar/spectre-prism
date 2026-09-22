# Architectural Decision Record (ADR 01): Containerization Strategy

## Context
Requirement (k) of SIH26156 states: *"Solution may be packaged in a container for making it platform independent."*
However, Requirement (j) mandates air-gapped deployment, and the core premise of the framework is to handle "billions of events per day" (high throughput).

## The Containerization "Trap"
Running a high-speed network ingestion pipeline inside a standard Docker environment introduces severe performance penalties:
1. **Docker Bridge Network Overhead:** UDP/TCP traffic routed through Docker's `veth` pairs and `iptables` NAT routing introduces user-space to kernel-space context switching.
2. **CPU/Memory Waste:** In embedded or highly constrained perimeter environments, dedicating host resources to the Docker daemon and container virtualization overhead contradicts our goal of a zero-copy, bare-metal efficient Rust pipeline.

## Decision: Dual-Deployment Strategy
To satisfy both maximum performance and the hackathon evaluation criteria, we adopt a Dual-Deployment strategy:

### 1. Production & Live Demo Deployment: Bare-Metal Static Binary
* **How:** PRISM is compiled as a statically linked Rust binary (`x86_64-unknown-linux-musl`).
* **Why:** This avoids all virtualization overhead, allowing the Tokio UDP listener to bind directly to the host network interface (NIC). This is how we achieve 50,000+ EPS and zero-copy memory efficiency during the live hackathon demo. 

### 2. Evaluator / Reproduction Deployment: Dockerized
* **How:** A lightweight `Dockerfile` (using Alpine/Scratch) and a `docker-compose.yml` file are provided in the repository.
* **Why:** This strictly satisfies Requirement (k) for "platform independence." Evaluators and judges can easily spin up the environment on their own machines using `docker run` to verify functionality without installing Rust or Python dependencies. 

## Consequence
During the technical presentation, we will explicitly highlight this decision. By demonstrating that we *can* containerize the app, but *choose* to run it bare-metal for the demo to bypass kernel networking bottlenecks, we prove a deep understanding of high-performance systems engineering.
