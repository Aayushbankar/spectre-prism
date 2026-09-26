#!/bin/bash
# Cleanup any previous instances
pkill -f 'target/release/prism' 2>/dev/null || true
pkill -f 'target/debug/prism' 2>/dev/null || true
rm -rf /tmp/prism/* 2>/dev/null || true
sleep 1
set -e

echo "============================================="
echo " PRISM Zero-Downtime AI Hot-Reloading Demo "
echo "============================================="

# 1. Compile PRISM (if not compiled)
if [ ! -f "target/release/prism" ]; then
    echo "[*] Compiling PRISM Engine..."
    cargo build --release -p prism >/dev/null 2>&1
fi

echo "[*] Starting PRISM in the background..."
./target/release/prism --udp-bind-addr 127.0.0.1:5514 > /tmp/prism_demo_output.log 2>&1 &
PRISM_PID=$!

cleanup() {
    echo "[*] Cleaning up..."
    kill -SIGINT $PRISM_PID 2>/dev/null
    wait $PRISM_PID 2>/dev/null
}
trap cleanup EXIT

# Ensure directories exist
mkdir -p /tmp/prism/rules
mkdir -p /tmp/prism/dlq

# Wait for PRISM to boot up
sleep 3

# 2. Let's send an UNKNOWN format (Nginx Web Proxy)
echo "[*] Sending UNKNOWN log format (Nginx Access Log) over UDP..."
# This doesn't match Fortinet, Cisco, or Palo Alto
echo "192.168.1.100 - - [10/Oct/2026:13:55:36 -0700] \"GET /index.html HTTP/1.1\" 200 2326" > /dev/udp/127.0.0.1/5514

# Wait for DLQ write
sleep 1

echo "[*] PRISM routed it to the DLQ."
echo "--- DLQ CONTENTS ---"
tail -n 1 /tmp/prism/vault/dlq_*.log | head -n 1
echo "--------------------"

echo "[*] Simulating AI Pipeline (Triage -> Coder -> Gatekeeper)..."
# We run the python script that executes the AI loop
cat << 'EOF' > /tmp/run_ai.py
import sys
import os
os.chdir('/mnt/work/projects/sih/prism')
sys.path.insert(0, os.path.abspath('prism-brain'))
from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper

log = '192.168.1.100 - - [10/Oct/2026:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326'
triage = TriageEngine({"device": "cpu"})
device_type = triage.classify(log)
print(f"[AI] Triaged device as: {device_type}")

coder = VrlCoder({"coder": {"enabled": True}})
vrl_code = coder.generate_vrl(log, device_type)
print(f"[AI] Generated VRL:\n{vrl_code}")

gatekeeper = Gatekeeper()
gatekeeper.approve_and_deploy(device_type, vrl_code, "nginx_sig", log)
print(f"[Gatekeeper] Validated via dry-run and deployed to /tmp/prism/rules/")
EOF

python3 /tmp/run_ai.py

echo "[*] Waiting for PRISM notify thread to Hot-Reload the rules..."
sleep 2

echo "[*] Sending the SAME UNKNOWN log format again..."
echo "192.168.1.100 - - [10/Oct/2026:13:55:36 -0700] \"GET /index.html HTTP/1.1\" 200 2326" > /dev/udp/127.0.0.1/5514

# Give PRISM a moment to process
sleep 2

echo "[*] Gracefully shutting down PRISM..."
kill -SIGINT $PRISM_PID
wait $PRISM_PID || true

echo "[*] Checking PRISM Output Log for successful parsing..."
echo "--- PRISM OUTPUT ---"
grep "192.168.1.100" /tmp/prism_demo_output.log | grep -v "REASON" | tail -n 1 || echo "Check /tmp/prism_demo_output.log"
echo "--------------------"

echo "Demo complete!"
