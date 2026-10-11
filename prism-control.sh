#!/usr/bin/env bash
# ============================================================
# PRISM — UNIFIED CONTROL PLANE MANAGER
# ============================================================
# Single script to manage all PRISM planes, llama.cpp server, TUI, and demo
# ============================================================

set -euo pipefail

# ============================================================
# CONFIGURATION
# ============================================================
PRISM_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRISM_BRAIN="${PRISM_ROOT}/prism-brain"
PRISM_BIN="${PRISM_ROOT}/target/release/prism"
PRISM_TUI="${PRISM_ROOT}/target/release/prism-tui"
DEMO_BASE="/tmp/prism_demo"
VAULT_DIR="${DEMO_BASE}/vault"
RULES_DIR="${DEMO_BASE}/rules"
PENDING_DIR="${DEMO_BASE}/pending_rules"
APPROVED_DIR="${DEMO_BASE}/approved_rules"
META_DIR="${DEMO_BASE}/rule_metadata"
LLAMA_MODEL="${LLAMA_MODEL:-${PRISM_ROOT}/models/Qwen2.5-Coder-3B-Instruct-Q4_K_M.gguf}"
LLAMA_PORT=8080
LLAMA_HOST="127.0.0.1"
PRISM_PORT=15514
AI_PORT=8080

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# PID tracking
LLAMA_PID=""
PRISM_PID=""
AI_PID=""
TUI_PID=""

# ============================================================
# UTILITY FUNCTIONS
# ============================================================
log() { echo -e "${GREEN}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; }
info() { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[✓]${NC} $*"; }
header() { echo -e "\n${MAGENTA}════════════════════════════════════════════════════════${NC}"; echo -e "${MAGENTA}  $*${NC}"; echo -e "${MAGENTA}═════════════════════════════════════════════════════════${NC}"; }

check_prereqs() {
    header "CHECKING PREREQUISITES"
    local missing=0
    command -v cargo >/dev/null 2>&1 || { error "cargo not found"; missing=1; }
    command -v python3 >/dev/null 2>&1 || { error "python3 not found"; missing=1; }
    command -v nc >/dev/null 2>&1 || { error "nc (netcat) not found"; missing=1; }
    command -v jq >/dev/null 2>&1 || { error "jq not found"; missing=1; }
    [[ -f "$PRISM_BIN" ]] || { error "PRISM binary not found at $PRISM_BIN. Run 'cargo build --release --workspace' first."; missing=1; }
    [[ -f "$PRISM_TUI" ]] || { error "PRISM TUI not found at $PRISM_TUI. Run 'cargo build --release --workspace' first."; missing=1; }
    [[ -f "$LLAMA_MODEL" ]] || { error "Llama model not found at $LLAMA_MODEL"; missing=1; }
    [[ -d "$PRISM_BRAIN" ]] || { error "PRISM brain not found at $PRISM_BRAIN"; missing=1; }
    [[ $missing -eq 0 ]] || exit 1
    success "All prerequisites satisfied"
}

setup_dirs() {
    header "SETTING UP DEMO DIRECTORIES"
    mkdir -p "$VAULT_DIR" "$RULES_DIR" "$PENDING_DIR" "$APPROVED_DIR" "$META_DIR"
    # Copy base VRL rules
    cp -n "$PRISM_ROOT"/rules/*.vrl "$RULES_DIR/" 2>/dev/null || true
    success "Demo directories ready at $DEMO_BASE"
}

check_llama_server() {
    curl -sf "http://$LLAMA_HOST:$LLAMA_PORT/health" >/dev/null 2>&1 || \
    curl -sf "http://$LLAMA_HOST:$LLAMA_PORT/v1/models" >/dev/null 2>&1
}

start_llama_server() {
    header "STARTING LLAMA.CPP SERVER (Qwen2.5-Coder-3B-Instruct-Q4_K_M)"
    if check_llama_server; then
        warn "Llama.cpp server already running on port $LLAMA_PORT"
        return 0
    fi
    [[ -f "$LLAMA_MODEL" ]] || { error "Model not found at $LLAMA_MODEL"; return 1; }
    info "Starting llama.cpp server on port $LLAMA_PORT..."
    nohup llama-server \
        -m "$LLAMA_MODEL" \
        -c 8192 \
        -n 1024 \
        --port 8080 \
        --host 127.0.0.1 \
        --ctx-size 8192 \
        --batch-size 512 \
        --ubatch-size 512 \
        --threads $(nproc) \
        --temp 0.1 \
        --repeat-penalty 1.1 \
        --repeat-last-n 64 \
        --mlock \
        > /tmp/llama_server.log 2>&1 &
    LLAMA_PID=$!
    echo $LLAMA_PID > /tmp/prism_llama.pid
    info "Waiting for llama.cpp server to be ready..."
    local retries=30
    while ! check_llama_server && [[ $retries -gt 0 ]]; do
        sleep 1
        ((retries--))
    done
    if check_llama_server; then
        success "Llama.cpp server running on http://127.0.0.1:8080 (PID: $LLAMA_PID)"
        return 0
    else
        error "Failed to start llama.cpp server. Check /tmp/llama_server.log"
        return 1
    fi
}

stop_llama_server() {
    if [[ -f /tmp/prism_llama.pid ]]; then
        local pid=$(cat /tmp/prism_llama.pid 2>/dev/null)
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null
            info "Stopped llama.cpp server (PID: $pid)"
        fi
        rm -f /tmp/prism_llama.pid
    fi
}

# ============================================================
# PLANE MANAGEMENT
# ============================================================
start_prism_dataplane() {
    header "STARTING PRISM DATA PLANE (Ingestion + VRL + Vault + Merkle)"
    if [[ -n "$PRISM_PID" ]] && kill -0 "$PRISM_PID" 2>/dev/null; then
        warn "PRISM data plane already running (PID: $PRISM_PID)"
        return 0
    fi
    mkdir -p "$VAULT_DIR" "$RULES_DIR"
    nohup "$PRISM_BIN" \
        --udp-bind-addr 127.0.0.1:15514 \
        --vault-dir "$VAULT_DIR" \
        --rules-dir "$RULES_DIR" \
        --batch-size 10 \
        > /tmp/prism_dataplane.log 2>&1 &
    PRISM_PID=$!
    echo $PRISM_PID > /tmp/prism_dataplane.pid
    sleep 2
    if kill -0 "$PRISM_PID" 2>/dev/null; then
        success "PRISM Data Plane started (PID: $PRISM_PID) on UDP 127.0.0.1:15514"
        return 0
    else
        error "Failed to start PRISM data plane. Check /tmp/prism_dataplane.log"
        return 1
    fi
}

stop_prism_dataplane() {
    if [[ -f /tmp/prism_dataplane.pid ]]; then
        local pid=$(cat /tmp/prism_dataplane.pid 2>/dev/null)
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null
            info "Stopped PRISM data plane (PID: $pid)"
        fi
        rm -f /tmp/prism_dataplane.pid
    fi
}

start_ai_brain() {
    header "STARTING AI BRAIN (Drain + Laya + VRL Generator + Gatekeeper)"
    if [[ -n "$AI_PID" ]] && kill -0 "$AI_PID" 2>/dev/null; then
        warn "AI Brain already running (PID: $AI_PID)"
        return 0
    fi
    cd "$PRISM_BRAIN"
    nohup python3 main.py \
        --vault-dir "$VAULT_DIR" \
        --base-dir "/tmp/prism" \
        --rules-dir "$RULES_DIR" \
        > /tmp/prism_ai_brain.log 2>&1 &
    AI_PID=$!
    echo $AI_PID > /tmp/prism_ai.pid
    sleep 3
    if kill -0 "$AI_PID" 2>/dev/null; then
        success "AI Brain started (PID: $AI_PID)"
        return 0
    else
        error "Failed to start AI Brain. Check /tmp/prism_ai_brain.log"
        return 1
    fi
}

stop_ai_brain() {
    if [[ -f /tmp/prism_ai.pid ]]; then
        local pid=$(cat /tmp/prism_ai.pid 2>/dev/null)
        if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null
            info "Stopped AI Brain (PID: $pid)"
        fi
        rm -f /tmp/prism_ai.pid
    fi
}

start_tui() {
    header "LAUNCHING PRISM TUI (Gatekeeper Dashboard)"
    if [[ -n "$TUI_PID" ]] && kill -0 "$TUI_PID" 2>/dev/null; then
        warn "TUI already running (PID: $TUI_PID)"
        return 0
    fi
    info "Launching TUI... Press 'q' to quit, 'Tab' to switch tabs, 'a'/'Enter' to approve rules"
    PRISM_BASE_DIR=/tmp/prism_demo "$PRISM_TUI"
}

stop_tui() {
    if [[ -n "$TUI_PID" ]] && kill -0 "$TUI_PID" 2>/dev/null; then
        kill "$TUI_PID" 2>/dev/null
        info "Stopped TUI (PID: $TUI_PID)"
        TUI_PID=""
    fi
}

# ============================================================
# INGESTION HELPERS
# ============================================================
send_known_logs() {
    header "SENDING KNOWN VENDOR LOGS (Fortinet + Cisco + Palo Alto)"
    echo 'date=2024-01-15 time=08:23:41 devname="FGT-DC-01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip=192.168.1.5 dstip=8.8.8.8 srcport=54321 dstport=443 proto=6 action="accept" sentbyte=1024 rcvdbyte=2048' | nc -u -w1 127.0.0.1 15514
    echo '%ASA-6-302013: Built inbound TCP connection 12345 for outside:192.168.1.10/54321 to inside:10.0.0.1/80' | nc -u -w1 127.0.0.1 15514
    echo '1,2024/01/15 12:00:00,0011C1234,THREAT,vulnerability,1,2024/01/15 12:00:00,192.168.1.20,10.0.0.2,80,443,6,tcp,allow,high' | nc -u -w1 127.0.0.1 15514
    success "Sent 3 known vendor logs (Fortinet, Cisco, Palo Alto)"
}

send_unknown_logs() {
    local count=${1:-5}
    header "SENDING $count UNKNOWN LOGS (Will trigger DLQ -> AI Brain)"
    for i in $(seq 1 $count); do
        echo "unknown_app_$i: user=admin action=login srcip=10.0.0.$i" | nc -u -w1 127.0.0.1 15514
        sleep 0.1
    done
    success "Sent $count unknown logs (should appear in DLQ)"
}

show_metrics() {
    header "LIVE METRICS"
    if [[ -f /tmp/prism_metrics.json ]]; then
        cat /tmp/prism_metrics.json | jq .
    else
        warn "No metrics file found at /tmp/prism_metrics.json"
    fi
}

show_vault_status() {
    header "VAULT & LEDGER STATUS"
    echo "--- Vault Contents ---"
    ls -la "$VAULT_DIR"/ 2>/dev/null || echo "Vault directory empty"
    echo ""
    echo "--- DLQ Files ---"
    ls -la "$VAULT_DIR"/dlq_*.log 2>/dev/null | head -10 || echo "No DLQ files"
    echo ""
    echo "--- Ledger (Merkle Roots) ---"
    if [[ -f "$VAULT_DIR/ledger.log" ]]; then
        cat "$VAULT_DIR/ledger.log"
    else
        echo "No ledger yet"
    fi
    echo ""
    echo "--- Active Rules ---"
    ls -la "$RULES_DIR"/*.vrl 2>/dev/null | awk '{print $9}' | sed 's|.*/||' || echo "No rules"
}

# ============================================================
# FULL PIPELINE AUTOMATION
# ============================================================
run_full_demo() {
    header "RUNNING FULL E2E PIPELINE (Automated)"
    check_prereqs
    setup_dirs
    start_llama_server || return 1
    start_prism_dataplane || return 1
    send_known_logs
    sleep 2
    send_unknown_logs 5
    start_ai_brain || return 1
    sleep 5
    # Wait for rule to appear
    local retries=30
    while [[ $retries -gt 0 ]]; do
        if ls "$PENDING_DIR"/*.vrl >/dev/null 2>&1; then
            break
        fi
        sleep 1
        ((retries--))
    done
    # Auto-approve first pending rule
    for rule in "$PENDING_DIR"/*.vrl; do
        [[ -f "$rule" ]] || continue
        local basename=$(basename "$rule")
        mv "$rule" "$APPROVED_DIR/$basename"
        cp "$APPROVED_DIR/$basename" "$RULES_DIR/$basename"
        success "Auto-approved rule: $basename"
        break
    done
    sleep 2
    send_known_logs  # Re-send to verify re-parse
    show_metrics
    success "Full pipeline demo complete!"
}

# ============================================================
show_menu() {
    clear 2>/dev/null || true
    header "PRISM — UNIFIED CONTROL PLANE"
    echo -e "${CYAN}┌────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${CYAN}│${NC}  ${MAGENTA}PRISM — Unified Control Plane${NC}                ${CYAN}│${NC}"
    echo -e "${CYAN}├────────────────────────────────────────────────────────────┤${NC}"
    echo -e "${CYAN}│${NC}  ${GREEN}1)${NC}  Start ALL (Llama + Data Plane + AI Brain)             ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}2)${NC}  Start Llama.cpp Server (Qwen2.5-Coder-3B)             ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}3)${NC}  Start Data Plane (Ingestion + VRL + Vault + Merkle)   ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}4)${NC}  Start AI Brain (Drain + Laya + VRL Gen + Gatekeeper)  ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}5)${NC}  Launch TUI (Gatekeeper Dashboard)                     ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}6)${NC}  Send Known Vendor Logs (Fortinet, Cisco, Palo Alto)   ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}7)${NC}  Send Unknown Logs (Trigger DLQ + AI Brain)            ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}8)${NC}  Run FULL E2E Pipeline (Auto)                          ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${GREEN}9)${NC}  Show Live Metrics                                     ${CYAN}│"
    echo -e "${CYAN}│${NC} ${GREEN}10)${NC}  Show Vault & Ledger Status                            ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${YELLOW}0)${NC}  Stop ALL Services                                     ${CYAN}│"
    echo -e "${CYAN}│${NC}  ${RED}q)${NC}  Quit                                                  ${CYAN}│"
    echo -e "${CYAN}└────────────────────────────────────────────────────────────┘${NC}"
    echo -n -e "${YELLOW}Select option: ${NC}"
}

cleanup_all() {
    header "STOPPING ALL SERVICES"
    stop_tui
    stop_ai_brain
    stop_prism_dataplane
    stop_llama_server
    success "All services stopped"
}

# ============================================================
# MAIN LOOP
# ============================================================
main() {
    trap cleanup_all EXIT INT TERM
    check_prereqs
    setup_dirs

    while true; do
        show_menu
        read -r choice
        echo
        case "$choice" in
            1) check_prereqs && setup_dirs && start_llama_server && start_prism_dataplane && start_ai_brain ;;
            2) start_llama_server ;;
            3) check_prereqs && setup_dirs && start_prism_dataplane ;;
            4) check_prereqs && setup_dirs && start_llama_server && start_ai_brain ;;
            5) start_tui ;;
            6) send_known_logs ;;
            7) send_unknown_logs 5 ;;
            8) run_full_demo ;;
            9) show_metrics ;;
            10) show_vault_status ;;
            0) trap - EXIT; cleanup_all; exit 0 ;;
            q|Q) trap - EXIT; cleanup_all; exit 0 ;;
            *) warn "Invalid option: $choice" ;;
        esac
        echo
        read -p "Press Enter to continue..." _
    done
}

# ============================================================
# ENTRY POINT
# ============================================================
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi