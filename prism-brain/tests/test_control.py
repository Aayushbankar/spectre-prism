import sys
import os
import shutil
import pytest
import time
import json
import tempfile
import threading
from pathlib import Path

# Setup paths so modules can be imported
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from cluster.cluster import LogClusterer
from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper
from watcher.watcher import start_watcher

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
    
    # Test Drain3 50k variance
    clusterer = LogClusterer()
    for i in range(50):
        cisco = f"%ASA-6-302013: Built inbound TCP connection {i} for outside:192.168.1.{i}/1234 (192.168.1.{i}/1234) to inside:10.0.0.1/80 (10.0.0.1/80)"
        fortinet = f'date=2024-01-01 time=12:00:{i:02d} devname="FW01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip=192.168.1.{i} dstip=8.8.8.8 action="accept"'
        clusterer.process_log(cisco)
        clusterer.process_log(fortinet)
    assert len(clusterer.miner.drain.clusters) <= 2
    
    # Test Watcher dlq.jsonl rotation
    results = []
    def callback(data):
        results.append(data)
        
    with tempfile.TemporaryDirectory() as tmpdir:
        path = os.path.join(tmpdir, "dlq.jsonl")
        with open(path, "w") as f:
            f.write("")
        observer = start_watcher(path, callback)
        
        with open(path, "a") as f:
            f.write(json.dumps({"test": 1}) + "\n")
            f.flush()
        time.sleep(0.5)
        
        with open(path, "a") as f:
            f.write(json.dumps({"test": 2}) + "\n")
            f.flush()
        time.sleep(0.5)
        
        observer.stop()
        observer.join()
        
        assert len(results) >= 2
        assert any(r.get("test") == 1 for r in results)
        assert any(r.get("test") == 2 for r in results)

def test_gpu_skip():
    pytest.importorskip("torch")
    config = {
        "device": "gpu",
        "triage": {"engine": "open-jev"},
        "coder": {"enabled": True, "host": "http://localhost:11434", "timeout": 0.5}
    }
    run_pipeline(config)
