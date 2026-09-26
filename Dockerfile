FROM rust:1.77-slim AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential cmake && rm -rf /var/lib/apt/lists/*
RUN cargo build --release -p prism

FROM python:3.11-slim
WORKDIR /app
COPY --from=builder /app/target/release/prism /usr/local/bin/prism
COPY prism-brain /app/prism-brain
COPY rules /app/rules
COPY prism-brain/requirements.txt /app/
RUN pip install -no-cache-dir -r requirements.txt
CMD ["prism", "--udp-bind-addr", "0.0.0.0:514"]
