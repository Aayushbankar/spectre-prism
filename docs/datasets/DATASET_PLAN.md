# Dataset Strategy for SIH26156

Due to strict operational security (OPSEC) and national compliance laws, real Indian government perimeter network logs (NTRO, CERT-In, NCIIPC) are classified and never published to public repositories like `data.gov.in`. 

To rigorously test PRISM and satisfy the hackathon evaluation jury, we will use a hybrid dataset strategy utilizing gold-standard academic intrusion datasets alongside synthetically generated vendor-specific (Cisco/Fortinet) logs.

## 1. Primary Evaluation Datasets

### A. The UNSW-NB15 Dataset
*   **Origin:** Australian Centre for Cyber Security (ACCS).
*   **Relevance:** Widely cited in Indian cybersecurity academia (e.g., IIT, NIT papers).
*   **Content:** Contains raw network packets (pcap) and pre-processed CSV flow logs spanning 9 attack families (Fuzzers, Analysis, Backdoors, DoS, Exploits, Generic, Reconnaissance, Shellcode, Worms).
*   **How PRISM Uses It:** We will stream the CSV flow logs through `prism-core` to benchmark parsing throughput (EPS) and validate OCSF Network Activity mapping.

### B. CIC-IDS-2017 / CSE-CIC-IDS2018
*   **Origin:** Canadian Institute for Cybersecurity.
*   **Relevance:** The de facto global standard for IDS evaluation.
*   **Content:** Realistic background traffic interspersed with DoS, DDoS, Brute Force, XSS, and SQL Injection attacks. 
*   **How PRISM Uses It:** Proves the system can handle extreme traffic volumes (high EPS) without dropping packets or saturating CPU.

## 2. Synthetic Perimeter Logs (Vendor Specific)

Because academic datasets are often CSVs (not actual Cisco/Fortinet Syslog), we must generate synthetic data to test the Data Plane's vendor parsing logic.

### Methodology
We will create a Python script (`generate_syslog.py`) that uses Faker to generate millions of realistic log lines matching the exact structural formats defined in our `PERIMETER_DEVICES.md` research.

**Fortinet Synthetic Format:**
```text
date=2024-09-22 time=14:30:01 logid="0000000013" type="traffic" subtype="forward" level="notice" vd="root" srcip=<RANDOM_IP> srcport=<RANDOM_PORT> dstip=<RANDOM_IP> dstport=443 action="close" sentbyte=<INT> rcvdbyte=<INT>
```

**Cisco ASA Synthetic Format:**
```text
%ASA-6-302014: Teardown TCP connection <ID> for outside:<RANDOM_IP>/<RANDOM_PORT> to inside:<RANDOM_IP>/443 duration 0:00:15 bytes <INT> TCP FINs
```

## 3. The "Unknown Format" Dataset (For AI Testing)
To demonstrate the AI Control Plane (Brain), we will feed PRISM a completely foreign dataset (e.g., NGINX web access logs or raw JSON AWS CloudTrail logs). 

**Demo Workflow:**
1. PRISM's Data Plane fails to match Cisco/Fortinet rules.
2. 10,000 NGINX logs are dumped into the Dead Letter Queue (DLQ).
3. The AI groups them, infers the schema, and auto-generates the VRL parser.
