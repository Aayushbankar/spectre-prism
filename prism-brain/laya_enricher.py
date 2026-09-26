#!/usr/bin/env python3
"""
Laya Enrichment Service for PRISM Drain Templates
Provides OCSF class_uid, vendor, confidence, and detector terms for Drain templates.
"""

import json
import sys
import argparse
from typing import List, Dict, Any
from dataclasses import dataclass, asdict

import warnings
warnings.filterwarnings("ignore", category=RuntimeWarning)

try:
    import laya
    from laya import Router
    HAS_LAYA = True
except ImportError:
    HAS_LAYA = False


@dataclass
class EnrichedTemplate:
    template_id: str
    pattern: List[str]
    count: int
    samples: List[str]
    specificity: float
    class_uid: int
    category_uid: int
    vendor: str
    confidence: float
    detector_terms: List[str]

class LayaEnricher:
    def __init__(self, device: str = "cpu", preload: bool = True):
        self.device = device
        self.has_laya = HAS_LAYA
        if self.has_laya:
            try:
                self.router = Router(device=device, preload=False)
                if preload:
                    try:
                        self.router.preload(["english"])
                    except Exception as e:
                        print(f"Warning: preload english failed ({e}), will load on demand", file=sys.stderr)
            except Exception as e:
                print(f"Warning: failed to initialize Laya Router ({e}), falling back to heuristic", file=sys.stderr)
                self.router = None
        else:
            self.router = None
        
        # OCSF class mapping
        self.class_map = {
            "network_activity": (4001, 4),
            "authentication": (3001, 3),
            "web_activity": (5001, 5),
            "file_activity": (8001, 8),
            "process_activity": (1001, 1),
            "dns_activity": (6001, 6),
        }
    
    def _heuristic_classify(self, tmpl: Dict) -> tuple:
        text = " ".join(tmpl.get("pattern", []) + tmpl.get("samples", [])).lower()
        if any(k in text for k in ["iptables", "netfilter", "kernel:", "inbound tcp", "bridge"]):
            vendor = "linux"
        elif any(k in text for k in ["%asa", "cisco"]):
            vendor = "cisco"
        elif any(k in text for k in ["fortigate", "fortinet", "devname=", "devid="]):
            vendor = "fortinet"
        elif any(k in text for k in ["palo alto", "pan-os", "threat"]):
            vendor = "palo_alto"
        elif any(k in text for k in ["snort", "[classification:"]):
            vendor = "snort"
        elif any(k in text for k in ["sshd", "openssh", "password for", "accepted publickey"]):
            vendor = "openssh"
        elif any(k in text for k in ["suricata", "eve.json"]):
            vendor = "suricata"
        elif any(k in text for k in ["zeek", "bro"]):
            vendor = "zeek"
        else:
            vendor = "linux"

        if any(k in text for k in ["ssh", "login", "password", "auth", "pam"]):
            dev_class = "authentication"
        elif any(k in text for k in ["dns", "query[a]"]):
            dev_class = "dns_activity"
        elif any(k in text for k in ["http", "get /", "post /"]):
            dev_class = "web_activity"
        else:
            dev_class = "network_activity"

        return dev_class, vendor

    def enrich_batch(self, templates: List[Dict]) -> List[Dict]:
        """Enrich a batch of templates with Laya classification."""
        enriched = []
        for tmpl in templates:
            if tmpl.get("count", 0) < 1 or (not tmpl.get("pattern") and not tmpl.get("samples")):
                # Skip truly empty templates - return minimal enrichment
                enriched.append({
                    "template_id": tmpl.get("id", ""),
                    "pattern": tmpl.get("pattern", []),
                    "count": tmpl.get("count", 0),
                    "samples": tmpl.get("samples", []),
                    "specificity": tmpl.get("specificity", 1.0),
                    "class_uid": 4001,
                    "category_uid": 4,
                    "vendor": "Unknown",
                    "confidence": 0.0,
                    "detector_terms": self._extract_detector_terms(tmpl.get("samples", []), tmpl.get("pattern", []))
                })
                continue
            
            if self.router is not None:
                # Use template pattern + sample for classification
                template_text = " ".join(tmpl.get("pattern", []))
                if tmpl.get("samples"):
                    prompt_doc = f"Template: {template_text}\nSample: {tmpl['samples'][0]}"
                else:
                    prompt_doc = template_text

                result = self.router.predict(
                    state={"document": prompt_doc},
                    questions=self._get_questions()
                )
                
                # Parse Laya result
                device_class = result["answers"]["device_class"]["choice"]
                class_uid, category_uid = self.class_map.get(device_class, (4001, 4))
                vendor = result["answers"]["vendor"]["choice"]
                dev_conf = float(result["answers"].get("device_class", {}).get("answer_confidence", 0.8))
                ven_conf = float(result["answers"].get("vendor", {}).get("answer_confidence", 0.8))
                confidence = round((dev_conf + ven_conf) / 2.0, 2)
            else:
                device_class, vendor = self._heuristic_classify(tmpl)
                class_uid, category_uid = self.class_map.get(device_class, (4001, 4))
                confidence = 0.85
            
            # Extract detector terms
            detector_terms = self._extract_detector_terms(tmpl.get("samples", []), tmpl.get("pattern", []))
            
            enriched.append({
                "template_id": tmpl["id"],
                "pattern": tmpl["pattern"],
                "count": tmpl["count"],
                "samples": tmpl.get("samples", []),
                "specificity": tmpl.get("specificity", 1.0),
                "class_uid": class_uid,
                "category_uid": category_uid,
                "vendor": vendor,
                "confidence": confidence,
                "detector_terms": detector_terms
            })
        
        return enriched
    
    def _get_questions(self) -> Dict:
        return {
            "device_class": {
                "type": "choice",
                "instructions": "Classify this log template into OCSF device class",
                "criteria": {
                    "network_activity": "Firewall, router, switch, network device traffic logs",
                    "authentication": "Login, logon, SSH, VPN, RADIUS, TACACS authentication events",
                    "web_activity": "HTTP, HTTPS, proxy, web server, API gateway logs",
                    "file_activity": "File creation, modification, deletion, access logs",
                    "process_activity": "Process creation, execution, command line logs",
                    "dns_activity": "DNS query, response, resolution logs"
                }
            },
            "vendor": {
                "type": "choice",
                "instructions": "Identify the vendor",
                "criteria": {
                    "fortinet": "Fortinet, FortiGate, FG",
                    "cisco": "Cisco, ASA, FTD, Meraki",
                    "palo_alto": "Palo Alto, PAN-OS, PA-",
                    "linux": "iptables, netfilter, kernel, systemd",
                    "suricata": "Suricata, EVE",
                    "zeek": "Zeek, Bro",
                    "snort": "Snort",
                    "checkpoint": "Check Point",
                    "pfsense": "pfSense, filterlog",
                    "openssh": "sshd, OpenSSH",
                    "unknown": "Cannot determine"
                }
            }
        }
    
    def _extract_detector_terms(self, samples: List[str], pattern: List[str] = None) -> List[str]:
        """Extract stable detector terms from samples and template pattern."""
        terms = []
        month_names = {"jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"}
        day_names = {"mon", "tue", "wed", "thu", "fri", "sat", "sun"}

        # 1. Daemon / process tags from samples (e.g. sshd[123]: -> sshd, kernel:, snort:)
        if samples:
            words = samples[0].split()
            # If standard syslog timestamp (Month Day Time), skip hostname at index 3 and look at daemon at index 4
            is_syslog = (len(words) >= 5 and words[0].lower() in month_names and words[1].isdigit() and ":" in words[2])
            start_idx = 4 if is_syslog else 0

            for word in words[start_idx:start_idx + 3]:
                tag = word.split("[")[0].split("(")[0].rstrip(":,")
                if len(tag) >= 3 and tag.lower() not in month_names and tag.lower() not in day_names and any(c.isalpha() for c in tag):
                    if all(tag in s for s in samples):
                        if tag not in terms:
                            terms.append(tag)

        # 2. Key-value markers from pattern and samples (e.g. SRC=, DST=, IN=)
        if pattern:
            for tok in pattern:
                if tok != "<*>" and "=" in tok:
                    key = tok.split("=")[0] + "="
                    if len(key) >= 3 and key not in terms:
                        terms.append(key)

        if samples:
            for word in samples[0].split():
                if "=" in word:
                    prefix = word.split("=")[0] + "="
                    if len(prefix) >= 3 and all(prefix in s for s in samples):
                        if prefix not in terms:
                            terms.append(prefix)

        # 3. Discriminative literals from pattern (excluding hostnames/timestamps)
        if pattern:
            is_syslog_pat = (len(pattern) >= 5 and pattern[0].lower() in month_names and any(pattern[1] == str(d) for d in range(1, 32)))
            for idx, tok in enumerate(pattern):
                if is_syslog_pat and idx == 3:
                    continue  # skip hostname
                if tok != "<*>" and len(tok) >= 3 and not tok.isdigit() and "=" not in tok:
                    clean = tok.strip("[]():,")
                    if clean.lower() not in month_names and clean.lower() not in day_names and any(c.isalpha() for c in clean):
                        if tok not in terms and clean not in terms:
                            terms.append(tok)

        # 4. Invariant phrases or keywords from samples
        if samples:
            for word in samples[0].split():
                clean = word.strip("[]():,")
                if len(clean) >= 4 and clean.lower() not in month_names and clean.lower() not in day_names and not clean.isdigit() and any(c.isalpha() for c in clean):
                    if all(clean in s for s in samples):
                        if clean not in terms and word not in terms:
                            terms.append(clean)

        return terms[:6]



def main():
    parser = argparse.ArgumentParser(description="Laya Enrichment Service for PRISM Drain")
    parser.add_argument("--input", "-i", required=True, help="Input JSON file (- for stdin)")
    parser.add_argument("--output", "-o", help="Output JSON file (default: stdout)")
    args = parser.parse_args()
    
    # Read input
    if args.input == "-":
        input_data = sys.stdin.read()
    else:
        with open(args.input, "r") as f:
            input_data = f.read()
    
    try:
        templates = json.loads(input_data)
    except json.JSONDecodeError as e:
        print(f"ERROR: Invalid JSON input: {e}", file=sys.stderr)
        sys.exit(1)
    
    # Enrich
    enricher = LayaEnricher()
    enriched = enricher.enrich_batch(templates)
    
    # Output
    output = json.dumps(enriched, indent=2)
    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
    else:
        print(output)


if __name__ == "__main__":
    main()