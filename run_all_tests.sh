#!/bin/bash
set -e

echo "================================================="
echo " PRISM MASTER E2E AUTOMATED TEST & DEMO"
echo "================================================="
echo "[*] Step 1: Performing hard reset of environments..."

# Kill old processes
pkill -f 'prism' 2>/dev/null || true
pkill -f 'load_gen' 2>/dev/null || true
pkill -f 'demo_real_stream' 2>/dev/null || true
sleep 1

# Wipe /tmp/prism state and recreate cleanly
rm -rf /tmp/prism/* 2>/dev/null || true
mkdir -p /tmp/prism/rules /tmp/prism/pending_rules /tmp/prism/vault/dlq
cp rules/*.vrl /tmp/prism/rules/ 2>/dev/null || true

echo "[*] Step 2: Starting PRISM Engine (UDP: 5514, TCP: 6514) in background..."
cargo run --release --bin prism -- --udp-bind-addr 127.0.0.1:5514 --tcp-bind-addr 127.0.0.1:6514 > /tmp/prism_server.log 2>&1 &
PRISM_PID=$!

echo "[*] Step 3: Starting AI Control Plane in background..."
source venv/bin/activate
python3 prism-brain/main.py > /tmp/prism_brain.log 2>&1 &
BRAIN_PID=$!

# Ensure cleanup happens when TUI is closed
cleanup() {
    echo -e "\n[*] Cleaning up background processes..."
    kill -9 $PRISM_PID $BRAIN_PID 2>/dev/null || true
    pkill -f demo_real_stream 2>/dev/null || true
    echo "[*] All tests finished. Exiting cleanly."
}
trap cleanup EXIT

echo "[*] Step 4: Spawning Background Traffic Injector..."
(
    # Disable exit-on-error inside the subshell so tests don't crash it
    set +e
    # Wait for boot
    sleep 5
    
    # CASE 1: Known Hardware Log (UDP) -> Should parse instantly
    echo '1,2026/09/26 10:00:00,010108010441,TRAFFIC,start,2305,2026/09/26 10:00:00,10.10.20.45,203.0.113.25,0.0.0.0,0.0.0.0,Rule-Web,user1,,ssl,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2026/09/26 10:00:00,654123,1,52341,443,0,0,0x0,tcp,allow,1420,700,720,12,6,6,0' > /dev/udp/127.0.0.1/5514
    
    sleep 3
    # CASE 2: Spam Deduplication Edge Case
    # Sending 3 unique NGINX logs. AI should deduplicate via Drain3 and only make 1 rule.
    echo '192.168.1.55 - - [10/Oct/2026:13:00:00] "GET /test1 HTTP/1.1" 404 123' > /dev/udp/127.0.0.1/5514
    echo '10.0.0.99 - - [11/Oct/2026:14:15:22] "GET /test2 HTTP/1.1" 404 456' > /dev/udp/127.0.0.1/5514
    echo '172.16.5.4 - - [12/Oct/2026:15:30:11] "GET /test3 HTTP/1.1" 404 789' > /dev/udp/127.0.0.1/5514
    
    sleep 6 # Wait for AI to generate rule and put it in Gatekeeper Pending Rules
    
    # CASE 3: HitL Auto-Approval
    # We move the file from pending_rules to rules to simulate the user pressing "Approve" in the TUI
    mv /tmp/prism/pending_rules/*.vrl /tmp/prism/rules/ 2>/dev/null || true
    mv /tmp/prism/pending_rules/*.yaml /tmp/prism/rules/ 2>/dev/null || true
    
    sleep 2 # Wait for Rust Notify Watcher to hot-reload the VRL
    
    # CASE 4: TCP Universal Ingest
    # Log should bypass DLQ and parse instantly via TCP
    echo '10.0.0.200 - - [12/Oct/2026:16:00:00] "GET /tcp-stream HTTP/1.1" 200 999' > /dev/tcp/127.0.0.1/6514
    
    # CASE 5: Garbage Data / Fuzzing
    head -c 100 /dev/urandom > /dev/udp/127.0.0.1/5514
    
    sleep 2
    # CASE 6: Continuous TUI Load Test
    # Tests the TUI EPS dynamic gauge scaling
    ./demo_real_stream.sh 5514 > /dev/null 2>&1 &
    STREAM_PID=$!
    sleep 15
    kill -9 $STREAM_PID 2>/dev/null || true

) &

echo "[*] Step 5: Booting Ratatui Dashboard (Foreground)..."
echo "Preparing display..."
sleep 2
cargo run --release -p prism-tui
