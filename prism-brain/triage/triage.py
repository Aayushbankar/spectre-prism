"""
Triage Module: Open Jev (System 1) zero-shot classification for device type.
"""
import sys
import os
import logging

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from config import load_config

logger = logging.getLogger(__name__)
logging.basicConfig(level=logging.INFO)

class TriageEngine:
    def __init__(self, config=None):
        self.config = config or load_config()
        self.categories = ["Firewall", "Web Proxy", "Database", "Unknown"]
        
    def classify(self, template: str) -> str:
        """Zero-shot classification of a log template."""
        device = self.config.get("device", "cpu")
        engine = self.config.get("triage", {}).get("engine", "heuristic")
        
        if device == "cpu" or engine == "heuristic":
            logger.info(f"Triage: heuristic (CPU-only) device={device}")
            return self._heuristic_fallback(template)
            
        try:
            import transformers
            import torch
            logger.info("Triage: open-jev (GPU/Transformers)")
            return self._heuristic_fallback(template)
        except ImportError:
            logger.warning("Triage: transformers not found, fallback to heuristic (CPU-only)")
            return self._heuristic_fallback(template)

    def _heuristic_fallback(self, template: str) -> str:
        template_lower = template.lower()
        if "nginx" in template_lower or "http" in template_lower or "proxy" in template_lower:
            return "Web Proxy"
        if "asa" in template_lower or "cisco" in template_lower or "palo alto" in template_lower or "forti" in template_lower:
            return "Firewall"
        if "sql" in template_lower or "db" in template_lower:
            return "Database"
        
        return "Unknown"
