"""
Coder Module: Ollama Llama-3 generating VRL remap scripts with robust prompt engineering.
"""
import sys
import os
import logging
import requests
import re
import json
from typing import Optional, Dict, Any

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from config import load_config

logger = logging.getLogger(__name__)

VRL_EXAMPLES = {
    "Network Activity": '''
# Network Activity (class_uid=4001)
.class_uid = 4001
.category_uid = 4
.type_uid = 400101
m_src, err = parse_regex(string!(.message), r'SRC=(?P<src>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  .src_endpoint.ip = m_src.src
}
m_dst, err = parse_regex(string!(.message), r'DST=(?P<dst>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  .dst_endpoint.ip = m_dst.dst
}
m_ip, err = parse_regex(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  if .src_endpoint.ip == null {
    .src_endpoint.ip = m_ip.ip
  }
  if .dst_endpoint.ip == null {
    .dst_endpoint.ip = m_ip.ip
  }
}
''',
    "Firewall": '''
# Firewall (class_uid=4001)
.class_uid = 4001
.category_uid = 4
.type_uid = 400101
m_src, err = parse_regex(string!(.message), r'SRC=(?P<src>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  .src_endpoint.ip = m_src.src
}
m_dst, err = parse_regex(string!(.message), r'DST=(?P<dst>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  .dst_endpoint.ip = m_dst.dst
}
m_ip, err = parse_regex(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  if .src_endpoint.ip == null {
    .src_endpoint.ip = m_ip.ip
  }
  if .dst_endpoint.ip == null {
    .dst_endpoint.ip = m_ip.ip
  }
}
''',
    "Authentication": '''
# Authentication (class_uid=3001)
.class_uid = 3001
.category_uid = 3
.type_uid = 300101
m_ip, err = parse_regex(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  .src_endpoint.ip = m_ip.ip
  .dst_endpoint.ip = m_ip.ip
}
m_user, err = parse_regex(string!(.message), r'user[=:]\"?(?P<user>[^\\s\"]+)')
if err == null {
  .user.name = m_user.user
}
''',
    "Web Activity": '''
# Web Activity (class_uid=5001)
.class_uid = 5001
.category_uid = 5
.type_uid = 500101
m_ip, err = parse_regex(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  .src_endpoint.ip = m_ip.ip
  .dst_endpoint.ip = m_ip.ip
}
m_url, err = parse_regex(string!(.message), r'(?P<url>https?://[^\\s]+)')
if err == null {
  .http.request.url = m_url.url
}
m_method, err = parse_regex(string!(.message), r'(?P<method>GET|POST|PUT|DELETE|HEAD)')
if err == null {
  .http.request.method = m_method.method
}
m_status, err = parse_regex(string!(.message), r'(?P<status>\\d{3})')
if err == null {
  .http.response.status_code = to_int!(m_status.status)
}
''',
    "File Activity": '''
# File Activity (class_uid=8001)
.class_uid = 8001
.category_uid = 8
.type_uid = 800101
m_ip, err = parse_regex(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {
  .src_endpoint.ip = m_ip.ip
  .dst_endpoint.ip = m_ip.ip
}
m_file, err = parse_regex(string!(.message), r'(?P<file>[\\w\\-. ]+\\.(?:exe|dll|pdf|docx?|xlsx?))')
if err == null {
  .file.name = m_file.file
}
m_path, err = parse_regex(string!(.message), r'(?P<path>[A-Za-z]:\\\\[^\\s]+|/[^\\s]+)')
if err == null {
  .file.path = m_path.path
}
'''
}

VRL_SYNTAX_RULES = """
CRITICAL VRL SYNTAX RULES (MUST FOLLOW):
1. Use `.field = value` for assignment (NOT `:=` or `=`)
2. DO NOT assign objects to `.message` (NEVER `.message = parse_syslog!(.message)`). `.message` is already the input string.
3. Use `m, err = parse_regex(string!(.message), r'pattern')` to extract fields safely.
4. Check `if err == null { ... }` before accessing captured regex fields `m.field`.
5. String literals use double quotes: "value"
6. Regex patterns use raw strings: r'pattern'
7. Comments start with #
8. NO markdown formatting - return ONLY raw VRL code
9. Always set .class_uid, .category_uid, .type_uid as integers
10. Use .src_endpoint.ip and .dst_endpoint.ip for network events
11. For user fields: .user.name = "value"
12. For HTTP: .http.request.method, .http.request.url, .http.response.status_code
13. For files: .file.name, .file.path
"""

class VrlCoder:
    def __init__(self, config=None):
        self.config = config or load_config()
        coder_conf = self.config.get("coder", {})
        self.host = coder_conf.get("host", "http://127.0.0.1:8080")
        self.enabled = str(coder_conf.get("enabled", "true")).lower() == "true"
        self.timeout = coder_conf.get("timeout", 15)
        self.model = coder_conf.get("model", "qwen2.5-coder:7b")
        self.max_retries = 3

    def generate_vrl(self, template: str, device_type: str) -> Optional[str]:
        """Generate a VRL script for the given log template using local LLM."""
        if not self.enabled:
            logger.info("Coder: heuristic VRL (CPU-only)")
            return self._heuristic_fallback(device_type)
            
        for attempt in range(self.max_retries):
            vrl = self._generate_with_llm(template, device_type, attempt)
            if vrl and self._validate_vrl(vrl, sample_log=template):
                logger.info(f"Coder: LLM generated valid VRL on attempt {attempt + 1}")
                return vrl
            logger.warning(f"Coder: Attempt {attempt + 1} failed validation, retrying...")
            
        logger.warning("Coder: All LLM attempts failed, falling back to heuristic")
        return self._heuristic_fallback(device_type)
        
    def _generate_with_llm(self, template: str, device_type: str, attempt: int) -> Optional[str]:
        """Generate VRL using LLM with progressive prompt refinement."""
        example = VRL_EXAMPLES.get(device_type, VRL_EXAMPLES["Network Activity"])
        
        if attempt == 0:
            system_prompt = f"""You are an expert in Vector Remap Language (VRL) for OCSF mapping.
{VRL_SYNTAX_RULES}

OCSF Class UIDs:
- Network Activity: 4001 (category 4)
- Authentication: 3001 (category 3)  
- Web Activity: 5001 (category 5)
- File Activity: 8001 (category 8)

EXAMPLE for {device_type}:
{example}

Return ONLY the VRL code. No explanations."""
        elif attempt == 1:
            system_prompt = f"""You are an expert in Vector Remap Language (VRL).
{VRL_SYNTAX_RULES}

PREVIOUS ATTEMPT FAILED VALIDATION. COMMON ERRORS TO AVOID:
- Do NOT use `:=` for assignment
- Do NOT assign anything to `.message`
- Do NOT use markdown code fences
- MUST set .class_uid, .category_uid, .type_uid as integers
- Use `m, err = parse_regex(string!(.message), r'pattern')` safely

EXAMPLE for {device_type}:
{example}

Return ONLY the VRL code."""
        else:
            system_prompt = f"""Generate MINIMAL valid VRL for {device_type}.
{VRL_SYNTAX_RULES}

BARE MINIMUM TEMPLATE:
.class_uid = {self._get_class_uid(device_type)}
.category_uid = {self._get_category_uid(device_type)}
.type_uid = {self._get_type_uid(device_type)}
m_src, err = parse_regex(string!(.message), r'SRC=(?P<src>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {{
  .src_endpoint.ip = m_src.src
}}
m_dst, err = parse_regex(string!(.message), r'DST=(?P<dst>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {{
  .dst_endpoint.ip = m_dst.dst
}}
m_ip, err = parse_regex(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)')
if err == null {{
  if .src_endpoint.ip == null {{
    .src_endpoint.ip = m_ip.ip
  }}
  if .dst_endpoint.ip == null {{
    .dst_endpoint.ip = m_ip.ip
  }}
}}

Return ONLY the VRL code above adapted for the log."""
            
        prompt = f"Log sample: {template[:500]}\n\nGenerate VRL to parse this {device_type} log into OCSF."
        
        try:
            payload = {
                "model": self.model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": prompt}
                ],
                "temperature": 0.1,
                "max_tokens": 1024
            }
            
            logger.info(f"Coder: Requesting VRL generation from {self.host} (attempt {attempt + 1})...")
            resp = requests.post(
                f"{self.host}/v1/chat/completions",
                json=payload,
                timeout=self.timeout
            )
            
            if resp.status_code == 200:
                data = resp.json()
                content = data.get("choices", [{}])[0].get("message", {}).get("content", "")
                
                content = self._clean_vrl_output(content)
                
                logger.info(f"Coder: LLM generated VRL: \n{content}")
                return content
            else:
                logger.warning(f"Coder: LLM failed with status {resp.status_code}: {resp.text}")
                
        except requests.exceptions.RequestException as e:
            logger.warning(f"Coder: LLM unreachable ({e})")
            
        return None
    
    def _clean_vrl_output(self, content: str) -> str:
        """Clean LLM output to extract pure VRL code."""
        content = re.sub(r'```\w*\n?', '', content)
        content = re.sub(r'```', '', content)
        content = re.sub(r'^.*?(?=\.class_uid|\.message|\.ip|m_|#)', '', content, flags=re.DOTALL)
        return content.strip()
    
    def _validate_vrl(self, vrl: str, sample_log: str = "") -> bool:
        """Basic syntactic and dry-run validation of generated VRL."""
        if not vrl or len(vrl.strip()) < 20:
            return False
            
        required = ['.class_uid', '.category_uid', '.type_uid']
        for req in required:
            if req not in vrl:
                logger.warning(f"VRL validation failed: missing {req}")
                return False
                
        forbidden = [':=', '```', '```vrl', '```rust', '#!/usr', '.message = parse_syslog']
        for forbid in forbidden:
            if forbid in vrl:
                logger.warning(f"VRL validation failed: contains forbidden '{forbid}'")
                return False
                
        if not re.search(r'\.class_uid\s*=\s*\d+', vrl):
            logger.warning("VRL validation failed: class_uid not properly set")
            return False

        if sample_log:
            try:
                import subprocess, tempfile
                prism_bin = "/mnt/work/projects/sih/prism/target/release/prism"
                if os.path.exists(prism_bin):
                    with tempfile.NamedTemporaryFile('w', suffix='.vrl', delete=False) as tf:
                        tf.write(vrl)
                        tmp_name = tf.name
                    try:
                        res = subprocess.run(
                            [prism_bin, "--dry-run-vrl", tmp_name, "--payload", "-"],
                            input=sample_log,
                            capture_output=True,
                            text=True,
                            timeout=5
                        )
                        if res.returncode != 0:
                            logger.warning(f"VRL dry-run validation failed: {res.stderr.strip()}")
                            return False
                    finally:
                        if os.path.exists(tmp_name):
                            os.remove(tmp_name)
            except Exception as e:
                logger.warning(f"VRL dry-run validation error: {e}")
            
        return True
    
    def _get_class_uid(self, device_type: str) -> int:
        mapping = {
            "Network Activity": 4001,
            "Firewall": 4001,
            "Authentication": 3001,
            "Web Activity": 5001,
            "Web Proxy": 5001,
            "File Activity": 8001,
        }
        return mapping.get(device_type, 4001)
    
    def _get_category_uid(self, device_type: str) -> int:
        mapping = {
            "Network Activity": 4,
            "Firewall": 4,
            "Authentication": 3,
            "Web Activity": 5,
            "Web Proxy": 5,
            "File Activity": 8,
        }
        return mapping.get(device_type, 4)
    
    def _get_type_uid(self, device_type: str) -> int:
        mapping = {
            "Network Activity": 400101,
            "Firewall": 400101,
            "Authentication": 300101,
            "Web Activity": 500101,
            "Web Proxy": 500101,
            "File Activity": 800101,
        }
        return mapping.get(device_type, 400101)

    def _heuristic_fallback(self, device_type: str) -> str:
        class_uid = self._get_class_uid(device_type)
        category_uid = self._get_category_uid(device_type)
        type_uid = self._get_type_uid(device_type)
        
        lines = [
            f".class_uid = {class_uid}",
            f".category_uid = {category_uid}",
            f".type_uid = {type_uid}",
            "m_src, err = parse_regex(string!(.message), r'SRC=(?P<src>\\d+\\.\\d+\\.\\d+\\.\\d+)')",
            "if err == null {",
            "  .src_endpoint.ip = m_src.src",
            "}",
            "m_dst, err = parse_regex(string!(.message), r'DST=(?P<dst>\\d+\\.\\d+\\.\\d+\\.\\d+)')",
            "if err == null {",
            "  .dst_endpoint.ip = m_dst.dst",
            "}",
            "m_ip, err = parse_regex(string!(.message), r'(?P<ip>\\d+\\.\\d+\\.\\d+\\.\\d+)')",
            "if err == null {",
            "  .ip = m_ip.ip",
            "  if .src_endpoint.ip == null {",
            "    .src_endpoint.ip = m_ip.ip",
            "  }",
            "  if .dst_endpoint.ip == null {",
            "    .dst_endpoint.ip = m_ip.ip",
            "  }",
            "}",
        ]
        
        if device_type in ["Authentication", "Firewall"]:
            lines.extend([
                "m_user, err = parse_regex(string!(.message), r'user[=:]\"?(?P<user>[^\\s\"]+)')",
                "if err == null {",
                "  .user.name = m_user.user",
                "}"
            ])
        elif device_type in ["Web Activity", "Web Proxy"]:
            lines.extend([
                "m_url, err = parse_regex(string!(.message), r'(?P<url>https?://[^\\s]+)')",
                "if err == null {",
                "  .http.request.url = m_url.url",
                "}",
                "m_method, err = parse_regex(string!(.message), r'(?P<method>GET|POST|PUT|DELETE|HEAD)')",
                "if err == null {",
                "  .http.request.method = m_method.method",
                "}",
                "m_status, err = parse_regex(string!(.message), r'(?P<status>\\d{3})')",
                "if err == null {",
                "  .http.response.status_code = to_int!(m_status.status)",
                "}"
            ])
        elif device_type == "File Activity":
            lines.extend([
                "m_file, err = parse_regex(string!(.message), r'(?P<file>[\\w\\-. ]+\\.(?:exe|dll|pdf|docx?|xlsx?))')",
                "if err == null {",
                "  .file.name = m_file.file",
                "}",
                "m_path, err = parse_regex(string!(.message), r'(?P<path>[A-Za-z]:\\\\[^\\s]+|/[^\\s]+)')",
                "if err == null {",
                "  .file.path = m_path.path",
                "}"
            ])
            
        return "\n".join(lines)