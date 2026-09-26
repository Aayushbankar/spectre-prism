import time
import sys
import os
import glob
import json
import re
import hashlib

def normalize_log_signature(payload: str) -> str:
    """Generate a stable signature by stripping timestamps, IPs, ports, and session IDs."""
    sig = payload
    # Strip IP addresses
    sig = re.sub(r'\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}', '<IP>', sig)
    # Strip timestamps (various formats)
    sig = re.sub(r'\d{4}[-/]\d{2}[-/]\d{2}[T ]\d{2}:\d{2}:\d{2}', '<TS>', sig)
    sig = re.sub(r'\w{3}\s+\d{1,2}\s+\d{4}\s+\d{2}:\d{2}:\d{2}', '<TS>', sig)  
    sig = re.sub(r'\d{2}/\w{3}/\d{4}:\d{2}:\d{2}:\d{2}', '<TS>', sig)
    # Strip port numbers (standalone 4-5 digit numbers)
    sig = re.sub(r'(?<=[:/])\d{4,5}(?=[\s/,]|$)', '<PORT>', sig)
    # Strip session/connection IDs (large numbers)
    sig = re.sub(r'\b\d{7,}\b', '<ID>', sig)
    return hashlib.sha256(sig.encode()).hexdigest()

# Ensure modules can be imported
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper
from cluster.cluster import LogClusterer

def main():
    print("========================================")
    print(" PRISM AI CONTROL PLANE (HOT-RELOADER) ")
    print("========================================")
    print("[AI] Brain is online and scanning DLQ...")
    
    triage = TriageEngine({"device": "cpu"})
    coder = VrlCoder({"coder": {"enabled": True}})
    gatekeeper = Gatekeeper()
    clusterer = LogClusterer()
    
    processed_signatures = set()
    
    # Write a heartbeat file so the TUI knows we are online
    with open("/tmp/prism_ai_status", "w") as f:
        f.write("online")

    try:
        while True:
            # Update heartbeat
            with open("/tmp/prism_ai_status", "w") as f:
                f.write(str(time.time()))
                
            vault_dir = "/tmp/prism/vault"
            if os.path.exists(vault_dir):
                dlq_files = glob.glob(f"{vault_dir}/dlq_*.log")
                for dlq_file in dlq_files:
                    try:
                        with open(dlq_file, "r") as f:
                            for line in f:
                                if not line.strip():
                                    continue
                                
                                try:
                                    data = json.loads(line)
                                    payload = data.get("payload", line)
                                except:
                                    payload = line
                                
                                sig = normalize_log_signature(payload)
                                if sig not in processed_signatures:
                                    print(f"\n[AI] Detected Unknown Payload in DLQ:\n{payload}")
                                    processed_signatures.add(sig)
                                    
                                    cluster_result = clusterer.process_log(payload)
                                    if cluster_result['size'] > 1:
                                        continue  # Template already seen, skip
                                    
                                    try:
                                        device_type = triage.classify(payload)
                                        print(f"[AI] Triaged device as: {device_type}")
                                        
                                        vrl_code = coder.generate_vrl(payload, device_type)
                                        print(f"[AI] Generated VRL parser.")
                                        
                                        rule_name = f"auto_{device_type.replace(' ', '_').lower()}_{int(time.time())}"
                                        
                                        gatekeeper.approve_and_deploy(device_type, vrl_code, rule_name, payload)
                                        print(f"[Gatekeeper] Rule {rule_name}.vrl sent to HitL Approval Queue!")
                                    except Exception as e:
                                        print(f"[AI] Pipeline failed for payload: {e}")
                    except Exception:
                        pass
            
            time.sleep(2)
    except KeyboardInterrupt:
        print("\n[AI] Shutting down...")
        if os.path.exists("/tmp/prism_ai_status"):
            os.remove("/tmp/prism_ai_status")

if __name__ == "__main__":
    main()
