# PRISM Kibana Dashboard Design Guide
**Project:** PRISM (SIH26156)  
**Target Audience:** UI/UX Designers & Frontend Developers  

## 1. Introduction: What are we building?
We are building the **"Executive View"** dashboard for our PRISM project demo. 

PRISM is a powerful backend engine that reads messy, completely different security logs from various firewalls (like Cisco, Fortinet, and Palo Alto) and magically transforms them all into one single, clean, standardized format. 

Your job is to design a dashboard (using Kibana) that visualizes this clean data. During our demo, we will show a split-screen:
* **Left Screen (The Engine Room):** A matrix-style terminal showing our Rust backend processing 50,000 logs per second.
* **Right Screen (The Executive View):** The dashboard **you** design, showing the beautiful, standardized results of that processing.

## 2. The Vibe: Professional Industry SIEM Dashboards
Security Information and Event Management (SIEM) dashboards are like the "Air Traffic Control" for cybersecurity. 
When designing this, aim for the aesthetic of industry leaders like **Splunk, Datadog, or CrowdStrike**.

**Design Principles:**
1. **Dark Mode is King:** Security analysts stare at these screens for 12 hours a day. Use dark themes (deep blues, dark grays, blacks) to reduce eye strain.
2. **High Contrast Neon Accents:** Use colors intentionally to draw attention to danger.
   * 🔴 **Red:** Blocked traffic, high severity threats, attacks.
   * 🟡 **Yellow/Orange:** Warnings, resets, suspicious activity.
   * 🟢 **Green:** Allowed, safe, normal traffic.
   * 🔵 **Blue/Cyan:** Neutral data, volume metrics.
3. **Data Density but Clean:** They need to see a lot of information at a glance, but it shouldn't look cluttered. Use clear grid layouts, distinct panels, and bold typography for big numbers.

## 3. The Data We Provide & How to Visualize It

Our backend sends data to Elasticsearch in a format called **OCSF** (Open Cybersecurity Schema Framework). You don't need to know how it works, just know that you have access to the following fields, and here is how you should visualize them:

### A. The Threat Map (Where are attacks coming from?)
* **The Data:** `src_endpoint.ip` (The IP address of the person connecting).
* **The Visualization:** A **Geo-IP / Region Map**. This is the visual centerpiece of the dashboard. It shows a map of the world with glowing dots or heatmaps indicating where traffic is originating. It looks incredibly cool and is a staple of security demos.

### B. Traffic Disposition (What did the firewall do?)
* **The Data:** `activity_id`. This is a number representing the action taken. (`1` = Allowed, `2` = Deny/Blocked, `3` = Reset).
* **The Visualization:** A **Donut Chart or Pie Chart**. 
  * Make the "Deny" slice Red.
  * Make the "Allow" slice Green.
  * This proves to the judges that PRISM successfully understands actions from all different firewall brands.

### C. Live Throughput (Are we processing fast?)
* **The Data:** `time` (When the event happened).
* **The Visualization:** A **Time-Series Area Chart or Line Chart** (EPS - Events Per Second). 
  * The X-axis is Time.
  * The Y-axis is the Count of logs.
  * This chart should look like a pulse, showing a massive spike when we blast 50,000 logs into the system during the demo.

### D. The Forensic Ledger (The "Receipts")
* **The Data:** `severity` (e.g., "High"), `src_endpoint.ip`, `dst_endpoint.ip` (target IP), and crucially, `metadata.provenance_hash`.
* **The Visualization:** A **Data Table** at the bottom of the dashboard.
  * Filter this table to only show "High" severity events.
  * **The `provenance_hash` is our secret weapon:** It looks like a long string of random characters (e.g., `a1b2c3d4...`). To a judge, this proves our system is legally compliant (Section 65B of the Indian Evidence Act), acting as a digital signature for the log. Make sure this column is clearly visible in the table.

## 4. Demo Execution (What will happen live)
When the judges are watching:
1. The dashboard will be empty.
2. We will hit "Start" on our backend.
3. The dashboard should be set to **auto-refresh every 1 second**.
4. Suddenly, the Map will light up, the Donut chart will spin, and the Line Chart will spike, instantly visualizing thousands of normalized threats. 

Focus on making those visual transitions look smooth, professional, and undeniably "cybersecurity".
