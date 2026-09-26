export type DashboardView = 'pipeline' | 'analytics' | 'tactical' | 'all';

export type DashboardTheme = 'dark' | 'light' | 'cyber';

export interface KPIMetric {
  id: string;
  label: string;
  value: string | number;
  change: number;
  isPositiveGood?: boolean;
  history: number[];
  subtitle?: string;
  accentColor?: string;
}

export interface OCSFEvent {
  id: string;
  time: number;
  formattedTime: string;
  category_uid: number;
  class_uid: number;
  activity_id: 1 | 2 | 3;
  activity_label: 'Allow' | 'Deny' | 'Reset';
  src_ip: string;
  src_country: string;
  src_country_code: string;
  dst_ip: string;
  dst_port: number;
  protocol: 'TCP' | 'UDP' | 'ICMP' | 'TLS';
  vendor: 'Palo Alto PA-5250' | 'Fortinet FortiGate' | 'Cisco ASA 5585' | 'AWS VPC Flow' | 'AWS VPC Flow Logs' | 'Suricata IDS';
  severity: 'Critical' | 'High' | 'Medium' | 'Low' | 'Info';
  severity_id: number;
  threat_type?: string;
  bytes: number;
  history_hash: string;
  vault_uri: string;
  merkle_leaf_index: number;
  vrl_rule: string;
  drain_template?: string;
}

export interface SankeyNode {
  id: string;
  label: string;
  col: number;
  value: number;
  color: string;
  sublabel?: string;
  flag?: string;
}

export interface SankeyLink {
  source: string;
  target: string;
  value: number;
  color: string;
}

export interface GeoThreatPoint {
  id: string;
  country: string;
  countryCode: string;
  x: number;
  y: number;
  threatCount: number;
  severity: 'Critical' | 'High' | 'Medium' | 'Low';
  recentTarget: string;
}

export interface ThroughputDataPoint {
  timestamp: string;
  ingestEps: number;
  normalizedEps: number;
  dlqEps: number;
  latencyUs: number;
}
