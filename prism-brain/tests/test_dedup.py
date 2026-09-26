import sys
import os

# Ensure prism-brain modules can be imported
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from main import normalize_log_signature
from cluster.cluster import LogClusterer

def test_log_signature_deduplication():
    # 1. Two Fortinet logs with different IPs
    fortinet_log1 = "Sep 25 2023 10:15:30 192.168.1.100 CEF:0|Fortinet|Fortigate|7.0|00001|Traffic|3|src=10.0.0.5 dst=8.8.8.8"
    fortinet_log2 = "Sep 25 2023 10:15:35 192.168.1.101 CEF:0|Fortinet|Fortigate|7.0|00001|Traffic|3|src=10.0.0.6 dst=1.1.1.1"
    
    sig1 = normalize_log_signature(fortinet_log1)
    sig2 = normalize_log_signature(fortinet_log2)
    
    assert sig1 == sig2
    
    # 2. A Fortinet log and a Cisco log
    cisco_log = "Sep 25 2023 10:15:30 192.168.1.200 %ASA-6-302013: Built inbound TCP connection 1234567 for outside:1.2.3.4/80 (1.2.3.4/80) to inside:10.0.0.5/1234 (10.0.0.5/1234)"
    sig3 = normalize_log_signature(cisco_log)
    
    assert sig1 != sig3
    
    # 3. Two identical logs
    sig4 = normalize_log_signature(cisco_log)
    assert sig3 == sig4

def test_drain3_clustering():
    clusterer = LogClusterer()
    
    log1 = "Connection established to 192.168.1.5"
    log2 = "Connection established to 10.0.0.12"
    
    res1 = clusterer.process_log(log1)
    assert res1['size'] == 1
    
    res2 = clusterer.process_log(log2)
    assert res2['size'] == 2
