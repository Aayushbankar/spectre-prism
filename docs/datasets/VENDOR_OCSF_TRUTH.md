# Vendor Syslog Samples → OCSF Ground Truth (SIH26156 / PRISM)

**Branch:** `research/dataset-v1` · **Worktree:** `/mnt/work/projects/sih-dataset-research`
**OCSF version:** v1.9.0 · **Output contract:** `docs/architecture/DATA_DICTIONARY_AND_IPC.md`
**Companions:** `docs/datasets/DATASET_PLAN.md`, `docs/references/PERIMETER_DEVICES.md`,
`docs/references/OCSF_SCHEMA.md`, `docs/references/LOG_FORMAT_STANDARDS.md`,
`datasets/fixtures/{golden_mixed.log,ocsf_expected.jsonl,README.md}`

> **SYNTHETIC-BUT-VALID NOTICE (read first).** Every raw log line below is
> **synthetic test data** — hand-built to be **syntactically valid** against the
> vendor's documented pattern (field order, delimiters, message IDs, envelope).
> IPs use documentation ranges (RFC 5737 `192.0.2.0/24`, `198.51.100.0/24`,
> `203.0.113.0/24`) plus RFC 1918 inside addresses. Hostnames/serials (`FG-500E`,
> `FG500E4Q17000123`, `001801000123`) are fictitious. No line is copied from a
> real customer deployment. Patterns derived from:
> Fortinet FortiOS Log Message Reference, Cisco ASA Syslog Messages Guide,
> PAN-OS Syslog Field Descriptions, Suricata EVE JSON schema, NGINX `combined`
> format, ArcSight CEF Standard, IBM LEEF Guide, Check Point Log Exporter
> (sk122323). See `docs/references/PERIMETER_DEVICES.md` and
> `docs/references/LOG_FORMAT_STANDARDS.md`.

## Conventions used everywhere

| Convention | Value |
| :--- | :--- |
| Base time | `2024-09-22T14:30:01Z` = epoch-ms `1727015401000`; variants add +N s (`+1000` ms per second) |
| `category_uid` / `category_name` | `4` / `Network` (IAM `3` only for §10 VPN auth → `3002`) |
| `class_uid` | `4001` network_activity · `4002` http_activity · `4003` dns_activity · `3002` authentication |
| `type_uid` | `class_uid*100 + activity_id` (e.g. `400102` = Network Activity: Close) |
| `activity_id` (4001) | `1` Open · `2` Close · `3` Reset · `4` Fail · `5` Refuse · `6` Traffic |
| `activity_id` (4002) | `1` Request |
| `activity_id` (4003) | `1` Query |
| `activity_id` (3002) | `1` Logon |
| `severity_id` / `severity` | `1` Informational · `2` Low · `3` Medium · `4` High · `5` Critical (per `docs/references/OCSF_SCHEMA.md` §3) |
| `connection_info.protocol_num` | IANA: `6` TCP · `17` UDP · `1` ICMP |
| `connection_info.direction_id` | `1` Inbound · `2` Outbound · `3` Lateral (inside→inside) |
| `metadata.version` | `"1.9.0"` |
| PRISM extensions | `metadata.history_hash` (`"BLAKE3:<hex>"`, computed at ingest — placeholder below) · `metadata.vault_uri` (`"parquet:<block>#<offset>"`, placeholder) |
| `observables` | always populated with src/dst IPs (+ domain/URL where present), per DATA_DICTIONARY (required) |
| `action_id` / `disposition_id` | `1` Allowed / `1` Allowed · `2` Denied / `2` Blocked · `6` Dropped · `21` Reset (security_control profile) |
| Fixture cross-ref | `F-<n>` = line n of `datasets/fixtures/golden_mixed.log`; `E-<n>` = line n of `ocsf_expected.jsonl` |

---

## 1. Fortinet FortiGate — KV over RFC 5424

**Envelope:** RFC 5424 (`PRI VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID [SD] MSG`).
FortiGate emits the KV payload as MSG; PRI `<134>` (local0/notice) typical.
Parser targets: `srcip srcport dstip dstport proto action sentbyte rcvdbyte duration transip polid`.

### Sample F-1 — forward close, allow (SYNTHETIC-BUT-VALID, F-1)

```text
<134>1 2024-09-22T14:30:01Z FG-500E fortigate 1234 - date=2024-09-22 time=14:30:01 devname="FG-500E" devid="FG500E4Q17000123" logid="0000000013" type="traffic" subtype="forward" level="notice" vd="root" eventtime=1727015401 srcip=10.1.100.11 srcport=54190 srcintf="port1" dstip=198.51.100.45 dstport=443 dstintf="port2" sessionid=402 proto=6 action="close" policyid=4 policytype="policy" service="HTTPS" duration=15 sentbyte=4520 rcvdbyte=8920 sentpkt=58 rcvdpkt=105 app="HTTPS.BROWSER" utmaction="allow"
```

### Sample F-2 — forward deny (SYNTHETIC-BUT-VALID, F-2)

```text
<134>1 2024-09-22T14:30:05Z FG-500E fortigate 1234 - date=2024-09-22 time=14:30:05 devname="FG-500E" devid="FG500E4Q17000123" logid="0000000002" type="traffic" subtype="forward" level="warning" vd="root" eventtime=1727015405 srcip=192.0.2.7 srcport=63312 srcintf="port1" dstip=203.0.113.99 dstport=22 dstintf="port2" sessionid=0 proto=6 action="deny" policyid=0 policytype="policy" service="SSH" duration=0 sentbyte=0 rcvdbyte=0 sentpkt=0 rcvdpkt=0 utmaction="block"
```

### Sample F-3 — UTM/IPS block (SYNTHETIC-BUT-VALID, F-3)

```text
<134>1 2024-09-22T14:30:12Z FG-500E fortigate 1234 - date=2024-09-22 time=14:30:12 devname="FG-500E" devid="FG500E4Q17000123" logid="0419016384" type="utm" subtype="ips" level="warning" vd="root" eventtime=1727015412 srcip=10.1.100.23 srcport=51234 srcintf="port1" dstip=198.51.100.80 dstport=80 dstintf="port2" sessionid=915 proto=6 action="dropped" policyid=4 attack="HTTP.URI.SQL.Injection" severity="high" ref="https://fortiguard.com/encyclopedia/ips/12345" msg="SQL injection blocked" sentbyte=1200 rcvdbyte=0
```

### Field table (Fortinet)

| KV key | Meaning | OCSF target |
| :--- | :--- | :--- |
| `srcip` / `srcport` / `srcintf` | pre-NAT source | `src_endpoint.ip` / `.port` / `.interface_name` |
| `dstip` / `dstport` / `dstintf` | destination | `dst_endpoint.ip` / `.port` / `.interface_name` |
| `proto` | IANA number | `connection_info.protocol_num` (+ `protocol_name`) |
| `action` | `close`→Close, `deny`/`dropped`→Deny | `activity_id`, `action_id`, `disposition_id` |
| `sentbyte` / `rcvdbyte` / `sentpkt` / `rcvdpkt` | counters | `traffic.bytes_out` / `.bytes_in` / `.packets_out` / `.packets_in` (+ totals) |
| `duration` | seconds | `connection_info` context / `duration_time` where used |
| `policyid` | firewall rule id | `firewall_rule.uid` (+ `.name` if known) |
| `trandisp` / `transip` | NAT disposition/IP | `src_endpoint.nat_ip` (translator) — record if present |
| `attack` / `msg` / `ref` | UTM detail | `finding_info` / `observables` context (kept, class stays 4001 per this fixture) |
| `devname` / `devid` | appliance identity | `metadata.product` + `device` hostname/uid |

### Expected OCSF — F-1 (4001 Close, allow)

```json
{
  "category_uid": 4,
  "category_name": "Network Activity",
  "class_uid": 4001,
  "class_name": "Network Activity",
  "activity_id": 2,
  "activity_name": "Close",
  "type_uid": 400102,
  "type_name": "Network Activity: Close",
  "time": 1727015401000,
  "severity_id": 1,
  "severity": "Informational",
  "action_id": 1,
  "action": "Allowed",
  "disposition_id": 1,
  "disposition": "Allowed",
  "src_endpoint": {"ip": "10.1.100.11", "port": 54190, "interface_name": "port1"},
  "dst_endpoint": {"ip": "198.51.100.45", "port": 443, "interface_name": "port2"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 8920, "bytes_out": 4520, "bytes": 13440, "packets_in": 105, "packets_out": 58, "packets": 163},
  "firewall_rule": {"uid": "4", "name": "policy-4"},
  "observables": [{"name": "10.1.100.11", "type": "IP Address"}, {"name": "198.51.100.45", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Fortinet", "name": "FortiGate", "version": "FortiOS 7.4"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — F-2 (4001 Refuse, deny)

```json
{
  "category_uid": 4,
  "category_name": "Network Activity",
  "class_uid": 4001,
  "class_name": "Network Activity",
  "activity_id": 5,
  "activity_name": "Refuse",
  "type_uid": 400105,
  "type_name": "Network Activity: Refuse",
  "time": 1727015405000,
  "severity_id": 3,
  "severity": "Medium",
  "action_id": 2,
  "action": "Denied",
  "disposition_id": 2,
  "disposition": "Blocked",
  "src_endpoint": {"ip": "192.0.2.7", "port": 63312, "interface_name": "port1"},
  "dst_endpoint": {"ip": "203.0.113.99", "port": 22, "interface_name": "port2"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "0", "name": "implicit-deny"},
  "observables": [{"name": "192.0.2.7", "type": "IP Address"}, {"name": "203.0.113.99", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Fortinet", "name": "FortiGate", "version": "FortiOS 7.4"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — F-3 (4001 Refuse, IPS dropped)

```json
{
  "category_uid": 4,
  "category_name": "Network Activity",
  "class_uid": 4001,
  "class_name": "Network Activity",
  "activity_id": 5,
  "activity_name": "Refuse",
  "type_uid": 400105,
  "type_name": "Network Activity: Refuse",
  "time": 1727015412000,
  "severity_id": 4,
  "severity": "High",
  "action_id": 2,
  "action": "Denied",
  "disposition_id": 6,
  "disposition": "Dropped",
  "src_endpoint": {"ip": "10.1.100.23", "port": 51234, "interface_name": "port1"},
  "dst_endpoint": {"ip": "198.51.100.80", "port": 80, "interface_name": "port2"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 1200, "bytes": 1200, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "4", "name": "policy-4"},
  "observables": [{"name": "10.1.100.23", "type": "IP Address"}, {"name": "198.51.100.80", "type": "IP Address"}, {"name": "HTTP.URI.SQL.Injection", "type": "Signature"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Fortinet", "name": "FortiGate-IPS", "version": "FortiOS 7.4"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 2. Cisco ASA — structured text over RFC 3164

**Envelope:** RFC 3164 (`<PRI>TIMESTAMP HOSTNAME MSG`; TIMESTAMP `Mmm dd hh:mm:ss`,
no year/timezone — year 2024 assumed by parser). PRI `<166>` (local4/info) typical.
Parser targets: `%ASA-sev-ID`, conn number, `for <ifc>:IP/port to <ifc>:IP/port`,
`duration`, `bytes`, `reason`.

### Sample C-1 — %ASA-6-302013 build inbound (SYNTHETIC-BUT-VALID, F-4)

```text
<166>Sep 22 14:30:20 asa-5525 %ASA-6-302013: Built inbound TCP connection 123456 for outside:203.0.113.15/49152 (203.0.113.15/49152) to inside:10.1.1.25/80 (10.1.1.25/80)
```

### Sample C-2 — %ASA-6-302014 teardown (SYNTHETIC-BUT-VALID, F-5)

```text
<166>Sep 22 14:30:50 asa-5525 %ASA-6-302014: Teardown TCP connection 123456 for outside:203.0.113.15/49152 to inside:10.1.1.25/80 duration 0:00:30 bytes 14502 TCP FINs
```

### Sample C-3 — %ASA-4-106100 ACL deny (SYNTHETIC-BUT-VALID, F-6)

```text
<166>Sep 22 14:31:05 asa-5525 %ASA-4-106100: access-list outside_in denied tcp outside/203.0.113.7(52344) -> inside/192.168.1.10(80) hit-cnt 1 first hit [0x1234abcd]
```

### Sample C-4 — %ASA-4-106023 spoof/protocol deny (SYNTHETIC-BUT-VALID, F-7)

```text
<166>Sep 22 14:31:10 asa-5525 %ASA-4-106023: Deny tcp src outside:203.0.113.9/12345 dst inside:10.1.1.5/25 by access-group "outside_in" [0x0, 0x0]
```

### Field table (Cisco ASA)

| Fragment | Meaning | OCSF target |
| :--- | :--- | :--- |
| `%ASA-6-302013` / `Built` | session open | `activity_id 1` Open, `action Allowed` |
| `%ASA-6-302014` / `Teardown … duration … bytes …` | session close + counters | `activity_id 2` Close, `traffic.bytes`, duration context |
| `%ASA-4-106100` / `denied … -> …` | ACL deny | `activity_id 5` Refuse, `action Denied`, `firewall_rule.name outside_in` |
| `%ASA-4-106023` / `Deny … by access-group` | spoof/protocol deny | `activity_id 5` Refuse, `action Denied` |
| `outside:IP/port to inside:IP/port` | 5-tuple + interfaces | `src/dst_endpoint` (+ `interface_name` from ifc) |
| `duration 0:00:30` | h:mm:ss | duration context (parser keeps raw; OCSF `duration_time` if emitted) |
| `TCP FINs` / `TCP Reset` | close reason | `disposition` hint (`Allowed` vs `Reset`) |

### Expected OCSF — C-1 (4001 Open)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 1, "activity_name": "Open",
  "type_uid": 400101, "type_name": "Network Activity: Open",
  "time": 1727015420000,
  "severity_id": 1, "severity": "Informational",
  "action_id": 1, "action": "Allowed", "disposition_id": 1, "disposition": "Allowed",
  "src_endpoint": {"ip": "203.0.113.15", "port": 49152, "interface_name": "outside"},
  "dst_endpoint": {"ip": "10.1.1.25", "port": 80, "interface_name": "inside"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "302013", "name": "ASA-conn-setup"},
  "observables": [{"name": "203.0.113.15", "type": "IP Address"}, {"name": "10.1.1.25", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Cisco", "name": "ASA", "version": "9.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — C-2 (4001 Close)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 2, "activity_name": "Close",
  "type_uid": 400102, "type_name": "Network Activity: Close",
  "time": 1727015450000,
  "severity_id": 1, "severity": "Informational",
  "action_id": 1, "action": "Allowed", "disposition_id": 1, "disposition": "Allowed",
  "src_endpoint": {"ip": "203.0.113.15", "port": 49152, "interface_name": "outside"},
  "dst_endpoint": {"ip": "10.1.1.25", "port": 80, "interface_name": "inside"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 14502, "bytes": 14502, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "302014", "name": "ASA-conn-teardown"},
  "observables": [{"name": "203.0.113.15", "type": "IP Address"}, {"name": "10.1.1.25", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Cisco", "name": "ASA", "version": "9.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — C-3 (4001 Refuse, ACL deny)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 5, "activity_name": "Refuse",
  "type_uid": 400105, "type_name": "Network Activity: Refuse",
  "time": 1727015465000,
  "severity_id": 3, "severity": "Medium",
  "action_id": 2, "action": "Denied", "disposition_id": 2, "disposition": "Blocked",
  "src_endpoint": {"ip": "203.0.113.7", "port": 52344, "interface_name": "outside"},
  "dst_endpoint": {"ip": "192.168.1.10", "port": 80, "interface_name": "inside"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "106100", "name": "outside_in"},
  "observables": [{"name": "203.0.113.7", "type": "IP Address"}, {"name": "192.168.1.10", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Cisco", "name": "ASA", "version": "9.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — C-4 (4001 Refuse, 106023)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 5, "activity_name": "Refuse",
  "type_uid": 400105, "type_name": "Network Activity: Refuse",
  "time": 1727015470000,
  "severity_id": 4, "severity": "High",
  "action_id": 2, "action": "Denied", "disposition_id": 2, "disposition": "Blocked",
  "src_endpoint": {"ip": "203.0.113.9", "port": 12345, "interface_name": "outside"},
  "dst_endpoint": {"ip": "10.1.1.5", "port": 25, "interface_name": "inside"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "106023", "name": "outside_in"},
  "observables": [{"name": "203.0.113.9", "type": "IP Address"}, {"name": "10.1.1.5", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Cisco", "name": "ASA", "version": "9.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 3. Palo Alto PAN-OS — CSV over syslog (TRAFFIC + THREAT)

**Envelope:** syslog (RFC 3164/5424) carrying a positional CSV body. Field order is
version-dependent; positions below follow PAN-OS 11.x TRAFFIC/THREAT column order
(see PAN-OS Syslog Field Descriptions). Parser must be positional, not KV.

### Sample P-1 — TRAFFIC allow (SYNTHETIC-BUT-VALID, F-8)

```text
1,2024/09/22 14:30:25,001801000123,TRAFFIC,allow,2304,2024/09/22 14:30:25,10.0.70.54,192.168.6.215,0.0.0.0,0.0.0.0,Rule-Web-Allow,,,ssl,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2024/09/22 14:30:25,654123,1,54210,443,0,0,0x0,tcp,allow,1420,700,720,12,6,6,0
```

### Sample P-2 — TRAFFIC deny/drop icmp (SYNTHETIC-BUT-VALID, F-9)

```text
1,2024/09/22 14:30:31,001801000123,TRAFFIC,drop,2304,2024/09/22 14:30:31,192.168.10.50,198.51.100.80,0.0.0.0,0.0.0.0,Rule-Block-External,,,ping,vsys1,trust,untrust,ethernet1/2,ethernet1/1,log-forwarding,2024/09/22 14:30:31,0,1,0,0,0,0,0x0,icmp,deny,60,60,0,1,0,0,0
```

### Sample P-3 — THREAT vulnerability block (SYNTHETIC-BUT-VALID, F-10)

```text
1,2024/09/22 14:30:40,001801000123,THREAT,vulnerability,2304,2024/09/22 14:30:40,10.0.70.54,198.51.100.80,0.0.0.0,0.0.0.0,Rule-Web-Allow,,web-admin,,web-browsing,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2024/09/22 14:30:40,654124,1,54211,443,0,0,0x0,tcp,block,1200,600,600,8,critical,6,Apache-Log4Shell-RCE,91541,0x0,10.0.70.54,0,0,0,0
```

### Field table (PAN-OS, positions used)

| # | Column | Example | OCSF target |
| :--- | :--- | :--- | :--- |
| 4 | Type | `TRAFFIC` / `THREAT` | class stays `4001` in this fixture |
| 5 | Action/Subtype | `allow` `drop` / `vulnerability` | `action_id` + `activity_id` |
| 8/9 | Src/Dst IP | `10.0.70.54` | `src/dst_endpoint.ip` |
| 12 | Rule name | `Rule-Web-Allow` | `firewall_rule.name` |
| 16–19 | vsys, zones, interfaces | `Trust/Untrust`, `ethernet1/2` | `src/dst_endpoint.interface_name`, zone context |
| 24/25 | Src/Dst port | `54210/443` | `src/dst_endpoint.port` |
| 29 | Proto | `tcp/icmp` | `connection_info.protocol_num` |
| 30 | Action | `allow/deny/block` | `action Denied/Allowed`, `disposition` |
| 31–35 | bytes/packets | `1420,700,720,12` | `traffic.*` |
| THREAT sev/name | `critical`, `Apache-Log4Shell-RCE` | `severity 5/Critical` + observable signature | `severity_id`, `observables` |

### Expected OCSF — P-1 (4001 Traffic, allow)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 6, "activity_name": "Traffic",
  "type_uid": 400106, "type_name": "Network Activity: Traffic",
  "time": 1727015425000,
  "severity_id": 1, "severity": "Informational",
  "action_id": 1, "action": "Allowed", "disposition_id": 1, "disposition": "Allowed",
  "src_endpoint": {"ip": "10.0.70.54", "port": 54210, "interface_name": "ethernet1/2"},
  "dst_endpoint": {"ip": "192.168.6.215", "port": 443, "interface_name": "ethernet1/1"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 720, "bytes_out": 700, "bytes": 1420, "packets_in": 6, "packets_out": 6, "packets": 12},
  "firewall_rule": {"uid": "Rule-Web-Allow", "name": "Rule-Web-Allow"},
  "observables": [{"name": "10.0.70.54", "type": "IP Address"}, {"name": "192.168.6.215", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Palo Alto Networks", "name": "PAN-OS", "version": "11.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — P-2 (4001 Refuse, deny)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 5, "activity_name": "Refuse",
  "type_uid": 400105, "type_name": "Network Activity: Refuse",
  "time": 1727015431000,
  "severity_id": 3, "severity": "Medium",
  "action_id": 2, "action": "Denied", "disposition_id": 6, "disposition": "Dropped",
  "src_endpoint": {"ip": "192.168.10.50", "port": 0, "interface_name": "ethernet1/2"},
  "dst_endpoint": {"ip": "198.51.100.80", "port": 0, "interface_name": "ethernet1/1"},
  "connection_info": {"protocol_num": 1, "protocol_name": "ICMP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 60, "bytes": 60, "packets_in": 0, "packets_out": 1, "packets": 1},
  "firewall_rule": {"uid": "Rule-Block-External", "name": "Rule-Block-External"},
  "observables": [{"name": "192.168.10.50", "type": "IP Address"}, {"name": "198.51.100.80", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Palo Alto Networks", "name": "PAN-OS", "version": "11.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — P-3 (4001 Refuse, THREAT block)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 5, "activity_name": "Refuse",
  "type_uid": 400105, "type_name": "Network Activity: Refuse",
  "time": 1727015440000,
  "severity_id": 5, "severity": "Critical",
  "action_id": 2, "action": "Denied", "disposition_id": 2, "disposition": "Blocked",
  "src_endpoint": {"ip": "10.0.70.54", "port": 54211, "interface_name": "ethernet1/2"},
  "dst_endpoint": {"ip": "198.51.100.80", "port": 443, "interface_name": "ethernet1/1"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 600, "bytes_out": 600, "bytes": 1200, "packets_in": 4, "packets_out": 4, "packets": 8},
  "firewall_rule": {"uid": "Rule-Web-Allow", "name": "Rule-Web-Allow"},
  "observables": [{"name": "10.0.70.54", "type": "IP Address"}, {"name": "198.51.100.80", "type": "IP Address"}, {"name": "Apache-Log4Shell-RCE", "type": "Signature"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Palo Alto Networks", "name": "PAN-OS Threat Prevention", "version": "11.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 4. Suricata EVE JSON — alert / http / dns / tls / flow

**Envelope:** single-line JSON per event (`event_type` discriminates). Transported as
syslog MSG or filebeat-shipped JSON; fixture stores raw JSON lines.
Parser targets: `timestamp flow_id event_type src_ip src_port dest_ip dest_port proto`
plus `alert` / `http` / `dns` / `tls` / `flow` sub-objects.

### Sample S-1 — alert ET MALWARE (SYNTHETIC-BUT-VALID, F-11)

```json
{"timestamp":"2024-09-22T14:30:45.123456+0000","flow_id":1122334455,"event_type":"alert","src_ip":"10.1.100.30","src_port":53122,"dest_ip":"198.51.100.60","dest_port":80,"proto":"TCP","alert":{"action":"allowed","gid":1,"signature_id":2008105,"rev":4,"signature":"ET MALWARE Win32/Emotet CnC Beacon","category":"Trojan Activity","severity":1}}
```

### Sample S-2 — http (SYNTHETIC-BUT-VALID, F-12 → OCSF 4002)

```json
{"timestamp":"2024-09-22T14:30:46.654321+0000","flow_id":1122334466,"event_type":"http","src_ip":"10.1.100.30","src_port":53123,"dest_ip":"198.51.100.60","dest_port":80,"proto":"TCP","http":{"hostname":"example.com","url":"/login.php","http_method":"POST","status":200,"length":512}}
```

### Sample S-3 — dns query (SYNTHETIC-BUT-VALID, F-13 → OCSF 4003)

```json
{"timestamp":"2024-09-22T14:30:47.111111+0000","flow_id":1122334477,"event_type":"dns","src_ip":"10.1.100.31","src_port":53333,"dest_ip":"10.1.0.53","dest_port":53,"proto":"UDP","dns":{"type":"query","query":{"rrname":"evil.example.net","rrtype":"A"},"rcode":"NOERROR","answers":[{"rrname":"evil.example.net","rrtype":"A","rdata":"198.51.100.99"}]}}
```

### Sample S-4 — tls handshake (SYNTHETIC-BUT-VALID, F-14)

```json
{"timestamp":"2024-09-22T14:30:48.222222+0000","flow_id":1122334488,"event_type":"tls","src_ip":"10.1.100.32","src_port":54444,"dest_ip":"198.51.100.70","dest_port":443,"proto":"TCP","tls":{"subject":"CN=example.com","issuerdn":"CN=Example CA","sni":"example.com","version":"TLS 1.3","ja3":{"hash":"e7d705a3286e0d1c"}}}
```

### Sample S-5 — flow summary (SYNTHETIC-BUT-VALID, F-15)

```json
{"timestamp":"2024-09-22T14:30:49.333333+0000","flow_id":1122334499,"event_type":"flow","src_ip":"10.1.100.33","src_port":55555,"dest_ip":"203.0.113.50","dest_port":443,"proto":"TCP","flow":{"pkts_toserver":20,"pkts_toclient":18,"bytes_toserver":2400,"bytes_toclient":15000,"start":"2024-09-22T14:30:10.000000+0000","end":"2024-09-22T14:30:49.333333+0000"}}
```

### Field table (Suricata)

| JSON pointer | Meaning | OCSF target |
| :--- | :--- | :--- |
| `event_type` | `alert/http/dns/tls/flow` | class selector: alert/tls/flow→`4001`, http→`4002`, dns→`4003` |
| `src_ip/src_port/dest_ip/dest_port` | 5-tuple | `src/dst_endpoint` |
| `proto` | `TCP/UDP/ICMP` | `connection_info.protocol_num` |
| `alert.signature/severity/action` | sig text + sev + allowed/blocked | `severity_id`, `action/disposition`, observable Signature |
| `http.hostname/url/http_method/status` | web fields | `4002 http_request.*`, `dst_endpoint.domain` |
| `dns.query.rrname/rrtype/answers` | query + answers | `4003 dns_question.*` + observables |
| `tls.sni/version/subject` | handshake | `4001` + `tls_*` context / observables |
| `flow.bytes_*/pkts_*` | counters | `traffic.*` |

### Expected OCSF — S-1 alert (4001 Traffic, High)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 6, "activity_name": "Traffic",
  "type_uid": 400106, "type_name": "Network Activity: Traffic",
  "time": 1727015445000,
  "severity_id": 4, "severity": "High",
  "action_id": 1, "action": "Allowed", "disposition_id": 1, "disposition": "Allowed",
  "src_endpoint": {"ip": "10.1.100.30", "port": 53122},
  "dst_endpoint": {"ip": "198.51.100.60", "port": 80},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "observables": [{"name": "10.1.100.30", "type": "IP Address"}, {"name": "198.51.100.60", "type": "IP Address"}, {"name": "ET MALWARE Win32/Emotet CnC Beacon", "type": "Signature"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "OISF", "name": "Suricata", "version": "7.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

> NOTE: a `2004 Detection Finding` mapping is legitimate for Suricata alerts in
> production; this fixture pins alerts to `4001` per the task contract
> (`4001 or 4002/4003/3002`), preserving the signature in `observables`.

### Expected OCSF — S-2 http (4002 Request)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4002, "class_name": "HTTP Activity",
  "activity_id": 1, "activity_name": "Request",
  "type_uid": 400201, "type_name": "HTTP Activity: Request",
  "time": 1727015446000,
  "severity_id": 1, "severity": "Informational",
  "src_endpoint": {"ip": "10.1.100.30", "port": 53123},
  "dst_endpoint": {"ip": "198.51.100.60", "port": 80, "domain": "example.com"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 512, "bytes_out": 0, "bytes": 512, "packets_in": 0, "packets_out": 0, "packets": 0},
  "http_request": {"http_method": "POST", "url": "/login.php", "hostname": "example.com"},
  "http_response": {"status": 200},
  "observables": [{"name": "10.1.100.30", "type": "IP Address"}, {"name": "example.com", "type": "Hostname"}, {"name": "example.com/login.php", "type": "URL"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "OISF", "name": "Suricata", "version": "7.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — S-3 dns (4003 Query)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4003, "class_name": "DNS Activity",
  "activity_id": 1, "activity_name": "Query",
  "type_uid": 400301, "type_name": "DNS Activity: Query",
  "time": 1727015447000,
  "severity_id": 1, "severity": "Informational",
  "src_endpoint": {"ip": "10.1.100.31", "port": 53333},
  "dst_endpoint": {"ip": "10.1.0.53", "port": 53},
  "connection_info": {"protocol_num": 17, "protocol_name": "UDP", "direction_id": 3, "direction": "Lateral"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "dns_question": {"name": "evil.example.net", "type": "A"},
  "dns_answer": [{"name": "evil.example.net", "type": "A", "rdata": "198.51.100.99"}],
  "observables": [{"name": "evil.example.net", "type": "Hostname"}, {"name": "198.51.100.99", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "OISF", "name": "Suricata", "version": "7.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — S-4 tls (4001 Open)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 1, "activity_name": "Open",
  "type_uid": 400101, "type_name": "Network Activity: Open",
  "time": 1727015448000,
  "severity_id": 1, "severity": "Informational",
  "action_id": 3, "action": "Observed", "disposition_id": 1, "disposition": "Allowed",
  "src_endpoint": {"ip": "10.1.100.32", "port": 54444},
  "dst_endpoint": {"ip": "198.51.100.70", "port": 443, "domain": "example.com"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "observables": [{"name": "10.1.100.32", "type": "IP Address"}, {"name": "198.51.100.70", "type": "IP Address"}, {"name": "example.com", "type": "Hostname"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "OISF", "name": "Suricata", "version": "7.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — S-5 flow (4001 Traffic)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 6, "activity_name": "Traffic",
  "type_uid": 400106, "type_name": "Network Activity: Traffic",
  "time": 1727015449000,
  "severity_id": 1, "severity": "Informational",
  "action_id": 3, "action": "Observed", "disposition_id": 1, "disposition": "Allowed",
  "src_endpoint": {"ip": "10.1.100.33", "port": 55555},
  "dst_endpoint": {"ip": "203.0.113.50", "port": 443},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 15000, "bytes_out": 2400, "bytes": 17400, "packets_in": 18, "packets_out": 20, "packets": 38},
  "observables": [{"name": "10.1.100.33", "type": "IP Address"}, {"name": "203.0.113.50", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "OISF", "name": "Suricata", "version": "7.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 5. NGINX combined log (reverse-proxy / WAF-adjacent)

**Format:** `combined` — `$remote_addr - $remote_user [$time_local] "$method $uri $proto" $status $bytes "$referer" "$ua"`.
Parser targets: IP, timestamp, method, URI, status, bytes, referer, UA. Class `4002`.

### Sample N-1 — GET 200 (SYNTHETIC-BUT-VALID, F-16)

```text
203.0.113.20 - - [22/Sep/2024:14:31:00 +0000] "GET /index.html HTTP/1.1" 200 612 "-" "Mozilla/5.0 (X11; Linux x86_64) test-agent/1.0"
```

### Sample N-2 — SQLi probe 403 (SYNTHETIC-BUT-VALID, F-18)

```text
198.51.100.33 - - [22/Sep/2024:14:31:02 +0000] "GET /search?q=%27%20OR%201%3D1-- HTTP/1.1" 403 234 "https://example.com/" "sqlmap/1.7-test"
```

### Field table (NGINX)

| Token | Meaning | OCSF target |
| :--- | :--- | :--- |
| `remote_addr` | client IP | `src_endpoint.ip` |
| `time_local` | `22/Sep/2024:14:31:00 +0000` | `time` (→ epoch-ms) |
| `method uri proto` | request line | `http_request.http_method/.url` |
| `status` | `200/403` | `http_response.status`; `2xx`→Informational, `403`→Medium + Denied |
| `bytes` | body bytes | `traffic.bytes_in` |
| `referer` / `ua` | context | observables / `http_request.user_agent` |

### Expected OCSF — N-1 (4002 Request, 200)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4002, "class_name": "HTTP Activity",
  "activity_id": 1, "activity_name": "Request",
  "type_uid": 400201, "type_name": "HTTP Activity: Request",
  "time": 1727015460000,
  "severity_id": 1, "severity": "Informational",
  "src_endpoint": {"ip": "203.0.113.20"},
  "dst_endpoint": {"domain": "example.com"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 612, "bytes_out": 0, "bytes": 612, "packets_in": 0, "packets_out": 0, "packets": 0},
  "http_request": {"http_method": "GET", "url": "/index.html"},
  "http_response": {"status": 200},
  "observables": [{"name": "203.0.113.20", "type": "IP Address"}, {"name": "/index.html", "type": "URL"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "NGINX", "name": "nginx", "version": "1.24"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — N-2 (4002 Request, 403 SQLi)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4002, "class_name": "HTTP Activity",
  "activity_id": 1, "activity_name": "Request",
  "type_uid": 400201, "type_name": "HTTP Activity: Request",
  "time": 1727015462000,
  "severity_id": 3, "severity": "Medium",
  "src_endpoint": {"ip": "198.51.100.33"},
  "dst_endpoint": {"domain": "example.com"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 234, "bytes_out": 0, "bytes": 234, "packets_in": 0, "packets_out": 0, "packets": 0},
  "http_request": {"http_method": "GET", "url": "/search?q=%27%20OR%201%3D1--"},
  "http_response": {"status": 403},
  "observables": [{"name": "198.51.100.33", "type": "IP Address"}, {"name": "/search?q=%27%20OR%201%3D1--", "type": "URL"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "NGINX", "name": "nginx", "version": "1.24"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 6. CEF (ArcSight Common Event Format)

**Spec:** no RFC (ArcSight/Micro Focus/OpenText). Header
`CEF:Version|Vendor|Product|Version|SigID|Name|Severity|ext k=v …`, usually inside
RFC 3164/5424 syslog. Parser: `parse_cef` (VRL) or pipe-split + KV.

### Sample E-1 (SYNTHETIC-BUT-VALID, F-19)

```text
<134>Sep 22 14:31:15 pa-220 CEF:0|Palo Alto Networks|PAN-OS|11.1.0|deleted|Threat Alert|8|src=10.0.70.54 spt=54211 dst=198.51.100.80 dpt=443 proto=tcp act=block msg=Apache-Log4Shell-RCE app=web-browsing rule=Rule-Web-Allow
```

### Field table (CEF)

| CEF part | Meaning | OCSF target |
| :--- | :--- | :--- |
| header `Vendor/Product/Version` | device identity | `metadata.product` |
| `SigID/Name/Severity(0-10)` | `deleted/Threat Alert/8` | `firewall_rule.uid`, severity → `severity_id 4/High` |
| `src/spt/dst/dpt/proto` | 5-tuple | `src/dst_endpoint`, `connection_info` |
| `act` | `block` | `action Denied`, `activity Refuse` |
| `msg/app/rule` | threat + context | observables + `firewall_rule.name` |

### Expected OCSF — E-1 (4001 Refuse)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 5, "activity_name": "Refuse",
  "type_uid": 400105, "type_name": "Network Activity: Refuse",
  "time": 1727015475000,
  "severity_id": 4, "severity": "High",
  "action_id": 2, "action": "Denied", "disposition_id": 2, "disposition": "Blocked",
  "src_endpoint": {"ip": "10.0.70.54", "port": 54211},
  "dst_endpoint": {"ip": "198.51.100.80", "port": 443},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "deleted", "name": "Rule-Web-Allow"},
  "observables": [{"name": "10.0.70.54", "type": "IP Address"}, {"name": "198.51.100.80", "type": "IP Address"}, {"name": "Apache-Log4Shell-RCE", "type": "Signature"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Palo Alto Networks", "name": "PAN-OS", "version": "11.1.0"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 7. LEEF (IBM QRadar Log Event Extended Format)

**Spec:** no RFC (IBM). `LEEF:2.0|Vendor|Product|Version|EventID|<delim>|k<delim>v…`.
Fixture uses `^` as the explicit LEEF 2.0 delimiter (avoids invisible-tab bugs).

### Sample L-1 (SYNTHETIC-BUT-VALID, F-20)

```text
<134>Sep 22 14:31:18 qradar-collector LEEF:2.0|Check Point|VPN-1|R81.20|FirewallSessionClosed|^|usrName=-^src=10.1.100.40^srcPort=60001^dst=198.51.100.90^dstPort=443^proto=6^action=Drop^ruleName=Cleanup-rule^bytesIn=0^bytesOut=840^packetsIn=0^packetsOut=2^duration=3
```

### Field table (LEEF)

| Key | Meaning | OCSF target |
| :--- | :--- | :--- |
| header `Vendor/Product/Version/EventID` | device + event | `metadata.product`, `firewall_rule.uid` |
| `src/srcPort/dst/dstPort/proto` | 5-tuple | `src/dst_endpoint`, `connection_info` |
| `action` | `Drop` | `activity Refuse`, `action Denied`, `disposition Dropped` |
| `bytesIn/bytesOut/packets*` | counters | `traffic.*` |
| `ruleName/duration` | context | `firewall_rule.name`, duration |

### Expected OCSF — L-1 (4001 Refuse)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 5, "activity_name": "Refuse",
  "type_uid": 400105, "type_name": "Network Activity: Refuse",
  "time": 1727015478000,
  "severity_id": 3, "severity": "Medium",
  "action_id": 2, "action": "Denied", "disposition_id": 6, "disposition": "Dropped",
  "src_endpoint": {"ip": "10.1.100.40", "port": 60001},
  "dst_endpoint": {"ip": "198.51.100.90", "port": 443},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 840, "bytes": 840, "packets_in": 0, "packets_out": 2, "packets": 2},
  "firewall_rule": {"uid": "FirewallSessionClosed", "name": "Cleanup-rule"},
  "observables": [{"name": "10.1.100.40", "type": "IP Address"}, {"name": "198.51.100.90", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Check Point", "name": "VPN-1", "version": "R81.20"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 8. Check Point (Log Exporter syslog)

**Spec:** Check Point Log Exporter (sk122323) — space-separated `key=value`
(plain syslog, no CEF wrapper unless configured). Parser targets
`action src dst sport dport proto service rule_name`.

### Sample K-1 — accept HTTPS (SYNTHETIC-BUT-VALID, F-21)

```text
<134>Sep 22 14:31:22 logexporter CheckPoint: time=1727015482 action=Accept src=10.1.100.50 sport=61001 dst=198.51.100.100 dport=443 proto=6 service=https rule_name="Allow-Outbound-HTTPS" rule_uid={1a2b3c4d-0000-0000-0000-000000000001} product="VPN-1 & FireWall-1" ifdir=outbound
```

### Field table (Check Point)

| Key | Meaning | OCSF target |
| :--- | :--- | :--- |
| `action` | `Accept/Drop` | `activity Traffic/Refuse`, `action Allowed/Denied` |
| `src/sport/dst/dport/proto` | 5-tuple | `src/dst_endpoint`, `connection_info` |
| `service` | `https` | port-443 corroboration |
| `rule_name/rule_uid` | policy | `firewall_rule` |
| `product/ifdir` | device + direction | `metadata.product`, `direction Outbound` |

### Expected OCSF — K-1 (4001 Traffic, allow)

```json
{
  "category_uid": 4, "category_name": "Network Activity",
  "class_uid": 4001, "class_name": "Network Activity",
  "activity_id": 6, "activity_name": "Traffic",
  "type_uid": 400106, "type_name": "Network Activity: Traffic",
  "time": 1727015482000,
  "severity_id": 1, "severity": "Informational",
  "action_id": 1, "action": "Allowed", "disposition_id": 1, "disposition": "Allowed",
  "src_endpoint": {"ip": "10.1.100.50", "port": 61001},
  "dst_endpoint": {"ip": "198.51.100.100", "port": 443},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 2, "direction": "Outbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "firewall_rule": {"uid": "{1a2b3c4d-0000-0000-0000-000000000001}", "name": "Allow-Outbound-HTTPS"},
  "observables": [{"name": "10.1.100.50", "type": "IP Address"}, {"name": "198.51.100.100", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Check Point", "name": "VPN-1 & FireWall-1", "version": "R81.20"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 9. VPN authentication (Cisco ASA AnyConnect → OCSF 3002)

**Envelope:** RFC 3164 ASA text (`%ASA-6-113004` success / `%ASA-4-113005` failure).
Only section mapping to IAM (`category_uid 3`, `class_uid 3002`). Parser targets
`Group … User … IP … Authentication (Accepted|Rejected)`.

### Sample V-1 — AnyConnect logon success (SYNTHETIC-BUT-VALID, F-22)

```text
<166>Sep 22 14:31:30 asa-5525 %ASA-6-113004: AAA user authentication Successful : server = LDAP : user = alice : user IP = 203.0.113.44
```

### Sample V-2 — AnyConnect logon failure (SYNTHETIC-BUT-VALID, F-23)

```text
<166>Sep 22 14:31:35 asa-5525 %ASA-4-113005: AAA user authentication Rejected : reason = Invalid password : server = LDAP : user = bob : user IP = 198.51.100.55
```

### Field table (VPN auth)

| Fragment | Meaning | OCSF target |
| :--- | :--- | :--- |
| `113004 Successful` | logon ok | `3002 activity 1 Logon`, `status Success`, severity Informational |
| `113005 Rejected` | logon fail | `3002 activity 1 Logon`, `status Failure`, severity Medium |
| `user = alice` | account | `user.name` (+ `user.uid` if directory-known) |
| `user IP` | public source | `src_endpoint.ip` |
| `server = LDAP` | auth backend | `auth_protocol` context |

### Expected OCSF — V-1 (3002 Logon success)

```json
{
  "category_uid": 3, "category_name": "Identity & Access Management",
  "class_uid": 3002, "class_name": "Authentication",
  "activity_id": 1, "activity_name": "Logon",
  "type_uid": 300201, "type_name": "Authentication: Logon",
  "time": 1727015490000,
  "severity_id": 1, "severity": "Informational",
  "status_id": 1, "status": "Success",
  "user": {"name": "alice"},
  "src_endpoint": {"ip": "203.0.113.44"},
  "dst_endpoint": {"ip": "10.1.1.1"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "observables": [{"name": "alice", "type": "User Name"}, {"name": "203.0.113.44", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Cisco", "name": "ASA AnyConnect", "version": "9.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

### Expected OCSF — V-2 (3002 Logon failure)

```json
{
  "category_uid": 3, "category_name": "Identity & Access Management",
  "class_uid": 3002, "class_name": "Authentication",
  "activity_id": 1, "activity_name": "Logon",
  "type_uid": 300201, "type_name": "Authentication: Logon",
  "time": 1727015495000,
  "severity_id": 3, "severity": "Medium",
  "status_id": 2, "status": "Failure",
  "user": {"name": "bob"},
  "src_endpoint": {"ip": "198.51.100.55"},
  "dst_endpoint": {"ip": "10.1.1.1"},
  "connection_info": {"protocol_num": 6, "protocol_name": "TCP", "direction_id": 1, "direction": "Inbound"},
  "traffic": {"bytes_in": 0, "bytes_out": 0, "bytes": 0, "packets_in": 0, "packets_out": 0, "packets": 0},
  "observables": [{"name": "bob", "type": "User Name"}, {"name": "198.51.100.55", "type": "IP Address"}],
  "metadata": {"version": "1.9.0", "product": {"vendor_name": "Cisco", "name": "ASA AnyConnect", "version": "9.x"}, "history_hash": "BLAKE3:<computed-at-ingest>", "vault_uri": "parquet:<block>#<offset>"}
}
```

---

## 10. Fixture-only extras (lines F-24…F-30, same mapping rules)

The golden fixture reuses the patterns above for 7 extra valid lines so the
mix reaches 30 valid + 5 alien = 35 total. No new syntax is introduced:

| Fixture line | Pattern reused | Key values |
| :--- | :--- | :--- |
| F-24 | Fortinet forward close (§1, allow) | `srcip=10.1.100.60:62001 → dstip=203.0.113.80:8443`, `sentbyte=9100 rcvdbyte=22100`, `time +130 s` |
| F-25 | Cisco `%ASA-6-302014` UDP teardown (§2) | `inside:10.1.1.60/53333 → outside:10.1.0.53/53`, `bytes 940`, `UDP` |
| F-26 | PAN-OS TRAFFIC drop (§3 P-2 shape) | `192.0.2.44 → 203.0.113.77`, `dns`, `deny`, `bytes 180` |
| F-27 | Suricata alert Log4Shell (§4 S-1 shape) | `10.1.100.70:54433 → 198.51.100.90:8080`, `ET EXPLOIT Apache Log4j RCE`, `severity 1` |
| F-28 | NGINX 404 (§5 N-1 shape) | `203.0.113.31 GET /old-page 404 168` |
| F-29 | Check Point Drop (§8 K-1 shape, Drop) | `src=10.1.100.80 → dst=203.0.113.90:23`, `action=Drop`, `rule_name="Cleanup-rule"` |
| F-30 | VPN failure (§9 V-2 shape) | `user = carol`, `user IP = 192.0.2.99`, `reason = Invalid password` |

Their `ocsf_expected.jsonl` entries follow the section templates byte-for-byte
in structure (only IPs/ports/times/bytes/severity flip per the table).

## 11. Alien / ambiguous lines (DLQ test, F-31…F-35 → `{"action":"REJECT"}`)

| # | Line (in `golden_mixed.log`) | Why it must REJECT |
| :--- | :--- | :--- |
| F-31 | `FROBNICATOR v9.9 [frobnicate] splines=reticulated widgets=<UNRECOGNIZED> quantum=42` | unknown proprietary single-token vendor, no parser |
| F-32 | `{"event":"???","frobnicate":{"splines":true},"vendor":"unknown-box-9000"}` | valid JSON but unknown schema (no `event_type`, no CEF/LEEF/EVE keys) |
| F-33 | `Sep 22 14:33:01 unknownbox weirddaemon[123]: wibble wobble frobnicate splines exceeded threshold splines=???` | RFC 3164 envelope but unknown app + free-text body, no grammar |
| F-34 | `--- TRUNCATED BINARY len=512 crc=deadbeef <unparseable>` | truncation marker / binary placeholder, not a log grammar |
| F-35 | `......` | ambiguous dots-only line (empty-ish content, no fields) |

Router contract: heuristic router flags `UNMAPPED` → append to `dlq.log` JSONL
(`utf8:`-prefixed `raw_payload`, see DATA_DICTIONARY §1) → `prism-brain`
(Drain3 + OpenJev + Ollama) drafts VRL → TUI `[Y]` approval → hot-reload.
Harness expectation for all five: exactly `{"action":"REJECT"}`.

---

## Coverage checklist

- [x] Fortinet FortiGate KV (RFC 5424): forward close / deny / UTM IPS (§1, F-1…F-3 + F-24)
- [x] Cisco ASA 302013 build / 302014 teardown / 106100 ACL deny / 106023 deny (§2, F-4…F-7 + F-25)
- [x] Palo Alto TRAFFIC allow / TRAFFIC drop / THREAT vuln-block (§3, F-8…F-10 + F-26)
- [x] Suricata EVE alert / http→4002 / dns→4003 / tls / flow (§4, F-11…F-15 + F-27)
- [x] NGINX combined 200 / 403-SQLi (§5, F-16/F-18 + F-28; F-17 POST-401 variant in fixture)
- [x] CEF (§6, F-19)
- [x] LEEF 2.0 `^`-delimited (§7, F-20)
- [x] Check Point Log Exporter accept (§8, F-21 + F-29 Drop)
- [x] VPN auth success + failure → 3002 (§9, F-22/F-23 + F-30)
- [x] 5 alien/ambiguous → REJECT (§11, F-31…F-35)
