"""
Coder Module: Ollama Llama-3 generating VRL remap scripts and router signatures.
"""
import sys
import os
import logging
import requests
from typing import Optional

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from config import load_config

logger = logging.getLogger(__name__)

class VrlCoder:
    def __init__(self, config=None):
        self.config = config or load_config()
        coder_conf = self.config.get("coder", {})
        self.host = coder_conf.get("host", "http://localhost:11434")
        self.enabled = str(coder_conf.get("enabled", "false")).lower() == "true"
        self.timeout = coder_conf.get("timeout", 2)
        self.model = coder_conf.get("model", "llama3")

    def generate_vrl(self, template: str, device_type: str) -> Optional[str]:
        """Generate a VRL script for the given log template using Llama-3."""
        if not self.enabled:
            logger.info("Coder: heuristic VRL (CPU-only)")
            return self._heuristic_fallback(device_type)

        prompt = f"Write a VRL script to parse this {device_type} log template: {template}. Extract 'ip' and map it to OCSF. Just return the VRL code."
        try:
            resp = requests.post(
                f"{self.host}/api/generate",
                json={"model": self.model, "prompt": prompt, "stream": False},
                timeout=self.timeout
            )
            if resp.status_code == 200:
                logger.info("Coder: Ollama generated VRL")
                return resp.json().get("response", "")
        except requests.exceptions.RequestException:
            logger.warning("Coder: Ollama unreachable, fallback to heuristic (CPU-only)")
            pass
        
        return self._heuristic_fallback(device_type)
        
    def _heuristic_fallback(self, device_type: str) -> str:
        if device_type == "Web Proxy":
            return ".ip = parse_regex!(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)').ip"
        return '.ip = "0.0.0.0"'
