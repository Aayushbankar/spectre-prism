# PRISM Judge's Evaluation & Adversarial Attack Guide

This guide is prepared for hackathon judges and red-team auditors evaluating PRISM for SIH PS-26156. It provides concrete commands to verify features, attack security boundaries, and validate resilience.

---

## ⚡ The 30-Second Evaluator One-Liner

Run the complete evaluation suite across the entire workspace in one command:
```bash
./tools/evaluate.sh
```
Or run the all-in-one verification one-liner:
```bash
cargo test --workspace --lib 2>&1 | grep "test result" && cd prism-brain && python3 -m pytest tests/ -q 2>&1 | tail -1 && cd .. && python3 verify_e2e_pipeline.py 2>&1 | grep -E "(PASSED|SUCCESS|All Tests)"
```

---

## 🛡️ Adversarial Attack Tests (Judge's Red Team Toolkit)

### Attack Vector 1: Tampered Ledger / Mutated Raw Payload
**Objective:** Prove that if a raw log is modified by a rogue insider or attacker, PRISM's cryptographic layer instantly detects and rejects it.
```bash
# Run mutation tamper tests
cargo test -p prism-merkle tampered_checkpoint_fails -- --nocapture
cargo test -p prism-merkle rewritten_history_detected -- --nocapture
```
**Expected Result:** `test tests::tampered_checkpoint_fails ... ok` and `test tests::rewritten_history_detected ... ok`. Any mutated byte produces a root mismatch.

---

### Attack Vector 2: Single Key Witness Compromise (Threshold Attack)
**Objective:** Prove that a compromised single witness node cannot forge Section 65B compliance.
```bash
# Verify 2-of-3 Ed25519 Quorum
cargo test -p prism-provenance test_witness_single_signer_fails -- --nocapture
cargo test -p prism-provenance test_witness_tampered_root_fails -- --nocapture
```
**Expected Result:** Both tests pass (`ok`), verifying that 1-of-3 signatures are rejected and modified roots fail signature verification.

---

### Attack Vector 3: Malformed & Malicious VRL Injection
**Objective:** Verify that the VRL engine and Scorer reject syntax errors, missing fields, or destructive code without crashing the runtime.
```bash
# Test VRL validation safeguards
cargo test -p prism-scorer -- --nocapture
cargo test -p prism-vrl-generator test_compile_vrl_invalid -- --nocapture
```
**Expected Result:** All tests pass, proving that invalid VRL is rejected at compile time before execution.

---

### Attack Vector 4: Byte Accounting & Data Loss Attack
**Objective:** Verify that normalization does not silently drop raw bytes or security fields.
```bash
cargo test -p prism-core accounting -- --nocapture
```
**Expected Result:** All tests pass with closure ratio > 0.95.

---

### Attack Vector 5: Alien Log Blast to UDP Port
**Objective:** Send completely unknown log payloads to PRISM and verify that the system never panics, routes unknown traffic to the DLQ, and initiates autonomous rule learning.
```bash
# 1. Start PRISM
PRISM_BASE_DIR=/tmp/attack_test ./target/release/prism --udp-bind-addr 127.0.0.1:15521 &
PRISM_PID=$!
sleep 2

# 2. Blast alien logs
for i in {1..20}; do
  echo "ALIEN_CORP_DEVICE_XYZ [id=$i] event='quantum_leak' severity=critical" | nc -u -w1 127.0.0.1 15521 || true
done

# 3. Verify quarantined in DLQ
sleep 2
ls -la /tmp/attack_test/vault/dlq_*.log

# 4. Clean up
kill $PRISM_PID 2>/dev/null || true
rm -rf /tmp/attack_test
```
**Expected Result:** 0 crashes, 0 dropped frames, exactly 20 logs written to DLQ files in the vault.
