# Dataset Strategy & Benchmark Corpus for PRISM

**Project:** PRISM (SIH26156 - NTRO)  
**Classification:** Research, Testing, and Evaluation Data Strategy

---

## 1. Operational Context & National Security Constraints
Real Indian government perimeter network telemetry (from NTRO, CERT-In, NCIIPC, NIC, or defense firewalls) is strictly classified under the Official Secrets Act and CERT-In directions. Such logs are never hosted on public repositories like `data.gov.in`.

To evaluate PRISM with high academic rigor and satisfy the SIH hackathon evaluation jury, we employ a **4-Tier Hybrid Dataset Strategy**:
1. **Academic Intrusion & Flow Datasets:** For high-volume (EPS) stress testing.
2. **Standard Log Benchmarks (LogPAI Loghub-2.0):** For measuring parser template extraction accuracy (Drain3).
3. **Real-world Vendor Perimeter Syslog Samples:** For validating exact regex/VRL field extraction (Fortinet, Cisco ASA, Palo Alto).
4. **Synthetic High-Throughput Syslog Generator:** For live streaming packet replay via UDP/QUIC to port 514.

---

## 2. Benchmark & Evaluation Datasets

### A. Academic Intrusion & Traffic Volume Benchmarks
*   **UNSW-NB15 Dataset (ACCS):**
    *   *Relevance:* Widely recognized by Indian academic and defense research bodies (IITs, DRDO citations).
    *   *Payload:* Raw PCAPs and pre-extracted flow records covering 9 attack classes (Fuzzers, Analysis, Backdoors, DoS, Exploits, Generic, Reconnaissance, Shellcode, Worms).
    *   *Application in PRISM:* Used to replay raw network flows into Syslog events and benchmark PRISM's zero-copy ingestion throughput under simulated multi-vector cyber attacks.
*   **CIC-IDS-2017 / CSE-CIC-IDS2018 (Canadian Institute for Cybersecurity):**
    *   *Relevance:* De facto international standard for IDS/IPS and firewall evaluation.
    *   *Payload:* Multi-gigabyte traffic captures containing benign background enterprise traffic mixed with modern attacks (Brute Force, Heartbleed, Botnet, DoS, DDoS, Web Attacks, and Infiltration).
    *   *Application in PRISM:* Stress-testing the Rust Data Plane to verify zero packet drops and zero CPU saturation at 50,000+ EPS.

### B. Standard Log Parser Benchmarks (Loghub-2.0)
*   **LogPAI Loghub & Loghub-2.0 (ISSRE '23):**
    *   *Repository:* `https://github.com/logpai/loghub` and `loghub-2.0`
    *   *Relevance:* The premier academic benchmark for AI/SLM and heuristic log parsing evaluation.
    *   *Datasets Used:* Linux Syslog, Apache Access/Error logs, Windows Event Logs, and Android logs.
    *   *Application in PRISM:* Used to independently benchmark the Drain3 compressor and Open Jev classification accuracy in `prism-brain` against ground-truth template labels.

### C. Open Perimeter & Security Datasets
*   **AIT Log Data Sets (Zenodo `records/6475510`):**
    *   Synthetic and captured logs from simulated enterprise networks, featuring multi-host firewall, syslog, VPN, and DNS logs mapped to attack timelines.
*   **Cybersecurity Threat Detection Logs (Kaggle):**
    *   Over 6 million labeled records simulating enterprise perimeter firewalls, network IDS, and proxy events with allowed/blocked decisions and threat classifications.
*   **SecRepo Security Data Repository:**
    *   Curated open collection containing raw sample logs from Snort/Suricata IDS, Zeek (Bro) network analyzers, IPTables, and ModSecurity WAF.

---

## 3. Real Vendor Syslog Sample Corpus

To verify VRL translation into OCSF Class 4001 (Network Activity), PRISM maintains verified test samples for the top 3 perimeter firewall vendors:

### 1. Fortinet FortiGate (FortiOS)
*   **Format:** Key-Value structured Syslog (RFC 5424 transport).
*   **Sample Traffic Log Line:**
    ```text
    date=2024-09-22 time=14:30:01 devname="FG-500E" devid="FG500E4Q17000123" logid="0000000013" type="traffic" subtype="forward" level="notice" vd="root" srcip=192.168.1.105 srcport=54210 srcintf="port1" dstip=198.51.100.45 dstport=443 dstintf="port2" polid=4 proto=6 action="close" duration=15 sentbyte=4520 rcvdbyte=8920 app="HTTPS"
    ```
*   **Target OCSF Class 4001 Mapping:**
    *   `src_endpoint.ip` = `"192.168.1.105"`
    *   `src_endpoint.port` = `54210`
    *   `dst_endpoint.ip` = `"198.51.100.45"`
    *   `dst_endpoint.port` = `443`
    *   `activity_id` = `2` (Close/Deny/Reset mapped via VRL)

### 2. Cisco ASA / Firepower
*   **Format:** BSD Syslog (RFC 3164) with `%ASA-` prefix codes.
*   **Sample Teardown Log Line:**
    ```text
    %ASA-6-302014: Teardown TCP connection 987654321 for outside:203.0.113.15/49152 to inside:10.1.1.25/80 duration 0:01:30 bytes 14502 TCP FINs
    ```
*   **Target OCSF Class 4001 Mapping:**
    *   `src_endpoint.ip` = `"203.0.113.15"`
    *   `src_endpoint.port` = `49152`
    *   `dst_endpoint.ip` = `"10.1.1.25"`
    *   `dst_endpoint.port` = `80`
    *   `activity_id` = `2` (Teardown/Close)

### 3. Palo Alto Networks (PAN-OS)
*   **Format:** Comma-Separated Values (CSV) over Syslog.
*   **Sample Traffic Log Line:**
    ```text
    1,2024/09/22 14:30:01,001801000123,TRAFFIC,drop,2304,2024/09/22 14:30:01,192.168.10.50,198.51.100.80,0.0.0.0,0.0.0.0,Rule-Block-External,,,ping,vsys1,trust,untrust,ethernet1/2,ethernet1/1,log-forwarding,2024/09/22 14:30:01,0,1,60,0,0,0,0,0x0,icmp,deny,60,0,0,0,0,,0,0,0,0,0,threat
    ```
*   **Target OCSF Class 4001 Mapping:**
    *   `src_endpoint.ip` = `"192.168.10.50"`
    *   `dst_endpoint.ip` = `"198.51.100.80"`
    *   `activity_id` = `2` (Deny/Drop)

---

## 4. Live Traffic Generator & DLQ Testing Strategy

To evaluate the system dynamically during development and the 2-minute demo video:

### A. Synthetic High-Throughput Streamer (`tools/streamer.py`)
*   Generates live, synthetic Cisco ASA, Fortinet, and Palo Alto Syslog UDP datagrams blasted directly to `127.0.0.1:514`.
*   Can be throttled or dialed up to 50,000+ EPS to demonstrate the Ratatui TUI live throughput gauge.

### B. The "Alien / Unknown" Schema Generator (For DLQ & AI Control Plane)
*   Injects unrecognized perimeter log formats:
    *   *Format A:* JSON-encoded Suricata EVE flow logs (`{"event_type": "alert", "src_ip": ...}`).
    *   *Format B:* NGINX Reverse Proxy perimeter access logs (`10.0.0.1 - - [22/Sep/2024:14:30:01] "GET /api HTTP/1.1"`).
    *   *Format C:* Proprietary VPN concentrator session logs.
*   **Expected System Behavior:**
    1. Heuristic router flags as `UNMAPPED`.
    2. Writes to `dlq.log`.
    3. `prism-brain` detects file change via `watchdog`.
    4. Drain3 compresses 10,000 alien logs into a single template.
    5. Open Jev classifies device type with >95% confidence.
    6. Ollama drafts a new `.vrl` script and signature.
    7. TUI prompts operator for `[Y]` approval.
    8. Zero dropped packets; router hot-reloads dynamically.
