import json
import random

def generate():
    with open("prism-brain/data/bigdata_50k.jsonl", "w") as f:
        # 15k Fortinet
        for i in range(15000):
            ip = f"192.168.1.{i%255}"
            log = f'date=2024-01-01 time=12:00:00 devname="FW01" devid="FG100" logid="0000000013" type="traffic" subtype="forward" level="notice" srcip={ip} dstip=8.8.8.8 action="accept"'
            f.write(json.dumps({"raw_payload": log}) + "\n")
        
        # 15k Cisco
        for i in range(15000):
            ip = f"10.0.0.{i%255}"
            log = f'%ASA-6-302013: Built inbound TCP connection 1234 for outside:{ip}/1234 ({ip}/1234) to inside:10.0.0.1/80 (10.0.0.1/80)'
            f.write(json.dumps({"raw_payload": log}) + "\n")
            
        # 10k Palo Alto
        for i in range(10000):
            ip = f"172.16.0.{i%255}"
            log = f'1,2024/01/01 12:00:00,0012345,THREAT,vulnerability,1,2024/01/01 12:00:00,{ip},8.8.8.8,0.0.0.0,0.0.0.0,rule1,vsys1,trust,untrust'
            f.write(json.dumps({"raw_payload": log}) + "\n")
            
        # 5k NGINX
        for i in range(5000):
            ip = f"192.168.2.{i%255}"
            log = f'{ip} - - [10/Oct/2026:13:55:36 -0700] "GET /index.html HTTP/1.1" 200 2326'
            f.write(json.dumps({"raw_payload": log}) + "\n")
            
        # 5k CloudTrail (JSON inside JSON)
        for i in range(5000):
            ip = f"192.168.3.{i%255}"
            log = json.dumps({"eventVersion": "1.08", "userIdentity": {"type": "IAMUser"}, "eventTime": "2024-01-01T12:00:00Z", "sourceIPAddress": ip})
            f.write(json.dumps({"raw_payload": log}) + "\n")

if __name__ == "__main__":
    generate()
