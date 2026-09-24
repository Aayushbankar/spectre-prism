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

    def approve_and_deploy(self, device_type: str, vrl_code: str, signature: str) -> str:
        """Approve and deploy the new VRL rule to the rules directory for Rust hot-reload."""
        rule_id = str(uuid.uuid4())[:8]
        vrl_path = os.path.join(self.rules_dir, f"{device_type.replace(' ', '_').lower()}_{rule_id}.vrl")
        
        with open(vrl_path, "w") as f:
            f.write(vrl_code)
            
        yaml_path = os.path.join(self.rules_dir, f"{device_type.replace(' ', '_').lower()}_{rule_id}.yaml")
        with open(yaml_path, "w") as f:
            f.write(f"signature: \"{signature}\"\nvrl_file: \"{vrl_path}\"\n")
            
        return vrl_path
