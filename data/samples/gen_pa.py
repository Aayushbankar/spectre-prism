import random
from datetime import datetime, timedelta

with open('/mnt/work/projects/sih/prism/data/samples/paloalto_threat.log', 'w') as f:
    base_time = datetime(2026, 9, 26, 10, 0, 0)
    for i in range(20):
        # 1,2021/08/24 11:51:12,010108010441,TRAFFIC,start,2305,2021/08/24 11:51:12,10.0.70.54,192.168.6.215,0.0.0.0,0.0.0.0,Rule-Web,user1,,ssl,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2021/08/24 11:51:12,654123,1,54210,443,0,0,0x0,tcp,allow,1420,700,720,12,6,6,0
        t = base_time + timedelta(seconds=i)
        ts = t.strftime("%Y/%m/%d %H:%M:%S")
        
        parts = []
        parts.append('1') # 0: FUTURE_USE
        parts.append(ts) # 1: Receive Time
        parts.append('010108010441') # 2: Serial Number
        parts.append('TRAFFIC') # 3: Type
        parts.append('start') # 4: Subtype
        parts.append('2305') # 5: FUTURE_USE
        parts.append(ts) # 6: Generated Time
        parts.append(f'10.10.20.{45 + i}') # 7: Source IP
        parts.append(f'203.0.113.{25 + i}') # 8: Destination IP
        parts.append('0.0.0.0') # 9: NAT Source IP
        parts.append('0.0.0.0') # 10: NAT Destination IP
        parts.append('Rule-Web') # 11: Rule Name
        parts.append('user1') # 12: Source User
        parts.append('') # 13: Destination User
        parts.append('ssl') # 14: Application
        parts.append('vsys1') # 15: Virtual System
        parts.append('Trust') # 16: Source Zone
        parts.append('Untrust') # 17: Destination Zone
        parts.append('ethernet1/2') # 18: Inbound Interface
        parts.append('ethernet1/1') # 19: Outbound Interface
        parts.append('LogForwarding_Syslog') # 20: Log Action
        parts.append(ts) # 21: FUTURE_USE
        parts.append('654123') # 22: Session ID
        parts.append('1') # 23: Repeat Count
        parts.append(str(52341 + i)) # 24: Source Port
        parts.append('443') # 25: Destination Port
        parts.append('0') # 26: NAT Source Port
        parts.append('0') # 27: NAT Destination Port
        parts.append('0x0') # 28: Flags
        parts.append('tcp') # 29: Protocol
        parts.append('allow') # 30: Action
        parts.append('1420') # 31: Bytes
        parts.append('700') # 32: Bytes Sent
        parts.append('720') # 33: Bytes Received
        parts.append('12') # 34: Packets
        parts.append('6') # 35: Start Time
        parts.append('6') # 36: Elapsed Time
        parts.append('0') # 37: Category
        
        line = ','.join(parts) + '\n'
        f.write(line)
