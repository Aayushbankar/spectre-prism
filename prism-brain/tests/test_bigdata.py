import sys
import os
import time
import json
import pytest
import subprocess
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from cluster.cluster import LogClusterer
from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper

def test_drain3_throughput():
    clusterer = LogClusterer()
    dataset_path = "prism-brain/data/bigdata_50k.jsonl"
    
    start_time = time.time()
    count = 0
    with open(dataset_path, "r") as f:
        for line in f:
            data = json.loads(line)
            clusterer.process_log(data["raw_payload"])
            count += 1
            
    duration = time.time() - start_time
    assert count == 52000
    assert duration < 5.0
    assert len(clusterer.miner.drain.clusters) < 300

def test_triage_engines():
    config = {"device": "cpu", "triage": {"engine": "heuristic"}}
    triage = TriageEngine(config)
    
    templates = [
        'date=2024-01-01 time=12:00:00 devname="FW01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip=<*> dstip=8.8.8.8 action="accept"',
        '%ASA-6-302013: Built inbound TCP connection 1234 for outside:<*>/1234 (<*>/1234) to inside:10.0.0.1/80 (10.0.0.1/80)',
        '<*> - - [10/Oct/2026:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326'
    ]
    
    start_time = time.time()
    for t in templates:
        triage.classify(t)
    duration = time.time() - start_time
    # Simulate Laya test for documentation
    # Since Laya relies on Torch which is missing, we measure Heuristic natively
    print(f"Triage Heuristic took {duration:.6f}s for {len(templates)} templates")

def test_e2e_throughput():
    clusterer = LogClusterer()
    config = {"device": "cpu", "triage": {"engine": "heuristic"}, "coder": {"enabled": False}}
    triage = TriageEngine(config)
    coder = VrlCoder(config)
    gatekeeper = Gatekeeper("/tmp/prism/rules")
    
    dataset_path = "prism-brain/data/bigdata_50k.jsonl"
    start_time = time.time()
    
    count = 0
    with open(dataset_path, "r") as f:
        for line in f:
            if count >= 1000:
                break
            data = json.loads(line)
            raw = data["raw_payload"]
            cluster_result = clusterer.process_log(raw)
            if cluster_result.get("change_type") != "none":
                device_type = triage.classify(cluster_result["template"])
                vrl = coder.generate_vrl(cluster_result["template"], device_type)
                gatekeeper.approve_and_deploy(device_type, vrl, "auto_signature")
            count += 1
            
    duration = time.time() - start_time
    eps = count / duration
    assert eps > 1000

def test_llama_coder():
    config = {"device": "cpu", "coder": {"enabled": True, "host": "http://127.0.0.1:8088", "timeout": 2.0}}
    coder = VrlCoder(config)
    template = '<*> - - [10/Oct/2026:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326'
    start_time = time.time()
    vrl = coder.generate_vrl(template, "Web Proxy")
    duration = time.time() - start_time
    assert ".ip =" in (vrl or "")
    print(f"Llama coder took {duration:.4f}s")
