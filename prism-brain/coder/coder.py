"""
Coder Module: Ollama Llama-3 generating VRL remap scripts and router signatures.
"""
import sys
import os
import logging
import requests
import re
from typing import Optional

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from config import load_config

logger = logging.getLogger(__name__)

class VrlCoder:
    def __init__(self, config=None):
        self.config = config or load_config()
        coder_conf = self.config.get("coder", {})
        self.host = coder_conf.get("host", "http://127.0.0.1:8080")
        self.enabled = str(coder_conf.get("enabled", "true")).lower() == "true"
        self.timeout = coder_conf.get("timeout", 30)
        self.model = coder_conf.get("model", "qwen")

    def generate_vrl(self, template: str, device_type: str) -> Optional[str]:
        """Generate a VRL script for the given log template using local Qwen2.5-Coder."""
        if not self.enabled:
            logger.info("Coder: heuristic VRL (CPU-only)")
            return self._heuristic_fallback(device_type)
            
        system_prompt = (
            "You are an expert in Vector Remap Language (VRL). "
            "You write pure VRL code without markdown formatting or explanation. "
            "Example VRL syntax for IP extraction:\n"
            ".ip = parse_regex!(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)').ip"
        )
        prompt = f"Write a VRL script to parse this {device_type} log template: {template}. Extract 'ip' into '.ip'. Just return the raw VRL code."
        
        try:
            payload = {
                "model": self.model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": prompt}
                ]
            }
            
            logger.info(f"Coder: Requesting VRL generation from {self.host}...")
            resp = requests.post(
                f"{self.host}/v1/chat/completions",
                json=payload,
                timeout=self.timeout
            )
            
            if resp.status_code == 200:
                data = resp.json()
                content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
                
                # Strip markdown code blocks
                content = re.sub(r'```\w*\n?', '', content).strip()

                logger.info(f"Coder: LLM generated VRL successfully: \n{content}")
                return content
            else:
                logger.warning(f"Coder: LLM failed with status {resp.status_code}, fallback to heuristic")

        except requests.exceptions.RequestException as e:
            logger.warning(f"Coder: LLM unreachable ({e}), fallback to heuristic (CPU-only)")
            pass
        
        return self._heuristic_fallback(device_type)
        
    def _heuristic_fallback(self, device_type: str) -> str:
        if device_type == "Web Proxy":
            return ".ip = parse_regex!(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)').ip"
        return '.ip = "0.0.0.0"'
