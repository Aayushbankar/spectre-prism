# Ratatui Dashboard Testing Guide

Follow these exact steps to manually spin up the PRISM system, generate real events (no mocks!), and watch the Ratatui Dashboard light up in real time.

## Pre-requisites
Since your laptop just restarted, ensure you are in the project root:
```bash
cd /mnt/work/projects/sih/prism
```

## Step 1: Start the SIEM (Elasticsearch & Kibana)
This runs in the background.
```bash
docker-compose up -d
```
*(If you don't care about Kibana for this exact test, you can skip this step).*

## Step 2: Start the Ratatui Dashboard (Terminal 1)
Open a new terminal window and run:
```bash
cargo run -p prism-tui
```
*Note: The dashboard will look mostly empty until the PRISM backend starts pumping data.*

## Step 3: Start the PRISM Backend Engine (Terminal 2)
In another terminal, start the main data plane:
```bash
cargo run --release -p prism --bin prism
```
*The engine will bind to UDP port `514` and start recording metrics. Your TUI's EPS chart will instantly connect and show `0 EPS` rather than being frozen.*

## Step 4: Inject Real Load (Terminal 3)
To see the TUI react, we need to send real network logs. PRISM is incredibly fast, so we can use a bash loop to blast the UDP port.

**Test 1: Normal Traffic Spike (Watch the EPS gauge)**
```bash
# Blast 20,000 logs into PRISM as fast as possible
for i in {1..20000}; do 
  echo 'logid="0000000013" type=traffic srcip=192.168.1.5' > /dev/udp/127.0.0.1/514
done
```
*Look at the Ratatui Dashboard. The green EPS gauge will spike dramatically, and the line chart will plot the burst!*

**Test 2: Triggering the Dead Letter Queue (Watch the DLQ Table)**
Send a completely malformed, unrecognized log format:
```bash
echo "192.168.1.100 - - [10/Oct/2026:13:55:36] \"GET /alien.html HTTP/1.1\" 404 0" > /dev/udp/127.0.0.1/514
```
*Look at the TUI's DLQ Pane. It will instantly extract the alien log and display it in the table in red.*

**Test 3: The Integrity Merkle Root**
*As logs flow through, watch the bottom-left pane. The `Latest Root` will constantly update with the live cryptographically signed BLAKE3 Merkle root.*

## Step 5: (Optional) Testing the HitL (Human-in-the-Loop) AI Engine
If you want to test the full loop where the AI writes a parser for that dropped Nginx log:
```bash
# From the project root:
./demo_ai_loop.sh
```
*Watch the bottom-right pane on your Ratatui TUI. When the Python AI generates a new `.vrl` parser, it will instantly pop up in the **HitL Gatekeeper** list as `[Pending] nginx_sig.vrl`.*
