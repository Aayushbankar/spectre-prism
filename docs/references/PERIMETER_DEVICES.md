# Perimeter Network Devices & Log Specifications

> **Purpose:** Verified documentation on perimeter network devices and actual log structures for top enterprise vendors.
> **Source:** Official vendor administration guides, syslog reference manuals, and knowledge base articles.

## 1. What Devices Count as "Perimeter Network Devices"?

A perimeter network device sits at an organizational boundary (e.g., between a LAN and the Internet, or a DMZ).

### Functional Taxonomy
*   **Next-Generation Firewalls (NGFW):** Fortinet FortiGate, Palo Alto PA-Series, Cisco ASA/FTD, Check Point. (Logs: Session creation/teardown, allowed/denied flows, NAT mappings).
*   **Intrusion Detection/Prevention (IDS/IPS):** Cisco Firepower, Fortinet IPS. (Logs: Signature ID, attacker/victim IP, payload classification).
*   **VPN Concentrators:** Cisco AnyConnect, Palo Alto GlobalProtect. (Logs: User auth, tunnel negotiation, bytes sent/received).
*   **Web Application Firewalls (WAF):** F5, FortiWeb, Cloudflare WAF. (Logs: HTTP methods, URI, violation signatures).
*   **Secure Web Gateways (SWG) / Proxies:** Zscaler, Symantec ProxySG. (Logs: Target URL, HTTP status, categorized threat status).

---

## 2. Vendor Log Formats & Documentation

| Vendor / Platform | Default Formats | Official Reference |
|---|---|---|
| **Fortinet FortiGate** (FortiOS) | **Proprietary Key-Value (`key=value`)**, CEF | [FortiOS Log Message Reference](https://docs.fortinet.com/document/fortigate/7.4.0/fortios-log-message-reference) |
| **Cisco ASA** | **Proprietary Structured Text** (`%ASA-sev-ID: text`) | [ASA Syslog Messages Guide](https://www.cisco.com/c/en/us/td/docs/security/asa/syslog/b_as_syslog_guide.html) |
| **Palo Alto Networks** (PAN-OS) | **Comma-Separated Values (CSV)**, Custom | [PAN-OS Syslog Field Descriptions](https://docs.paloaltonetworks.com/pan-os/11-2/pan-os-admin/monitoring/use-syslog-for-monitoring/syslog-field-descriptions) |
| **Check Point** | **Proprietary Binary**, CEF/JSON via Exporter | [Check Point Log Exporter (sk122323)](https://supportcenter.checkpoint.com/supportcenter/portal?eventSubmit_doGoviewsolutiondetails=&solutionid=sk122323) |

---

## 3. Real Log Samples

### Fortinet FortiGate (Key-Value)
**Forward Traffic Log (Session End):**
```text
date=2018-12-27 time=11:07:55 logid="0000000013" type="traffic" subtype="forward" level="notice" vd="vdom1" eventtime=1545937675 srcip=10.1.100.11 srcport=54190 srcintf="port12" dstip=52.53.140.235 dstport=443 dstintf="port11" sessionid=402 proto=6 action="close" policyid=1 policytype="policy" service="HTTPS" dstcountry="United States" trandisp="snat" transip=172.16.200.1 appid=40568 app="HTTPS.BROWSER" duration=2 sentbyte=3652 rcvdbyte=146668 sentpkt=58 rcvdpkt=105 utmaction="allow"
```
*Key extraction targets: `srcip`, `srcport`, `dstip`, `dstport`, `action`, `sentbyte`, `rcvdbyte`, `transip` (NAT).*

### Cisco ASA (Structured Text)
**Teardown TCP Connection (`%ASA-6-302014`):**
```text
%ASA-6-302014: Teardown TCP connection 123456 for inside:192.168.1.50/49152 to outside:198.51.100.25/443 duration 0:00:15 bytes 4520 TCP FINs
```
*Key extraction targets: Message ID `302014`, source IP `192.168.1.50`, source port `49152`, dest IP `198.51.100.25`, dest port `443`, duration, bytes.*

### Palo Alto PAN-OS (Positional CSV)
**Traffic Log:**
```text
1,2021/08/24 11:51:12,010108010441,TRAFFIC,start,2305,2021/08/24 11:51:12,10.0.70.54,192.168.6.215,0.0.0.0,0.0.0.0,Rule-Web,user1,,ssl,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2021/08/24 11:51:12,654123,1,54210,443,0,0,0x0,tcp,allow,1420,700,720,12,6,6,0
```
*Requires positional parsing based on PAN-OS version schema.*

---

## Architectural Takeaways for SIEM Engineering
1. **Format Discrepancies:** No universal standard exists natively on the boxes. KV (Fortinet), Text Grammars (Cisco), CSV (Palo Alto).
2. **Session Lifecycle:** Firewalls generate logs for Session Start (no bytes) and Session End (with bytes). Mapping must account for both states.
3. **NAT Discrepancies:** Parsers must account for both pre-NAT (`srcip`) and post-NAT (`transip`) addresses within the same log line to provide complete traceability.
