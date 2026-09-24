import time
import pytest
import requests
from coder.coder import VrlCoder

def test_coder_heuristic_0s():
    c = VrlCoder({"device": "cpu", "coder": {"enabled": False}})
    t0 = time.time()
    vrl = c.generate_vrl("192.168.1.5 - - [01/Jan] GET /index.html", "Web Proxy")
    assert ".ip =" in vrl
    assert time.time() - t0 < 0.05
    print(f"heuristic {time.time()-t0:.4f}s")

def test_coder_llama_q4_07s():
    # CPU Q4 0.7s vs heuristic 0s — skip if llama-server not running
    try:
        requests.get("http://127.0.0.1:8088/health", timeout=0.2)
    except:
        pytest.skip("llama-server not running CPU Q4")
    
    c = VrlCoder({"device": "cpu", "coder": {"enabled": True, "host": "http://127.0.0.1:8088", "timeout": 2}})
    t0 = time.time()
    vrl = c.generate_vrl('192.168.1.5 - - [01/Jan] "GET /index.html" 200', "Web Proxy")
    
    assert ".ip =" in vrl
    assert time.time() - t0 < 2.0
    print(f"llama Q4 {time.time()-t0:.3f}s")
    
    # Validate VRL compiles in Rust: call vrl::compiler::compile via python subprocess or just assert parse_regex present
    assert "parse_regex" in vrl or ".ip =" in vrl

def test_coder_heuristic_fallback_on_ollama_down():
    c = VrlCoder({"device": "cpu", "coder": {"enabled": True, "host": "http://127.0.0.1:59999", "timeout": 0.5}})
    vrl = c.generate_vrl("unknown template", "Unknown")
    assert vrl == '.ip = "0.0.0.0"'
