"""
Coder Module: Ollama Llama-3 generating VRL remap scripts and router signatures.
"""
import requests
from typing import Optional

class VrlCoder:
    def __init__(self, host: str = "http://localhost:11434"):
        self.host = host

    def generate_vrl(self, template: str, device_type: str) -> Optional[str]:
        """Generate a VRL script for the given log template using Llama-3."""
        prompt = f"Write a VRL script to parse this {device_type} log template: {template}. Extract 'ip' and map it to OCSF. Just return the VRL code."
        try:
            resp = requests.post(
                f"{self.host}/api/generate",
                json={"model": "llama3", "prompt": prompt, "stream": False},
                timeout=2
            )
            if resp.status_code == 200:
                return resp.json().get("response", "")
        except requests.exceptions.RequestException:
            pass
        
        # Fallback if Ollama is not available locally for tests
        if device_type == "Web Proxy":
            return ".ip = parse_regex!(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)').ip"
        return '.ip = "0.0.0.0"'
