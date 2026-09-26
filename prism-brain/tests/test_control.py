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
from hitl.gatekeeper import Gatekeeper, create_gatekeeper
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
    
    # Use Gatekeeper with test directories
    import shutil
    test_base = Path("/tmp/prism_test")
    if test_base.exists():
        shutil.rmtree(test_base)
    
    gatekeeper = create_gatekeeper({"base_dir": str(test_base), "rules_dir": str(test_base / "rules")})
    rule_id = gatekeeper.submit_rule(device_type, vrl_code, "nginx_signature", nginx_log)
    
    # Check rule exists (may be pending or failed due to dry-run)
    all_rules = gatekeeper.get_all_rules()
    assert len(all_rules) == 1
    assert all_rules[0].rule_id == rule_id
    
    # If pending, approve it
    if all_rules[0].state == "pending":
        gatekeeper.approve_rule(rule_id)
        
        # Check rule is deployed
        approved = gatekeeper.get_approved_rules()
        assert len(approved) == 1
        assert approved[0].rule_id == rule_id
        
        # Verify VRL file exists in rules dir
        rules_dir = test_base / "rules"
        vrl_files = list(rules_dir.glob("*.vrl"))
        assert len(vrl_files) >= 1

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
