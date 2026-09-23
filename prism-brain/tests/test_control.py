import sys
import os
import shutil
import pytest

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from cluster.cluster import LogClusterer
from triage.triage import TriageEngine
from coder.coder import VrlCoder
from hitl.gatekeeper import Gatekeeper

@pytest.mark.parametrize("device_mode, coder_enabled", [
    ("cpu", False),
    ("gpu", True)
])
def test_control_plane_end_to_end(device_mode, coder_enabled):
    config = {
        "device": device_mode,
        "triage": {"engine": "open-jev" if device_mode == "gpu" else "heuristic"},
        "coder": {"enabled": coder_enabled, "host": "http://localhost:11434", "timeout": 0.5}
    }
    
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
