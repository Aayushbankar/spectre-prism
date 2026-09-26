#!/usr/bin/env bash
# evaluate.sh - One-command PRISM evaluation for judges
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

echo "========================================================"
echo "🎯 PRISM SIH PS-26156 ONE-COMMAND EVALUATOR"
echo "========================================================"

EVAL_DIR="/tmp/prism_eval"
mkdir -p "$EVAL_DIR/vault" "$EVAL_DIR/rules" "$EVAL_DIR/pending_rules" "$EVAL_DIR/approved_rules" "$EVAL_DIR/rule_metadata"
cp rules/*.vrl "$EVAL_DIR/rules/" 2>/dev/null || true

PRISM_PID=""
AI_PID=""

cleanup() {
  echo "[CLEANUP] Stopping background evaluation processes..."
  if [[ -n "$PRISM_PID" ]]; then
    kill "$PRISM_PID" 2>/dev/null || true
  fi
  if [[ -n "$AI_PID" ]]; then
    kill "$AI_PID" 2>/dev/null || true
  fi
  pkill -f "target/release/prism" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

echo "1. Building PRISM release workspace..."
cargo build --release --workspace

echo "2. Running unit test suite across all crates..."
cargo test --workspace --lib

echo "3. Starting PRISM Data Plane daemon..."
PRISM_BASE_DIR="$EVAL_DIR" ./target/release/prism --udp-bind-addr 127.0.0.1:15519 --vault-dir "$EVAL_DIR/vault" --rules-dir "$EVAL_DIR/rules" --batch-size 10 &
PRISM_PID=$!
sleep 2

echo "4. Starting PRISM AI Brain (Drain + Laya ModernBERT)..."
cd "$REPO_ROOT/prism-brain"
python3 main.py --vault-dir "$EVAL_DIR/vault" --base-dir "$EVAL_DIR" --rules-dir "$EVAL_DIR/rules" &
AI_PID=$!
cd "$REPO_ROOT"
sleep 2

echo "5. Injecting unknown logs into UDP port 15519..."
for i in {1..10}; do
  echo "unknown_app_$i: user=admin action=login srcip=10.0.0.$i" | nc -u -w1 127.0.0.1 15519 || true
done
sleep 2

echo "6. Checking DLQ output in Vault..."
dlq_count=$(ls -1 "$EVAL_DIR/vault"/dlq_*.log 2>/dev/null | wc -l)
echo "    Found $dlq_count DLQ log files generated for untracked logs."

echo "7. Running comprehensive E2E verification suite with live TUI HitL..."
python3 verify_e2e_pipeline.py

echo "========================================================"
echo "✅ EVALUATION COMPLETE: ALL CRITERIA VERIFIED"
echo "========================================================"
