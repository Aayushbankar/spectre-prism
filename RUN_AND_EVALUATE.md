# PRISM SIH PS-26156 — Manual Run & Evaluation Guide
# ============================================================
# Run these commands in ORDER. Each section verifies a PS requirement.

# ============================================================
# PREREQUISITES (verify before starting)
# ============================================================

# 1. Rust toolchain
rustc --version && cargo --version
# Expected: rustc 1.75+, cargo 1.75+

# 2. Python 3.11+ with venv
python3 --version
# Expected: Python 3.11+

# 3. netcat (for UDP/TCP testing)
which nc || which netcat
# Expected: /usr/bin/nc or similar

# 4. Ollama (optional - for LLM VRL generation)
ollama list 2>/dev/null | head -5 || echo "Ollama not running - heuristic fallback will be used"

# 5. Project built
cd /mnt/work/projects/sih/prism
ls -la target/release/prism
# Expected: binary exists ~15MB

# 6. Python deps installed
cd /mnt/work/projects/sih/prism/prism-brain
pip list | grep -E "requests|watchdog|drain3|filelock|pyyaml|pytest"
# Expected: all packages listed

# ============================================================
# SECTION 1: BUILD VERIFICATION (PS: "High-performance Rust data plane")
# ============================================================

# Clean build to prove no hidden issues
cd /mnt/work/projects/sih/prism
cargo build --release -p prism 2>&1 | tail -10
# Must show: "Finished `release` profile [optimized] target(s) in XX.XXs"

# Verify binary runs
./target/release/prism --help
# Must show: PRISM Ingest and Process Pipeline with UDP/TCP/File options

# ============================================================
# SECTION 2: UNIT TESTS (PS: "Reliable, tested pipeline")
# ============================================================

# Rust tests (all must pass)
cargo test --lib 2>&1 | grep -E "(test result|running)"
# Expected: 16 passed across prism-common, prism-core, prism-provenance

# Python tests (all must pass)
cd /mnt/work/projects/sih/prism/prism-brain
python3 -m pytest tests/ -q 2>&1 | tail -3
# Expected: "27 passed, 3 skipped" (bigdata GPU tests skipped without GPU)

# ============================================================
# SECTION 3: UDP INGEST TEST (PS: "Multi-vendor log ingestion")
# ============================================================

# Terminal 1: Start PRISM UDP listener
cd /mnt/work/projects/sih/prism
mkdir -p /tmp/prism_test/vault /tmp/prism_test/rules
./target/release/prism --udp-bind-addr 127.0.0.1:15514 --vault-dir /tmp/prism_test/vault --batch-size 10

# Terminal 2: Send test logs (run each, watch Terminal 1 for EPS > 0)
# Fortinet
echo 'date=2024-01-15 time=08:23:41 devname="FGT-DC-01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip=192.168.1.5 dstip=8.8.8.8 srcport=54321 dstport=443 proto=6 action="accept" sentbyte=1024 rcvdbyte=2048' | nc -u -w1 127.0.0.1 15514

# Cisco ASA
echo '%ASA-6-302013: Built inbound TCP connection 12345 for outside:192.168.1.10/54321 to inside:10.0.0.1/80' | nc -u -w1 127.0.0.1 15514

# Palo Alto
echo '1,2024/01/15 08:23:41,0011C1234,THREAT,vulnerability,1,2024/01/15 08:23:41,192.168.1.20,10.0.0.2,80,443,6,tcp,allow,high' | nc -u -w1 127.0.0.1 15514

# Verify in Terminal 1: [STATS] should show EPS > 0, fortinet=1, cisco=1, paloalto=1

# Check metrics file
cat /tmp/prism_metrics.json | python3 -m json.tool
# Must show: all three vendors counted, processed=3, dlq=0

# Check vault - no DLQ for known vendors
ls -la /tmp/prism_test/vault/
# Expected: ledger.log exists, NO dlq_*.log files

# Stop PRISM (Ctrl+C in Terminal 1)

# ============================================================
# SECTION 4: UNKNOWN LOG → DLQ ROUTING (PS: "Unknown logs quarantined")
# ============================================================

cd /mnt/work/projects/sih/prism
./target/release/prism --udp-bind-addr 127.0.0.1:15515 --vault-dir /tmp/prism_test/vault --batch-size 10 &
PRISM_PID=$!
sleep 2

echo 'this is completely unknown garbage format' | nc -u -w1 127.0.0.1 15515
sleep 2
kill $PRISM_PID

# Verify DLQ
ls -la /tmp/prism_test/vault/dlq_*.log
# Must show: at least 1 dlq_*.log file

cat /tmp/prism_test/vault/dlq_*.log | head -3
# Must show: REASON=Unknown Vendor, PAYLOAD=unknown garbage

# ============================================================
# SECTION 5: TCP INGEST TEST (PS: "TCP syslog support")
# ============================================================

cd /mnt/work/projects/sih/prism
./target/release/prism --tcp-bind-addr 127.0.0.1:15516 --vault-dir /tmp/prism_test/vault --batch-size 10 &
PRISM_PID=$!
sleep 2

# Send via TCP (with newline)
echo -e 'date=2024-01-15 time=08:23:41 devname="FGT-DC-01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip=192.168.1.5 dstip=8.8.8.8 srcport=54321 dstport=443 proto=6 action="accept" sentbyte=1024 rcvdbyte=2048\n' | nc -w2 127.0.0.1 15516

sleep 2
kill $PRISM_PID

cat /tmp/prism_metrics.json | python3 -m json.tool
# Must show: processed=1, fortinet=1

# ============================================================
# SECTION 6: HITL APPROVAL FLOW (PS: "Human-in-the-loop AI parser generation")
# ============================================================

cd /mnt/work/projects/sih/prism/prism-brain

# Submit a rule via Gatekeeper (auto dry-run validation)
python3 -c "
import sys
sys.path.insert(0, '.')
from hitl.gatekeeper import create_gatekeeper

gk = create_gatekeeper({'base_dir': '/tmp/prism_test', 'rules_dir': '/tmp/prism_test/rules'})
vrl = '''m1 = parse_regex(string!(.message), r'(?s).*?srcip=(?P<srcip>\d+\.\d+\.\d+\.\d+).*') ?? {}
if exists(m1.srcip) { .srcip = m1.srcip }
m2 = parse_regex(string!(.message), r'(?s).*?dstip=(?P<dstip>\d+\.\d+\.\d+\.\d+).*') ?? {}
if exists(m2.dstip) { .dstip = m2.dstip }
m3 = parse_regex(string!(.message), r'(?s).*?srcport=(?P<srcport>\d+).*') ?? {}
if exists(m3.srcport) { .srcport = m3.srcport }
m4 = parse_regex(string!(.message), r'(?s).*?dstport=(?P<dstport>\d+).*') ?? {}
if exists(m4.dstport) { .dstport = m4.dstport }
m5 = parse_regex(string!(.message), r'(?s).*?action=\"(?P<action>[^\"]+)\".*') ?? {}
if exists(m5.action) { .action = m5.action }
m6 = parse_regex(string!(.message), r'(?s).*?level=\"(?P<level>[^\"]+)\".*') ?? {}
if exists(m6.level) { .level = m6.level }
.class_uid = 4001
.category_uid = 4
.type_uid = 400101
.src_endpoint.ip = .srcip
.dst_endpoint.ip = .dstip'''

rule_id = gk.submit_rule('Firewall', vrl, 'fortinet_sig_001', 'date=2024-01-15 time=08:23:41 devname=\"FGT-DC-01\" devid=\"FG100\" logid=\"0000000013\" type=\"traffic\" subtype=\"forward\" level=\"notice\" srcip=192.168.1.5 dstip=8.8.8.8 srcport=54321 dstport=443 proto=6 action=\"accept\" sentbyte=1024 rcvdbyte=2048')
print(f'Submitted: {rule_id}')

# Check pending
pending = gk.get_pending_rules()
print(f'Pending: {len(pending)}')

# Approve
gk.approve_rule(rule_id)
print('Approved')

# Verify deployed
approved = gk.get_approved_rules()
print(f'Approved: {len(approved)}')

# Verify file in rules dir
import os
vrl_files = [f for f in os.listdir('/tmp/prism_test/rules') if f.endswith('.vrl')]
print(f'VRL files in rules dir: {len(vrl_files)}')
for vf in vrl_files:
    with open(os.path.join('/tmp/prism_test/rules', vf)) as f:
        c = f.read()
        assert '.class_uid = 4001' in c
        print(f'  {vf}: VALID')
"

# ============================================================
# SECTION 7: VRL DRY-RUN VALIDATION (PS: "Safe hot-reload")
# ============================================================

cd /mnt/work/projects/sih/prism
./target/release/prism --dry-run-vrl /tmp/prism_test/rules/*.vrl --payload 'date=2024-01-15 time=08:23:41 devname="FGT-DC-01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip=192.168.1.5 dstip=8.8.8.8 srcport=54321 dstport=443 proto=6 action="accept" sentbyte=1024 rcvdbyte=2048'
# Must output: Valid JSON with src_endpoint.ip=192.168.1.5, class_uid=4001

# ============================================================
# SECTION 8: OCSF OUTPUT VALIDATION (PS: "Universal OCSF taxonomy")
# ============================================================

cd /mnt/work/projects/sih/prism/prism-brain
python3 -c "
import sys
sys.path.insert(0, '.')
from coder.coder import VrlCoder

coder = VrlCoder({'device': 'cpu', 'coder': {'enabled': False}})

# Test all 4 OCSF classes generate correct class_uid
test_cases = [
    ('Firewall log srcip=1.2.3.4 dstip=5.6.7.8 action=accept', 'Firewall', 4001),
    ('User jdoe login from 10.0.0.1', 'Authentication', 3001),
    ('GET http://example.com 200 from 192.168.1.1', 'Web Proxy', 5001),
    ('File test.exe written to C:\\temp\\', 'File Activity', 8001),
]

for log, dtype, expected_uid in test_cases:
    vrl = coder.generate_vrl(log, dtype)
    assert f'.class_uid = {expected_uid}' in vrl, f'{dtype}: missing class_uid {expected_uid}'
    assert '.category_uid =' in vrl
    assert '.type_uid =' in vrl
    print(f'✓ {dtype}: class_uid={expected_uid} OK')
print('All 4 OCSF classes generate correct UIDs')
"

# ============================================================
# SECTION 9: TUI VERIFICATION (PS: "Operational visibility")
# ============================================================

# Terminal 1: Start PRISM with metrics
cd /mnt/work/projects/sih/prism
./target/release/prism --udp-bind-addr 127.0.0.1:15517 --vault-dir /tmp/prism_test/vault --batch-size 100 &
PRISM_PID=$!
sleep 2

# Terminal 2: Start AI brain (generates rules from DLQ)
cd /mnt/work/projects/sih/prism/prism-brain
python3 main.py &
AI_PID=$!
sleep 2

# Terminal 3: Send unknown logs to trigger AI
for i in {1..5}; do
  echo "unknown_app_$i: user=admin action=login srcip=10.0.0.$i" | nc -u -w1 127.0.0.1 15517
done
sleep 3

# Terminal 4: Launch TUI (will show Gatekeeper tab with pending rules)
cd /mnt/work/projects/sih/prism
./target/release/prism-tui
# Navigate: Tab → Gatekeeper tab
# Should see: ⏳ PENDING rules with device type
# Keys: ↑/↓ navigate, Enter/Approve, R/Reject, ?=help

# Cleanup
kill $PRISM_PID $AI_PID 2>/dev/null

# ============================================================
# SECTION 10: PROVENANCE LEDGER (PS: "Immutable audit trail")
# ============================================================

cd /mnt/work/projects/sih/prism
./target/release/prism --udp-bind-addr 127.0.0.1:15518 --vault-dir /tmp/prism_test/vault --batch-size 2 &
PRISM_PID=$!
sleep 2

# Send 3 logs to trigger batch flush (batch_size=2)
for i in {1..3}; do
  echo "date=2024-01-15 time=08:23:41 devname=\"FGT-DC-01\" logid=\"0000000013\" type=\"traffic\" level=\"notice\" srcip=192.168.1.$i dstip=8.8.8.8 action=\"accept\"" | nc -u -w1 127.0.0.1 15518
done
sleep 3
kill $PRISM_PID

# Verify ledger
cat /tmp/prism_test/vault/ledger.log
# Must show: 32-char hex Merkle roots (one per batch)

# Verify vault structure
find /tmp/prism_test/vault -type f -name "*.parquet" | head -3
# Must show: Parquet files with OCSF events

# ============================================================
# SECTION 11: E2E AUTOMATED TEST (all-in-one)
# ============================================================

cd /mnt/work/projects/sih/prism
./test_e2e.sh
# Must show: "=== All Tests Passed ===" at end

# ============================================================
# SIH JUDGE EVALUATION CHECKLIST (PS-26156 Requirements)
# ============================================================

# | # | PS-26156 Requirement                    | Verification Command                          | Pass Criteria                          |
# |---|------------------------------------------|----------------------------------------------|----------------------------------------|
# | 1 | High-performance Rust data plane         | cargo test --lib + E2E UDP test              | >10k EPS, zero data races              |
# | 2 | Multi-vendor ingestion (Fortinet/Cisco/  | Section 3 UDP test                           | All 3 vendors parsed, correct OCSF     |
# |   | Palo Alto)                               |                                              |                                        |
# | 3 | Unknown log quarantine (DLQ)             | Section 4                                    | Unknown → DLQ, known → processed       |
# | 4 | TCP syslog support                       | Section 5                                    | TCP logs processed                     |
# | 5 | Universal OCSF taxonomy (not hardcoded)  | Section 8                                    | 4 class_uids: 4001,3001,5001,8001      |
# | 6 | AI parser generation (local LLM)         | python -c "from coder.coder import...        | Generates valid VRL with class_uid     |
# | 7 | VRL syntax validation (no bad deploys)   | Section 7 + Gatekeeper dry-run               | Invalid VRL rejected, valid passes     |
# | 8 | Human-in-the-loop approval               | Section 6 + TUI                              | Pending → Approve → Deploy atomic      |
# | 9 | Hot-reload without restart               | Gatekeeper approve + VRL watcher             | New .vrl picked up in <1s              |
# |10 | Provenance/integrity (Merkle + ledger)   | Section 10                                   | ledger.log has hex roots, vault=parquet|
# |11 | Operational metrics (EPS, drops, DLQ)    | /tmp/prism_metrics.json + TUI                | Real-time JSON + TUI dashboard         |
# |12 | Air-gapped deployment ready              | cargo build --release + pip install -r reqs  | No external deps at runtime            |
# |13 | Byte-level accounting (fidelity)         | cargo test accounting                        | closure_ratio > 0.95 for known formats |
# |14 | TUI operational visibility               | Section 9                                    | 4 tabs: Dashboard/Telemetry/Gate/DLQ   |

# ============================================================
# QUICK SMOKE TEST (run this single command for full verify)
# ============================================================

cd /mnt/work/projects/sih/prism && \
cargo test --lib 2>&1 | grep "test result" && \
cd prism-brain && python3 -m pytest tests/ -q 2>&1 | tail -1 && \
cd .. && ./test_e2e.sh 2>&1 | grep -E "(SUCCESS|FAILED|All Tests)"

# Expected output:
# test result: ok. 16 passed...
# 27 passed, 3 skipped...
# [SUCCESS] === All Tests Passed ===

# ============================================================
# STATUS CHECK COMMANDS (run anytime during demo)
# ============================================================

# Live metrics
watch -n 1 'cat /tmp/prism_metrics.json | python3 -m json.tool'

# Live DLQ
watch -n 2 'find /tmp/prism_test/vault -name "dlq_*.log" -exec head -3 {} \;'

# Live ledger
watch -n 2 'cat /tmp/prism_test/vault/ledger.log'

# Live rules
watch -n 2 'ls -la /tmp/prism_test/rules/'

# Gatekeeper state
cd /mnt/work/projects/sih/prism/prism-brain && python3 -c "
import sys; sys.path.insert(0, '.')
from hitl.gatekeeper import create_gatekeeper
gk = create_gatekeeper({'base_dir': '/tmp/prism_test', 'rules_dir': '/tmp/prism_test/rules'})
for r in gk.get_all_rules():
    print(f'{r.rule_id[:40]} | {r.state:10} | {r.device_type}')
"

# ============================================================
# CLEANUP (between test runs)
# ============================================================

pkill -f "prism.*test" 2>/dev/null; pkill -f "python.*main.py" 2>/dev/null
rm -rf /tmp/prism_test /tmp/prism_metrics.json /tmp/prism_ai_status /tmp/prism/pending_rules /tmp/prism/rules /tmp/prism/vault
# All clean for next run