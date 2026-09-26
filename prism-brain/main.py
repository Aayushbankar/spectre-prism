import time
import sys
import os
import glob
import json
import re
import hashlib
import argparse

def normalize_log_signature(payload: str) -> str:
    """Generate a stable signature by stripping timestamps, IPs, ports, and session IDs."""
    sig = payload
    sig = re.sub(r'\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}', '<IP>', sig)
    sig = re.sub(r'\d{4}[-/]\d{2}[-/]\d{2}[T ]\d{2}:\d{2}:\d{2}', '<TS>', sig)
    sig = re.sub(r'\w{3}\s+\d{1,2}\s+\d{4}\s+\d{2}:\d{2}:\d{2}', '<TS>', sig)  
    sig = re.sub(r'\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2}', '<TS>', sig)
    sig = re.sub(r'(?<=[:/])\d{4,5}(?=[\s/,]|$)', '<PORT>', sig)
    sig = re.sub(r'\b\d{7,}\b', '<ID>', sig)
    return hashlib.sha256(sig.encode()).hexdigest()

# Ensure modules can be imported
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper, create_gatekeeper
from cluster.cluster import LogClusterer

def main():
    parser = argparse.ArgumentParser(description="PRISM AI Control Plane")
    parser.add_argument("--vault-dir", default="/tmp/prism/vault", help="Vault directory to scan for DLQ")
    parser.add_argument("--base-dir", default="/tmp/prism", help="Gatekeeper base directory")
    parser.add_argument("--rules-dir", default="/tmp/prism/rules", help="Active rules directory")
    parser.add_argument("--poll-interval", type=int, default=2, help="Poll interval in seconds")
    args = parser.parse_args()

    print("========================================")
    print(" PRISM AI CONTROL PLANE (HOT-RELOADER) ")
    print("========================================")
    print(f"[AI] Brain is online and scanning DLQ in {args.vault_dir}...")
    print(f"[AI] Gatekeeper base: {args.base_dir}, rules: {args.rules_dir}")
    
    triage = TriageEngine({"device": "cpu"})
    coder = VrlCoder({"coder": {"enabled": True}})
    gatekeeper = create_gatekeeper({"base_dir": args.base_dir, "rules_dir": args.rules_dir})
    clusterer = LogClusterer()
    
    # Track processed (file, line_num) to handle same-template-different-IP cases
    processed_entries = set()
    
    # Write a heartbeat file so the TUI knows we are online
    with open("/tmp/prism_ai_status", "w") as f:
        f.write("online")

    try:
        while True:
            # Update heartbeat
            with open("/tmp/prism_ai_status", "w") as f:
                f.write(str(time.time()))
                
            vault_dir = args.vault_dir
            if os.path.exists(vault_dir):
                dlq_files = glob.glob(f"{vault_dir}/dlq_*.log")
                for dlq_file in dlq_files:
                    try:
                        with open(dlq_file, "r") as f:
                            lines = f.readlines()
                        
                        for line_num, line in enumerate(lines):
                            if not line.strip():
                                continue
                            
                            # Track by file + line number to avoid reprocessing
                            entry_key = (dlq_file, line_num)
                            if entry_key in processed_entries:
                                continue
                            processed_entries.add(entry_key)
                            
                            try:
                                data = json.loads(line)
                                raw = data.get("raw_payload", data.get("payload", line))
                                if isinstance(raw, str) and raw.startswith("utf8:"):
                                    payload = raw[5:]
                                else:
                                    payload = str(raw)
                            except:
                                if "PAYLOAD=" in line:
                                    payload = line.split("PAYLOAD=", 1)[1].strip()
                                else:
                                    payload = line.strip()

                            
                            sig = normalize_log_signature(payload)
                            print(f"\n[AI] Detected Unknown Payload in DLQ (sig={sig[:8]}):\n{payload.strip()}")
                            
                            cluster_result = clusterer.process_log(payload)
                            if cluster_result['size'] > 1:
                                print(f"[AI] Template already clustered (size={cluster_result['size']}), skipping")
                                continue
                            
                            try:
                                device_type = triage.classify(payload)
                                print(f"[AI] Triaged device as: {device_type}")
                                
                                vrl_code = coder.generate_vrl(payload, device_type)
                                print(f"[AI] Generated VRL parser.")
                                
                                rule_name = f"auto_{device_type.replace(' ', '_').lower()}_{int(time.time())}"
                                
                                rule_id = gatekeeper.submit_rule(device_type, vrl_code, rule_name, payload)
                                print(f"[Gatekeeper] Rule {rule_id} submitted to HitL Approval Queue!")
                            except ValueError as e:
                                print(f"[AI] Dry-run validation failed: {e}")
                            except Exception as e:
                                print(f"[AI] Pipeline failed for payload: {e}")
                    except Exception as e:
                        print(f"[AI] Error reading DLQ {dlq_file}: {e}")
            
            time.sleep(args.poll_interval)
    except KeyboardInterrupt:
        print("\n[AI] Shutting down...")
        if os.path.exists("/tmp/prism_ai_status"):
            os.remove("/tmp/prism_ai_status")

if __name__ == "__main__":
    main()