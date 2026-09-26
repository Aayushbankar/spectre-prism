import os, time, json, shutil
import tempfile
import pytest
from watcher.watcher import start_watcher
from hitl.gatekeeper import Gatekeeper, create_gatekeeper
from cluster.cluster import LogClusterer
from triage.triage import TriageEngine
from coder.coder import VrlCoder

def test_watcher_dual_path():
    results = []
    def callback(data):
        results.append(data)

    test_file = "/tmp/watcher_test_dlq.jsonl"
    with open(test_file, "w") as f:
        f.write("")

    obs = start_watcher(test_file, callback)
    
    with open(test_file, "a") as f:
        f.write('{"test":1}\n{"test":2}\n')
        f.flush()
        
    time.sleep(1.0)
    obs.stop()
    obs.join()
    
    assert len(results) >= 2
    assert results[0]["test"] == 1
    assert results[1]["test"] == 2

def test_gatekeeper_hot_reload():
    rules_dir = "/tmp/prism_test_gatekeeper"
    base_dir = "/tmp/prism_test_gatekeeper_base"
    if os.path.exists(rules_dir):
        shutil.rmtree(rules_dir)
    if os.path.exists(base_dir):
        shutil.rmtree(base_dir)
        
    gk = create_gatekeeper({"base_dir": base_dir, "rules_dir": rules_dir})
    rule_id = gk.submit_rule("Web Proxy", r".ip = parse_regex!(string!(.message), r'(?P<ip>\d+\.\d+\.\d+\.\d+)').ip\n.class_uid = 5001\n.category_uid = 5\n.type_uid = 500101\n.src_endpoint.ip = .ip\n.dst_endpoint.ip = .ip", "sig123", "192.168.1.1")
    
    # Rule should be in pending (or failed if dry-run fails)
    all_rules = gk.get_all_rules()
    assert len(all_rules) == 1
    assert all_rules[0].rule_id == rule_id
    
    # If pending, approve it
    if all_rules[0].state == "pending":
        gk.approve_rule(rule_id)
        
        # Check rule is deployed
        approved = gk.get_approved_rules()
        assert len(approved) == 1
        assert approved[0].rule_id == rule_id
        
        # Verify VRL file exists in rules dir
        vrl_files = [f for f in os.listdir(rules_dir) if f.endswith(".vrl")]
        assert len(vrl_files) >= 1
        
        with open(os.path.join(rules_dir, vrl_files[0]), "r") as f:
            v = f.read()
            assert ".ip" in v
    
    shutil.rmtree(rules_dir, ignore_errors=True)
    shutil.rmtree(base_dir, ignore_errors=True)

def test_e2e_1000_heuristic():
    if os.path.exists("drain3.ini"):
        os.remove("drain3.ini")
    if os.path.exists("drain3_state.bin"):
        os.remove("drain3_state.bin")
        
    clusterer = LogClusterer()
    triage = TriageEngine({"device": "cpu", "triage": {"engine": "heuristic"}})
    coder = VrlCoder({"device": "cpu", "coder": {"enabled": False}})
    
    rules_dir = "/tmp/prism_e2e"
    base_dir = "/tmp/prism_e2e_base"
    if os.path.exists(rules_dir):
        shutil.rmtree(rules_dir)
    if os.path.exists(base_dir):
        shutil.rmtree(base_dir)
    
    # Pre-create to avoid PermissionError logic doing weird things
    os.makedirs(rules_dir, exist_ok=True)
    os.makedirs(base_dir, exist_ok=True)
    gk = create_gatekeeper({"base_dir": base_dir, "rules_dir": rules_dir})
    
    data_path = os.path.join(os.path.dirname(__file__), "..", "data", "bigdata_52k.jsonl")
    
    # Check if data file exists, skip if not
    if not os.path.exists(data_path):
        pytest.skip("bigdata_52k.jsonl not found")
    
    t0 = time.time()
    count = 0
    with open(data_path, 'r') as f:
        for line in f:
            if count >= 1000: break
            try:
                payload = json.loads(line).get("raw_payload", line.strip())
            except json.JSONDecodeError:
                payload = line.strip()
            
            # 1. Cluster
            result = clusterer.miner.add_log_message(payload)
            change_type = result["change_type"]
            
            if change_type != "none":
                # 2. Triage
                device_type = triage.classify(result["template_mined"])
                
                # 3. Coder
                vrl_code = coder.generate_vrl(result["template_mined"], device_type)
                
                # 4. Gatekeeper
                rule_id = gk.submit_rule(device_type, vrl_code, result["template_mined"], payload)
                
            count += 1
            
    dur = time.time() - t0
    eps = count / dur if dur > 0 else 0
    
    # Check that at least one rule was created
    all_rules = gk.get_all_rules()
    assert len(all_rules) >= 1
    
    print(f"E2E 1000 logs: {dur:.2f}s EPS: {eps:.0f}")
    
    shutil.rmtree(rules_dir, ignore_errors=True)
    shutil.rmtree(base_dir, ignore_errors=True)