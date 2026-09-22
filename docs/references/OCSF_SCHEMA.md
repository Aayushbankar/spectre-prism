# Open Cybersecurity Schema Framework (OCSF) Reference

> **Purpose:** Verified schema definitions, class UIDs, and field structures for normalizing perimeter network logs.
> **Source:** Official OCSF schema repository (`schema.ocsf.io`, v1.9.0).

## 1. What is OCSF?
- **Founders:** AWS and Splunk (based on Symantec ICD Schema).
- **Status:** Linux Foundation open-source project (as of Nov 2024).
- **Current Version:** `1.9.0` (Stable).
- **Goal:** Vendor-agnostic cybersecurity taxonomy and standardized schema for security telemetry.

## 2. Relevant Event Classes for Perimeter Devices

### Network Category (`category_uid: 4`)
- **`network_activity`** (`class_uid: 4001`) — Connection open, close, traffic flow. The primary class for stateful firewalls.
- **`http_activity`** (`class_uid: 4002`) — Web requests/responses (WAF, Proxies).
- **`dns_activity`** (`class_uid: 4003`) — DNS queries/answers.
- **`tunnel_activity`** (`class_uid: 4014`) — VPN, IPsec, SSL VPN setup and teardown.

### IAM Category (`category_uid: 3`)
- **`authentication`** (`class_uid: 3002`) — VPN user logins, administrative logins.

### Findings Category (`category_uid: 2`)
- **`detection_finding`** (`class_uid: 2004`) — IDS/IPS alerts and security detections.

## 3. Network Activity (`4001`) Core Structure

An OCSF event is a JSON object. For a firewall log mapped to `4001`, the schema requires/recommends these specific fields:

### Required Fields
- `activity_id` (int): `1` (Open), `2` (Close), `3` (Reset), `4` (Fail), `5` (Refuse), `6` (Traffic).
- `category_uid` (int): `4`
- `class_uid` (int): `4001`
- `type_uid` (int): e.g., `400102` (Network Activity: Close)
- `time` (timestamp): Milliseconds since UTC epoch.
- `severity_id` (int): `1` (Info), `2` (Low), `3` (Medium), `4` (High), `5` (Critical).
- `metadata` (object): Contains schema `version` and `product` info.

### Primary Recommended Fields (The Payload)
- `src_endpoint` (object):
  - `.ip` (string: IP address)
  - `.port` (int)
  - `.interface_name` (string)
- `dst_endpoint` (object): Same structure as src_endpoint.
- `connection_info` (object):
  - `.direction_id` (int): `1` (Inbound), `2` (Outbound), `3` (Lateral).
  - `.protocol_num` (int): IANA protocol (e.g., `6` for TCP, `17` for UDP).
- `traffic` (object):
  - `.bytes`, `.bytes_in`, `.bytes_out` (long)
  - `.packets`, `.packets_in`, `.packets_out` (long)

### Security Control Profile (For Allow/Deny Actions)
When a firewall allows or blocks traffic, the `security_control` profile overlays these fields:
- `action_id` (int): `1` (Allowed), `2` (Denied), `3` (Observed).
- `disposition_id` (int): `1` (Allowed), `2` (Blocked), `6` (Dropped), `21` (Reset).
- `firewall_rule` (object): `.uid`, `.name`
