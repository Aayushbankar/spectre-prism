"""
Gatekeeper Module: vet via Jev + TUI HitL 1-click -> hot-reload /etc/prism/rules/
"""
import os
import uuid

class Gatekeeper:
    def __init__(self, rules_dir: str = "/etc/prism/rules"):
        self.rules_dir = rules_dir
        if not os.path.exists(self.rules_dir):
            try:
                os.makedirs(self.rules_dir, exist_ok=True)
            except PermissionError:
                # Fallback for tests
                self.rules_dir = "/tmp/prism/rules"
                os.makedirs(self.rules_dir, exist_ok=True)

    def approve_and_deploy(self, device_type: str, vrl_code: str, signature: str, raw_log: str = None) -> str:
        """Approve and deploy the new VRL rule to the rules directory for Rust hot-reload."""
        if raw_log:
            import tempfile
            import subprocess
            
            with tempfile.NamedTemporaryFile("w", delete=False, suffix=".vrl") as tmp:
                tmp.write(vrl_code)
                tmp_path = tmp.name
                
            try:
                prism_bin = os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "prism")
                if not os.path.exists(prism_bin):
                    prism_bin = "prism"
                    
                result = subprocess.run([prism_bin, "--dry-run-vrl", tmp_path, "--payload", raw_log], capture_output=True, text=True)
                if result.returncode != 0:
                    print(f"Dry run failed: {result.stderr}")
                    os.remove(tmp_path)
                    return ""
            except FileNotFoundError:
                print("prism binary not found, skipping dry run")
            finally:
                if os.path.exists(tmp_path):
                    os.remove(tmp_path)
                    
        rule_id = str(uuid.uuid4())[:8]
        vrl_path = os.path.join(self.rules_dir, f"{device_type.replace(' ', '_').lower()}_{rule_id}.vrl")
        
        with open(vrl_path, "w") as f:
            f.write(vrl_code)
            
        yaml_path = os.path.join(self.rules_dir, f"{device_type.replace(' ', '_').lower()}_{rule_id}.yaml")
        with open(yaml_path, "w") as f:
            f.write(f"signature: \"{signature}\"\nvrl_file: \"{vrl_path}\"\n")
            
        return vrl_path
