import time
import sys
import os
import glob
import json

# Ensure modules can be imported
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))

from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper

def main():
    print("========================================")
    print(" PRISM AI CONTROL PLANE (HOT-RELOADER) ")
    print("========================================")
    print("[AI] Brain is online and scanning DLQ...")
    
    triage = TriageEngine({"device": "cpu"})
    coder = VrlCoder({"coder": {"enabled": True}})
    gatekeeper = Gatekeeper()
    
    processed_payloads = set()
    
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
                                
                                if payload not in processed_payloads:
                                    print(f"\n[AI] Detected Unknown Payload in DLQ:\n{payload}")
                                    processed_payloads.add(payload)
                                    
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
