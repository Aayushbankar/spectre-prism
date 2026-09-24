import time
import json
import os
from cluster.cluster import LogClusterer

def test_drain_fortinet_isolated():
    c = LogClusterer()
    t0 = time.time()
    for i in range(5000):
        c.process_log(f'date=2024-01-01 time=12:00:{i%60:02d} devname="FW01" logid="0000000013" srcip=192.168.1.{i} dstip=8.8.8.8 action="accept"')
    assert len(c.miner.drain.clusters) == 1
    assert time.time() - t0 < 1.0

def test_drain_cisco_isolated():
    c = LogClusterer()
    t0 = time.time()
    for i in range(5000):
        c.process_log(f'%ASA-6-302013: Built inbound TCP connection {i} for outside:192.168.1.{i}/1234 (192.168.1.{i}/1234) to inside:10.0.0.1/80 (10.0.0.1/80)')
    assert len(c.miner.drain.clusters) == 1
    assert time.time() - t0 < 1.0

def test_drain_mixed_10k():
    c = LogClusterer()
    t0 = time.time()
    for i in range(5000):
        c.process_log(f'date=2024-01-01 time=12:00:{i%60:02d} devname="FW01" logid="0000000013" srcip=192.168.1.{i} dstip=8.8.8.8 action="accept"')
        c.process_log(f'%ASA-6-302013: Built inbound TCP connection {i} for outside:192.168.1.{i}/1234 (192.168.1.{i}/1234) to inside:10.0.0.1/80 (10.0.0.1/80)')
    assert len(c.miner.drain.clusters) <= 2
    assert time.time() - t0 < 1.0

def test_drain_50k_bigdata():
    c = LogClusterer()
    t0 = time.time()
    data_path = os.path.join(os.path.dirname(__file__), "..", "data", "bigdata_52k.jsonl")
    if os.path.exists(data_path):
        with open(data_path, 'r') as f:
            for line in f:
                c.process_log(line.strip())
    assert len(c.miner.drain.clusters) <= 10
    assert time.time() - t0 < 2.0
