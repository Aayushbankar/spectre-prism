"""
Triage Module: Open Jev (System 1) zero-shot classification for device type.
"""

class TriageEngine:
    def __init__(self):
        self.categories = ["Firewall", "Web Proxy", "Database", "Unknown"]
        
    def classify(self, template: str) -> str:
        """Zero-shot classification of a log template."""
        # Simple heuristic fallback for test gating without full torch/transformers overhead
        template_lower = template.lower()
        if "nginx" in template_lower or "http" in template_lower or "proxy" in template_lower:
            return "Web Proxy"
        if "asa" in template_lower or "cisco" in template_lower or "palo alto" in template_lower or "forti" in template_lower:
            return "Firewall"
        if "sql" in template_lower or "db" in template_lower:
            return "Database"
        
        return "Unknown"
