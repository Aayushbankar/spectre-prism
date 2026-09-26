#!/usr/bin/env python3
"""
PRISM End-to-End Pipeline Verification Script
Tests: UDP -> Drain -> Laya -> VRL (Real Crate) -> Vault -> Merkle -> Witness -> Metrics -> TUI Gatekeeper Flow
Captures real live screenshots and ANSI text logs of the running TUI at every key transition.
"""

import os
import sys
import time
import json
import socket
import shutil
import subprocess
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

WORKSPACE = Path("/mnt/work/projects/sih/prism")
PRISM_BIN = WORKSPACE / "target/release/prism"
PRISM_TUI_BIN = WORKSPACE / "target/release/prism-tui"
VAULT_DIR = Path("/tmp/prism/vault")
RULES_DIR = Path("/tmp/prism/rules")
BASE_DIR = Path("/tmp/prism")
METRICS_FILE = Path("/tmp/prism_metrics.json")
SCREENSHOTS_DIR = WORKSPACE / "screenshots"
UDP_PORT = 15514
TMUX_SESSION = "prism_tui_verify"

FONT_PATH = "/usr/share/fonts/liberation/LiberationMono-Regular.ttf"

def render_terminal_to_image(text: str, output_png_path: Path):
    """Render terminal ASCII/text capture into a high-res PNG screenshot."""
    lines = text.splitlines()
    line_count = max(len(lines), 35)
    char_count = max(max(len(line) for line in lines) if lines else 120, 120)

    char_width = 9
    char_height = 18
    pad_x = 24
    pad_y = 24
    
    img_width = char_count * char_width + pad_x * 2
    img_height = line_count * char_height + pad_y * 2

    img = Image.new("RGB", (img_width, img_height), color="#0f141c")
    draw = ImageDraw.Draw(img)

    try:
        font = ImageFont.truetype(FONT_PATH, 14)
    except Exception:
        font = ImageFont.load_default()

    y = pad_y
    for line in lines:
        x = pad_x
        # Basic highlight color detection
        fill_color = "#d1d5db"
        if "APPROVED" in line or "✓" in line:
            fill_color = "#34d399"
        elif "PENDING" in line:
            fill_color = "#fbbf24"
        elif "DEAD LETTER" in line or "ERR" in line:
            fill_color = "#f87171"
        elif "PRISM" in line or "DASHBOARD" in line or "GATEKEEPER" in line:
            fill_color = "#60a5fa"

        draw.text((x, y), line, font=font, fill=fill_color)
        y += char_height

    output_png_path.parent.mkdir(parents=True, exist_ok=True)
    img.save(str(output_png_path))

def capture_tui_screen(name: str):
    """Capture the live tmux TUI pane content to both .txt and .png."""
    res = subprocess.run(["tmux", "capture-pane", "-t", TMUX_SESSION, "-p"], capture_output=True, text=True)
    pane_text = res.stdout
    txt_path = SCREENSHOTS_DIR / f"{name}.txt"
    png_path = SCREENSHOTS_DIR / f"{name}.png"
    
    txt_path.write_text(pane_text)
    render_terminal_to_image(pane_text, png_path)
    print(f"    📸 Captured TUI Screenshot: {png_path.name}")

def cleanup():
    print("[INIT] Cleaning up /tmp/prism and old tmux sessions...")
    subprocess.run(["tmux", "kill-session", "-t", TMUX_SESSION], stderr=subprocess.DEVNULL)
    
    if BASE_DIR.exists():
        shutil.rmtree(BASE_DIR, ignore_errors=True)
    if METRICS_FILE.exists():
        METRICS_FILE.unlink()
    if SCREENSHOTS_DIR.exists():
        shutil.rmtree(SCREENSHOTS_DIR, ignore_errors=True)
    
    SCREENSHOTS_DIR.mkdir(parents=True, exist_ok=True)
    for d in [RULES_DIR, VAULT_DIR, BASE_DIR / "pending_rules", BASE_DIR / "approved_rules", BASE_DIR / "rule_metadata"]:
        d.mkdir(parents=True, exist_ok=True)
    
    # Copy default vendor rules
    for r in (WORKSPACE / "rules").glob("*.vrl"):
        shutil.copy(r, RULES_DIR / r.name)
    print(f"[INIT] Copied {len(list(RULES_DIR.glob('*.vrl')))} base VRL rules to {RULES_DIR}")

def send_udp(payloads: list[str]):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    for p in payloads:
        sock.sendto(p.encode('utf-8'), ("127.0.0.1", UDP_PORT))
        time.sleep(0.01)
    sock.close()

def main():
    print("=" * 65)
    print("🚀 PRISM END-TO-END PIPELINE & LIVE TUI HITL VERIFICATION")
    print("=" * 65)

    cleanup()

    # 1. Start PRISM Release Binary
    print(f"\n[1] Starting PRISM release binary on UDP port {UDP_PORT}...")
    prism_proc = subprocess.Popen(
        [str(PRISM_BIN), "--udp-bind-addr", f"127.0.0.1:{UDP_PORT}", "--vault-dir", str(VAULT_DIR), "--batch-size", "5"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    time.sleep(2)
    assert prism_proc.poll() is None, "PRISM binary failed to start!"
    print(f"    ✓ PRISM binary running (PID: {prism_proc.pid})")

    # 2. Ingest Known Fortinet Logs
    print("\n[2] Ingesting Fortinet traffic logs (known vendor)...")
    fortinet_sample = (
        '<134>date=2024-01-15 time=08:23:41 devname="FGT-DC-01" devid="FG3H0E5018902345" logid="0000000013" '
        'type="traffic" subtype="forward" level="notice" vd="root" eventtime=1705312221 srcip=10.10.20.45 '
        'srcport=52341 srcintf="port5" srcintfpolicy="lan" dstip=203.0.113.25 dstport=443 dstintf="port1" '
        'dstintfpolicy="wan" sessionid=847293651 proto=6 action="accept" sentbyte=15234 rcvdbyte=892451 duration=45\n'
    )
    send_udp([fortinet_sample] * 10)
    time.sleep(3)

    # Check metrics
    with open(METRICS_FILE, "r") as f:
        metrics = json.load(f)
    print(f"    ✓ Processed: {metrics.get('processed')}, DLQ: {metrics.get('dlq')}, Fortinet: {metrics.get('telemetry', {}).get('fortinet')}")
    assert metrics.get('processed', 0) >= 10, "Expected at least 10 processed Fortinet logs"
    assert metrics.get('dlq', 0) == 0, "Expected 0 DLQ logs for Fortinet"

    # Check Merkle ledger
    ledger_file = VAULT_DIR / "ledger.log"
    assert ledger_file.exists(), "Ledger log must exist"
    ledger_content = ledger_file.read_text().strip().splitlines()
    print(f"    ✓ Merkle Root generated in ledger ({len(ledger_content)} batches): {ledger_content[-1][:16]}...")

    # 3. Ingest Unknown Real Logs (iptables from data/real_corpora/iptables.log)
    print("\n[3] Ingesting unknown real logs from data/real_corpora/iptables.log...")
    iptables_path = WORKSPACE / "data/real_corpora/iptables.log"
    with open(iptables_path, "r") as f:
        iptables_lines = [line.strip() for line in f if line.strip()][:10]

    send_udp(iptables_lines)
    time.sleep(3)

    with open(METRICS_FILE, "r") as f:
        metrics = json.load(f)
    print(f"    ✓ Processed: {metrics.get('processed')}, DLQ: {metrics.get('dlq')}")
    assert metrics.get('dlq', 0) >= 10, "Expected DLQ count >= 10 for unknown iptables logs"

    # 4. Launch PRISM TUI in tmux virtual terminal
    print("\n[4] Starting PRISM TUI in tmux virtual terminal (120x35)...")
    subprocess.run([
        "tmux", "new-session", "-d", "-s", TMUX_SESSION, "-x", "120", "-y", "35",
        str(PRISM_TUI_BIN)
    ], check=True)
    time.sleep(2)
    capture_tui_screen("01_dashboard_with_dlq")

    # 5. Start AI Brain to cluster, triage with Laya, and generate VRL
    print("\n[5] Starting AI Brain (prism-brain/main.py) to triage DLQ with Laya...")
    brain_proc = subprocess.Popen(
        [sys.executable, "prism-brain/main.py", "--vault-dir", str(VAULT_DIR), "--base-dir", str(BASE_DIR), "--rules-dir", str(RULES_DIR), "--poll-interval", "1"],
        cwd=str(WORKSPACE),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    print("    Waiting for AI Brain to cluster, triage with Laya, and generate VRL...")
    pending_rule = None
    for _ in range(40):
        time.sleep(2)
        for mf in (BASE_DIR / "rule_metadata").glob("*.json"):
            with open(mf, "r") as f:
                data = json.load(f)
            if data.get("state") == "pending":
                pending_rule = data
                break
        if pending_rule:
            break

    assert pending_rule is not None, "AI Brain failed to submit pending rule!"
    print(f"    ✓ AI Brain generated rule: {pending_rule['rule_id']}")
    print(f"    ✓ Triaged device type: {pending_rule['device_type']}")
    print(f"    ✓ State: {pending_rule['state']}")
    print(f"    ✓ VRL Path: {pending_rule['vrl_path']}")

    # 6. Navigate TUI to Gatekeeper Tab
    print("\n[6] Navigating TUI to Gatekeeper Tab (Tab -> Tab)...")
    time.sleep(2) # Give TUI poll loop a moment to ingest new rule metadata
    subprocess.run(["tmux", "send-keys", "-t", TMUX_SESSION, "Tab"], check=True)
    time.sleep(0.5)
    subprocess.run(["tmux", "send-keys", "-t", TMUX_SESSION, "Tab"], check=True)
    time.sleep(1.5)
    capture_tui_screen("02_gatekeeper_pending")

    # 7. Approve Rule via TUI HitL keypress ('a')
    print("\n[7] Triggering HitL Approval through TUI keypress ('a')...")
    subprocess.run(["tmux", "send-keys", "-t", TMUX_SESSION, "a"], check=True)
    time.sleep(2)
    capture_tui_screen("03_gatekeeper_approved")

    # Verify rule file was deployed and metadata is approved
    meta_path = BASE_DIR / "rule_metadata" / f"{pending_rule['rule_id']}.json"
    with open(meta_path, "r") as f:
        approved_meta = json.load(f)
    assert approved_meta.get("state") == "approved", "TUI failed to approve rule in metadata!"
    vrl_deployed = Path(approved_meta.get("vrl_path", ""))
    assert vrl_deployed.exists(), f"Approved VRL not found at {vrl_deployed}!"
    print(f"    ✓ TUI successfully transitioned rule to APPROVED and deployed to {vrl_deployed.name}")

    # 8. Verify Hot-Reload & DLQ Re-Parsing in TUI
    print("\n[8] Waiting for PRISM VrlEngine hot-reload & automatic DLQ re-parser...")
    time.sleep(5)

    with open(METRICS_FILE, "r") as f:
        metrics = json.load(f)
    print(f"    ✓ Updated Metrics: Processed={metrics.get('processed')}, DLQ={metrics.get('dlq')}")
    assert metrics.get('processed', 0) > 10, "Expected DLQ re-parser to increment processed events!"

    # Navigate back to Dashboard in TUI
    subprocess.run(["tmux", "send-keys", "-t", TMUX_SESSION, "Tab"], check=True)
    time.sleep(0.5)
    subprocess.run(["tmux", "send-keys", "-t", TMUX_SESSION, "Tab"], check=True)
    time.sleep(1.5)
    capture_tui_screen("04_dashboard_dlq_reparsed")

    # 9. Ingest 10 more real iptables logs - directly processed now!
    print("\n[9] Sending 10 more real iptables logs (now matching approved dynamic rule)...")
    prev_processed = metrics.get('processed', 0)
    prev_dlq = metrics.get('dlq', 0)
    send_udp(iptables_lines)
    time.sleep(3)

    with open(METRICS_FILE, "r") as f:
        final_metrics = json.load(f)
    print(f"    ✓ Final Metrics: Processed={final_metrics.get('processed')}, DLQ={final_metrics.get('dlq')}")
    assert final_metrics.get('processed', 0) >= prev_processed + 10, "Expected new iptables logs to be processed directly!"
    assert final_metrics.get('dlq', 0) == prev_dlq, "Expected DLQ count to remain constant after rule approval!"

    time.sleep(1)
    capture_tui_screen("05_final_dashboard")

    # 10. Clean shutdown
    print("\n[10] Shutting down services cleanly...")
    subprocess.run(["tmux", "send-keys", "-t", TMUX_SESSION, "q"])
    time.sleep(1)
    subprocess.run(["tmux", "kill-session", "-t", TMUX_SESSION], stderr=subprocess.DEVNULL)
    
    prism_proc.terminate()
    brain_proc.terminate()
    prism_proc.wait(timeout=5)
    brain_proc.wait(timeout=5)

    # Save final metrics copy to workspace root
    shutil.copy(METRICS_FILE, WORKSPACE / "metrics.json")
    print(f"    ✓ Saved final metrics to {WORKSPACE / 'metrics.json'}")

    print("\n" + "=" * 65)
    print("🎉 FULL PIPELINE VERIFICATION PASSED WITH LIVE TUI HITL!")
    print(f"📸 Screenshots saved to: {SCREENSHOTS_DIR}")
    print("=" * 65)

if __name__ == "__main__":
    main()
