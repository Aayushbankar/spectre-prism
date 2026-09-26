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

try:
    import laya
    from laya import Router
except ImportError:
    print("ERROR: laya not installed. Run: pip install laya==0.3.16 && huggingface-cli download convaiinnovations/laya", file=sys.stderr)
    sys.exit(1)

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
        self.router = Router(preload=preload)
        self.device = device
        
        # OCSF class mapping
        self.class_map = {
            "network_activity": (4001, 4),
            "authentication": (3001, 3),
            "web_activity": (5001, 5),
            "file_activity": (8001, 8),
            "process_activity": (1001, 1),
            "dns_activity": (6001, 6),
        }
        
        # Classification questions for Laya
        self.questions = {
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
            },
            "confidence": {
                "type": "score",
                "instructions": "How confident are you in the classification?",
                "criteria": ["low", "medium", "high"]
            }
        }
    
    def enrich_batch(self, templates: List[Dict]) -> List[Dict]:
        """Enrich a batch of templates with Laya classification."""
        enriched = []
        for tmpl in templates:
            if tmpl.get("count", 0) < 2:
                # Skip singletons - return minimal enrichment
                enriched.append({
                    "template_id": tmpl["id"],
                    "pattern": tmpl["pattern"],
                    "count": tmpl["count"],
                    "samples": tmpl["samples"],
                    "specificity": tmpl.get("specificity", 1.0),
                    "class_uid": 4001,
                    "category_uid": 4,
                    "vendor": "Unknown",
                    "confidence": 0.0,
                    "detector_terms": tmpl.get("detector_terms", [])
                })
                continue
            
            # Use template pattern for classification
            template_text = " ".join(tmpl["pattern"])
            result = self.router.predict(
                state={"document": template_text},
                questions=self._get_questions()
            )
            
            # Parse Laya result
            class_map = {
                "network_activity": (4001, 4),
                "authentication": (3001, 3),
                "web_activity": (5001, 5),
                "file_activity": (8001, 8),
                "process_activity": (1001, 1),
                "dns_activity": (6001, 6),
            }
            
            device_class = result["answers"]["device_class"]["choice"]
            class_uid, category_uid = class_map.get(device_class, (4001, 4))
            
            vendor = result["answers"]["vendor"]["choice"]
            confidence_map = {"low": 0.3, "medium": 0.6, "high": 0.9}
            confidence = confidence_map.get(result["answers"]["confidence"]["score"], 0.5)
            
            # Extract detector terms
            detector_terms = self._extract_detector_terms(tmpl.get("samples", []))
            
            enriched.append({
                "template_id": tmpl["id"],
                "pattern": tmpl["pattern"],
                "count": tmpl["count"],
                "samples": tmpl["samples"],
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
            },
            "confidence": {
                "type": "score",
                "instructions": "How confident are you in the classification?",
                "criteria": ["low", "medium", "high"]
            }
        }
    
    def _extract_detector_terms(self, samples: List[str]) -> List[str]:
        """Extract stable detector terms from samples using Laya."""
        if not samples:
            return []
        
        # Use Laya to find stable discriminative terms
        # Custom question: "What fixed text identifies this log format?"
        try:
            # Use first few samples for detector extraction
            sample_text = "\n".join(samples[:5])
            questions = {
                "detector_terms": {
                    "type": "choice",
                    "instructions": "What fixed text tokens (literals) appear in EVERY log line and identify this specific log format? Choose the most specific stable tokens.",
                    "criteria": {
                        "field_markers": "Field names followed by = (e.g., srcip=, dstip=, action=)",
                        "vendor_tags": "Vendor/product identifiers (e.g., devname=, %ASA-, CEF:)",
                        "event_types": "Event type markers (e.g., type=traffic, event_type=alert)",
                        "none": "No stable identifying tokens found"
                    }
                }
            }
            
            result = self.router.predict(
                state={"document": "\n".join(samples[:3])},
                questions={
                    "detector_terms": {
                        "type": "choice",
                        "instructions": "What fixed text tokens appear in EVERY log line and identify this log format?",
                        "criteria": {
                            "field_markers": "Field names with = (srcip=, dstip=, action=)",
                            "vendor_tags": "Vendor identifiers (devname=, %ASA-, CEF:)",
                            "event_types": "Event type markers (type=traffic, event_type=alert)",
                            "none": "No stable tokens found"
                        }
                    }
                }
            )
            
            terms = []
            choice = result["answers"]["detector_terms"]["choice"]
            if choice == "field_markers":
                terms = ["srcip=", "dstip=", "action=", "srcport=", "dstport=", "proto="]
            elif choice == "vendor_tags":
                terms = ["devname=", "%ASA-", "CEF:", "LEEF:"]
            elif choice == "event_types":
                terms = ["type=traffic", "event_type=alert", "action="]
            
            return terms
        except Exception:
            # Fallback to heuristic extraction
            return self._fallback_detector_terms(samples)
    
    def _fallback_detector_terms(self, samples: List[str]) -> List[str]:
        """Fallback heuristic detector extraction."""
        if not samples:
            return []
        
        first = samples[0]
        terms = []
        for token in first.split():
            if token.endswith('=') and len(token) > 2:
                # Check if this token= appears in all samples
                if all(token in s for s in samples):
                    terms.append(token)
            elif len(token) >= 6 and all(token in s for s in samples):
                terms.append(token)
        return terms[:5]


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