import sys
import os
import shutil
import pytest
from pathlib import Path

# Setup paths so modules can be imported
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from cluster.cluster import LogClusterer
from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper

def run_pipeline(config):
    # 1. Feed alien NGINX log
    nginx_log = '192.168.1.100 - - [10/Oct/2026:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326'
    
    # 2. Drain3 clustering
    clusterer = LogClusterer()
    cluster_result = clusterer.process_log(nginx_log)
    assert "cluster_id" in cluster_result
    
    # 3. Open Jev classification
    triage = TriageEngine(config)
    device_type = triage.classify(cluster_result["template"])
    assert device_type == "Web Proxy"
    
    # 4. Ollama VRL generation
    coder = VrlCoder(config)
    vrl_code = coder.generate_vrl(cluster_result["template"], device_type)
    assert ".ip =" in vrl_code
    
    # 5. HitL Gatekeeper deployment
    gatekeeper = Gatekeeper("/tmp/prism_tests/rules")
    vrl_path = gatekeeper.approve_and_deploy(device_type, vrl_code, "nginx_signature")
    
    assert os.path.exists(vrl_path)
    assert os.path.exists(vrl_path.replace(".vrl", ".yaml"))
    
    # Cleanup
    shutil.rmtree("/tmp/prism_tests/rules")

def test_heuristic_cpu():
    config = {
        "device": "cpu",
        "triage": {"engine": "heuristic"},
        "coder": {"enabled": False, "host": "http://localhost:11434", "timeout": 0.5}
    }
    run_pipeline(config)

def test_gpu_skip():
    pytest.importorskip("torch")
    config = {
        "device": "gpu",
        "triage": {"engine": "open-jev"},
        "coder": {"enabled": True, "host": "http://localhost:11434", "timeout": 0.5}
    }
    run_pipeline(config)
