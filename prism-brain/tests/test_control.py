import sys
import os
import shutil
import pytest
import time
import json
import threading
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from cluster.cluster import LogClusterer
from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper
from watcher.watcher import start_watcher
from config import load_config

def run_pipeline(config):
    nginx_log = '192.168.1.100 - - [10/Oct/2026:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326'
    
    clusterer = LogClusterer()
    cluster_result = clusterer.process_log(nginx_log)
    assert "cluster_id" in cluster_result
    
    triage = TriageEngine(config)
    device_type = triage.classify(cluster_result["template"])
    assert device_type == "Web Proxy"
    
    coder = VrlCoder(config)
    vrl_code = coder.generate_vrl(cluster_result["template"], device_type)
    assert ".ip =" in vrl_code
    
    # Use default Gatekeeper (which falls back to /tmp/prism/rules when /etc is denied)
    gatekeeper = Gatekeeper()
    vrl_path = gatekeeper.approve_and_deploy(device_type, vrl_code, "nginx_signature")
    
    assert os.path.exists(vrl_path)
    assert os.path.exists(vrl_path.replace(".vrl", ".yaml"))
    
    # Clean up the fallback rules dir
    if gatekeeper.rules_dir.startswith("/tmp/"):
        shutil.rmtree(gatekeeper.rules_dir, ignore_errors=True)

def test_heuristic_cpu():
    config = {
        "device": "cpu",
        "triage": {"engine": "heuristic"},
        "coder": {"enabled": False, "host": "http://localhost:11434", "timeout": 0.5},
        "watcher": {"path": "/var/run/prism/dlq.jsonl", "fallback": "/tmp/prism_dlq.log"}
    }
    run_pipeline(config)
    
    # Test Drain3 10k variance
    clusterer_fortinet = LogClusterer()
    clusterer_cisco = LogClusterer()
    clusterer_mixed = LogClusterer()
    
    start_time = time.time()
    for i in range(5000):
        cisco = f"%ASA-6-302013: Built inbound TCP connection {i} for outside:192.168.1.{i}/1234 (192.168.1.{i}/1234) to inside:10.0.0.1/80 (10.0.0.1/80)"
        fortinet = f'date=2024-01-01 time=12:00:{i:02d} devname="FW01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip=192.168.1.{i} dstip=8.8.8.8 action="accept"'
        
        clusterer_fortinet.process_log(fortinet)
        clusterer_cisco.process_log(cisco)
        clusterer_mixed.process_log(fortinet)
        clusterer_mixed.process_log(cisco)
        
    duration = time.time() - start_time
    assert duration < 1.0, f"Duration was {duration}s"
    
    assert len(clusterer_fortinet.miner.drain.clusters) == 1
    assert len(clusterer_cisco.miner.drain.clusters) == 1
    assert len(clusterer_mixed.miner.drain.clusters) <= 2
    
    # Test Watcher DUAL Path
    results = []
    def callback(data):
        results.append(data)
        
    watcher_conf = config["watcher"]
    primary_path = watcher_conf["path"]
    fallback_path = watcher_conf["fallback"]
    
    # Decide which path to use (simulating permission denied on /var/run)
    try:
        os.makedirs(os.path.dirname(primary_path), exist_ok=True)
        test_path = primary_path
    except PermissionError:
        test_path = fallback_path
        os.makedirs(os.path.dirname(test_path), exist_ok=True)

    with open(test_path, "w") as f:
        f.write("")
        
    observer = start_watcher(test_path, callback)
    
    with open(test_path, "a") as f:
        f.write(json.dumps({"test": 1}) + "\n")
        f.flush()
    time.sleep(0.5)
    
    with open(test_path, "a") as f:
        f.write(json.dumps({"test": 2}) + "\n")
        f.flush()
    time.sleep(0.5)
    
    observer.stop()
    observer.join()
    
    assert len(results) >= 2
    assert any(r.get("test") == 1 for r in results)
    assert any(r.get("test") == 2 for r in results)
    
    if test_path.startswith("/tmp/"):
        os.remove(test_path)

def test_gpu_skip():
    try:
        import torch
    except Exception:
        pytest.skip("Torch not fully installed")
    config = {
        "device": "gpu",
        "triage": {"engine": "open-jev"},
        "coder": {"enabled": True, "host": "http://localhost:11434", "timeout": 0.5}
    }
    run_pipeline(config)
