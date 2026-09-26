import React, { useState, useEffect } from 'react';
import {
  Zap,
  Activity,
  ShieldCheck,
  ShieldAlert,
  Binary,
  Cpu,
  Layers,
  Search,
  ExternalLink,
  ChevronRight,
  Filter,
  CheckCircle2,
  AlertTriangle,
  ArrowUpRight,
  ArrowDownRight,
  FileCheck2,
  Lock,
  Globe,
  Radio,
  Sparkles,
} from 'lucide-react';
import type { DashboardTheme, OCSFEvent, ThroughputDataPoint } from '../types';
import { SAMPLE_FORENSIC_EVENTS } from '../mock/data';
import { WorldThreatMap } from './WorldThreatMap';
import { MerkleAuditModal } from './MerkleAuditModal';

interface Props {
  theme: DashboardTheme;
  isLiveStreaming: boolean;
  onOpenGatekeeper?: () => void;
  onOpenDLQ?: () => void;
  onOpenVrl?: () => void;
  onOpenAccounting?: () => void;
}

const INITIAL_THROUGHPUT: ThroughputDataPoint[] = [
  { timestamp: '19:00', ingestEps: 285400, normalizedEps: 282100, dlqEps: 12, latencyUs: 8.1 },
  { timestamp: '19:01', ingestEps: 291200, normalizedEps: 288400, dlqEps: 15, latencyUs: 8.3 },
  { timestamp: '19:02', ingestEps: 288700, normalizedEps: 285200, dlqEps: 14, latencyUs: 8.0 },
  { timestamp: '19:03', ingestEps: 304200, normalizedEps: 301100, dlqEps: 22, latencyUs: 8.9 },
  { timestamp: '19:04', ingestEps: 318900, normalizedEps: 314500, dlqEps: 28, latencyUs: 9.4 },
  { timestamp: '19:05', ingestEps: 312480, normalizedEps: 308920, dlqEps: 18, latencyUs: 8.42 },
];

export const CommandCenterView: React.FC<Props> = ({
  theme,
  isLiveStreaming,
  onOpenGatekeeper,
  onOpenDLQ,
  onOpenVrl,
  onOpenAccounting,
}) => {
  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  const [events, setEvents] = useState<OCSFEvent[]>(SAMPLE_FORENSIC_EVENTS);
  const [throughputHistory, setThroughputHistory] = useState<ThroughputDataPoint[]>(INITIAL_THROUGHPUT);
  const [selectedEventForAudit, setSelectedEventForAudit] = useState<OCSFEvent | null>(null);
  const [searchFilter, setSearchFilter] = useState('');
  const [selectedVendorFilter, setSelectedVendorFilter] = useState<string>('All');
  const [hoveredChartPoint, setHoveredChartPoint] = useState<ThroughputDataPoint | null>(null);

  // Live polling / stream simulation
  useEffect(() => {
    if (!isLiveStreaming) return;

    const interval = setInterval(() => {
      // 1. Generate live event
      const vendors = ['Palo Alto PA-5250', 'Fortinet FortiGate', 'Cisco ASA 5585', 'AWS VPC Flow'] as const;
      const protocols = ['TCP', 'UDP', 'TLS'] as const;
      const severities = ['Low', 'Medium', 'High', 'Critical'] as const;
      const actions = ['Allow', 'Deny', 'Reset'] as const;

      const v = vendors[Math.floor(Math.random() * vendors.length)];
      const proto = protocols[Math.floor(Math.random() * protocols.length)];
      const sev = severities[Math.floor(Math.random() * severities.length)];
      const act = actions[Math.floor(Math.random() * actions.length)];

      const now = Date.now();
      const newEvt: OCSFEvent = {
        id: `evt-${now}-${Math.floor(Math.random() * 1000)}`,
        time: now,
        formattedTime: new Date(now).toISOString().substring(11, 23),
        category_uid: 4,
        class_uid: 4001,
        activity_id: act === 'Allow' ? 1 : act === 'Deny' ? 2 : 3,
        activity_label: act,
        src_ip: `198.51.100.${Math.floor(Math.random() * 254) + 1}`,
        src_country: 'China',
        src_country_code: 'CN',
        dst_ip: '10.0.5.50',
        dst_port: Math.random() > 0.5 ? 443 : 80,
        protocol: proto,
        vendor: v,
        severity: sev,
        severity_id: sev === 'Critical' ? 4 : sev === 'High' ? 3 : 2,
        bytes: Math.floor(Math.random() * 8000) + 200,
        provenance_hash: `b3_${Math.random().toString(16).substring(2, 10)}${Math.random().toString(16).substring(2, 10)}`,
        vault_uri: `s3://prism-cold-vault/2026/09/26/chunk_${Math.floor(now / 10000)}.parquet`,
        merkle_leaf_index: Math.floor(Math.random() * 50000),
        vrl_rule: `${v.toLowerCase().replace(/[\s-]/g, '_')}_ocsf4001.vrl`,
      };

      setEvents((prev) => [newEvt, ...prev.slice(0, 49)]);

      // 2. Append throughput point
      setThroughputHistory((prev) => {
        const baseEps = 285000 + Math.floor(Math.random() * 45000);
        const normEps = Math.floor(baseEps * (0.985 + Math.random() * 0.012));
        const dlqEps = Math.floor(Math.random() * 42);
        const latency = 7.5 + Math.random() * 2.8;

        const timeStr = new Date(now).toISOString().substring(14, 19);
        const newPt: ThroughputDataPoint = {
          timestamp: timeStr,
          ingestEps: baseEps,
          normalizedEps: normEps,
          dlqEps,
          latencyUs: parseFloat(latency.toFixed(2)),
        };
        return [...prev.slice(1), newPt];
      });
    }, 1800);

    return () => clearInterval(interval);
  }, [isLiveStreaming]);

  const latestPt = throughputHistory[throughputHistory.length - 1] || {
    ingestEps: 312480,
    normalizedEps: 308920,
    dlqEps: 18,
    latencyUs: 8.42,
  };

  const filteredEvents = events.filter((e) => {
    const matchesVendor = selectedVendorFilter === 'All' || e.vendor.toLowerCase().includes(selectedVendorFilter.toLowerCase());
    const matchesSearch =
      searchFilter === '' ||
      e.src_ip.includes(searchFilter) ||
      e.dst_ip.includes(searchFilter) ||
      e.provenance_hash.includes(searchFilter) ||
      e.vendor.toLowerCase().includes(searchFilter.toLowerCase());
    return matchesVendor && matchesSearch;
  });

  return (
    <div className="space-y-6">
      {/* Merkle Audit Modal */}
      <MerkleAuditModal
        event={selectedEventForAudit}
        onClose={() => setSelectedEventForAudit(null)}
      />

      {/* Top Telemetry KPI Ribbon (Sparklines + Real-Time Gauges) */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
        {/* KPI 1: Ingest EPS */}
        <div
          className={`p-3.5 rounded-2xl border transition-all duration-300 shadow-lg ${
            isDark ? 'bg-[#121520]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
              Throughput
            </span>
            <span className="w-2 h-2 rounded-full bg-cyan-400 animate-ping" />
          </div>
          <div className="text-xl font-extrabold tracking-tight mt-1 text-cyan-400">
            {(latestPt.ingestEps / 1000).toFixed(1)}k <span className="text-xs font-normal text-slate-400">EPS</span>
          </div>
          <div className="flex items-center gap-1 text-[10px] text-emerald-400 font-semibold mt-1">
            <ArrowUpRight className="w-3 h-3" />
            <span>+14.2% surge</span>
            <span className="text-slate-500 font-mono ml-auto">0 Drops</span>
          </div>
        </div>

        {/* KPI 2: Parse Latency */}
        <div
          className={`p-3.5 rounded-2xl border transition-all duration-300 shadow-lg ${
            isDark ? 'bg-[#121520]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
              Parse SLA (p99)
            </span>
            <Activity className="w-3.5 h-3.5 text-indigo-400" />
          </div>
          <div className="text-xl font-extrabold tracking-tight mt-1 text-indigo-400">
            {latestPt.latencyUs} <span className="text-xs font-normal text-slate-400">µs</span>
          </div>
          <div className="text-[10px] text-slate-400 font-mono mt-1">
            Tokio zero-alloc &lt; 15µs
          </div>
        </div>

        {/* KPI 3: Buffer Utilization */}
        <div
          className={`p-3.5 rounded-2xl border transition-all duration-300 shadow-lg ${
            isDark ? 'bg-[#121520]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
              Ring Buffer
            </span>
            <Zap className="w-3.5 h-3.5 text-amber-400" />
          </div>
          <div className="text-xl font-extrabold tracking-tight mt-1 text-amber-400">
            42% <span className="text-xs font-normal text-slate-400">Load</span>
          </div>
          <div className="w-full bg-slate-800 rounded-full h-1.5 mt-2 overflow-hidden">
            <div className="bg-amber-400 h-full rounded-full transition-all duration-500" style={{ width: '42%' }} />
          </div>
        </div>

        {/* KPI 4: Section 65B Witness Quorum */}
        <div
          className={`p-3.5 rounded-2xl border transition-all duration-300 shadow-lg ${
            isDark ? 'bg-[#121520]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
              Sec 65B Quorum
            </span>
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
          </div>
          <div className="text-xl font-extrabold tracking-tight mt-1 text-emerald-400">
            2 / 3 <span className="text-xs font-normal text-slate-400">Signed</span>
          </div>
          <div className="text-[10px] text-emerald-400 font-mono mt-1">
            NTRO + CERT-In Cosigned
          </div>
        </div>

        {/* KPI 5: Byte Closure Ratio */}
        <div
          className={`p-3.5 rounded-2xl border transition-all duration-300 shadow-lg ${
            isDark ? 'bg-[#121520]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
              Byte Closure
            </span>
            <Binary className="w-3.5 h-3.5 text-cyan-400" />
          </div>
          <div className="text-xl font-extrabold tracking-tight mt-1 text-cyan-300">
            94.8% <span className="text-xs font-normal text-slate-400">Ratio</span>
          </div>
          <div className="text-[10px] text-cyan-400 font-mono mt-1">
            PASS (&gt;90% court threshold)
          </div>
        </div>

        {/* KPI 6: Merkle Roots */}
        <div
          className={`p-3.5 rounded-2xl border transition-all duration-300 shadow-lg ${
            isDark ? 'bg-[#121520]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
        >
          <div className="flex items-center justify-between">
            <span className="text-[11px] font-mono font-bold uppercase tracking-wider text-slate-400">
              Merkle Roots
            </span>
            <FileCheck2 className="w-3.5 h-3.5 text-purple-400" />
          </div>
          <div className="text-xl font-extrabold tracking-tight mt-1 text-purple-300">
            492 <span className="text-xs font-normal text-slate-400">Roots</span>
          </div>
          <div className="text-[10px] text-slate-400 font-mono mt-1">
            BLAKE3 Cold Parquet
          </div>
        </div>
      </div>

      {/* Primary Cockpit Grid: Tactical Threat Radar (58%) + Real-Time Spline Engine (42%) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Global Tactical Threat Radar & Ingest Hubs (7 cols) */}
        <div className="lg:col-span-7 space-y-3">
          <div className="flex items-center justify-between px-1">
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse" />
              <h2 className="text-sm font-bold tracking-tight uppercase font-mono text-slate-200">
                Tactical Attack Vector Radar & Sovereign Ingest Hubs
              </h2>
            </div>
            <span className="text-[11px] font-mono text-amber-400 bg-amber-950/40 px-2 py-0.5 rounded border border-amber-800/80">
              99.94% MITIGATED
            </span>
          </div>
          <WorldThreatMap theme={theme} />
        </div>

        {/* Right: Real-Time Spline Throughput & Latency Spectrum (5 cols) */}
        <div
          className={`lg:col-span-5 rounded-2xl border p-5 flex flex-col justify-between shadow-2xl transition-all duration-300 ${
            isDark ? 'bg-[#111420]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div>
            <div className="flex items-center justify-between border-b border-slate-800 pb-3">
              <div>
                <h3 className="text-sm font-bold tracking-tight">Real-Time Ingestion & Egress Spline</h3>
                <p className="text-xs text-slate-400">
                  Ingress (Cyan) vs. Normalized OCSF (Purple) vs. DLQ Spikes (Rose)
                </p>
              </div>
              <span className="text-[11px] font-mono text-cyan-400 bg-cyan-950/60 px-2 py-0.5 rounded border border-cyan-800">
                {(latestPt.ingestEps / 1000).toFixed(0)}k EPS
              </span>
            </div>

            {/* SVG Spline Chart */}
            <div className="relative pt-4 pb-2">
              <svg viewBox="0 0 400 160" className="w-full h-44 overflow-visible select-none">
                <defs>
                  <linearGradient id="cmdIngestGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#06b6d4" stopOpacity="0.45" />
                    <stop offset="100%" stopColor="#06b6d4" stopOpacity="0.0" />
                  </linearGradient>
                  <linearGradient id="cmdNormGrad" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stopColor="#a855f7" stopOpacity="0.35" />
                    <stop offset="100%" stopColor="#a855f7" stopOpacity="0.0" />
                  </linearGradient>
                </defs>

                {/* Subtle Grid Lines */}
                <line x1="0" y1="40" x2="400" y2="40" stroke="#334155" strokeDasharray="3,3" opacity="0.3" />
                <line x1="0" y1="80" x2="400" y2="80" stroke="#334155" strokeDasharray="3,3" opacity="0.3" />
                <line x1="0" y1="120" x2="400" y2="120" stroke="#334155" strokeDasharray="3,3" opacity="0.3" />

                {/* Draw Smooth Curves */}
                {(() => {
                  const pts = throughputHistory.slice(-16);
                  if (pts.length < 2) return null;
                  const maxEps = 350000;
                  const minEps = 240000;
                  const stepX = 400 / (pts.length - 1);

                  const ingestCoords = pts.map((p, i) => {
                    const x = i * stepX;
                    const y = 140 - ((p.ingestEps - minEps) / (maxEps - minEps)) * 120;
                    return { x, y: Math.max(10, Math.min(150, y)) };
                  });

                  const normCoords = pts.map((p, i) => {
                    const x = i * stepX;
                    const y = 140 - ((p.normalizedEps - minEps) / (maxEps - minEps)) * 120;
                    return { x, y: Math.max(10, Math.min(150, y)) };
                  });

                  // Cubic bezier string generator
                  const makePath = (coords: { x: number; y: number }[]) => {
                    let d = `M ${coords[0].x},${coords[0].y}`;
                    for (let i = 0; i < coords.length - 1; i++) {
                      const cpX = (coords[i].x + coords[i + 1].x) / 2;
                      d += ` C ${cpX},${coords[i].y} ${cpX},${coords[i + 1].y} ${coords[i + 1].x},${coords[i + 1].y}`;
                    }
                    return d;
                  };

                  const ingestPath = makePath(ingestCoords);
                  const normPath = makePath(normCoords);

                  const ingestArea = `${ingestPath} L 400,160 L 0,160 Z`;
                  const normArea = `${normPath} L 400,160 L 0,160 Z`;

                  return (
                    <>
                      <path d={ingestArea} fill="url(#cmdIngestGrad)" />
                      <path d={normArea} fill="url(#cmdNormGrad)" />
                      <path d={ingestPath} fill="none" stroke="#06b6d4" strokeWidth="2.5" />
                      <path d={normPath} fill="none" stroke="#a855f7" strokeWidth="2" strokeDasharray="4,2" />

                      {/* Current pulse dot */}
                      <circle
                        cx={ingestCoords[ingestCoords.length - 1].x}
                        cy={ingestCoords[ingestCoords.length - 1].y}
                        r="4"
                        fill="#06b6d4"
                        className="animate-ping"
                      />
                      <circle
                        cx={ingestCoords[ingestCoords.length - 1].x}
                        cy={ingestCoords[ingestCoords.length - 1].y}
                        r="3"
                        fill="#ffffff"
                      />
                    </>
                  );
                })()}
              </svg>
            </div>
          </div>

          {/* Sub-Millisecond Latency Distribution Bars */}
          <div className="pt-3 border-t border-slate-800 space-y-2">
            <div className="flex items-center justify-between text-xs">
              <span className="font-mono text-slate-400">Normalization Latency Profile</span>
              <span className="font-mono text-emerald-400 font-bold">p99 = 14.1 µs</span>
            </div>

            <div className="grid grid-cols-4 gap-2 text-center text-[10px] font-mono">
              <div className="p-2 rounded-xl bg-slate-900/80 border border-emerald-500/20">
                <div className="text-emerald-400 font-bold">68.2%</div>
                <div className="text-slate-400">&lt; 5 µs</div>
              </div>
              <div className="p-2 rounded-xl bg-slate-900/80 border border-cyan-500/20">
                <div className="text-cyan-400 font-bold">26.1%</div>
                <div className="text-slate-400">5 - 15 µs</div>
              </div>
              <div className="p-2 rounded-xl bg-slate-900/80 border border-amber-500/20">
                <div className="text-amber-400 font-bold">5.3%</div>
                <div className="text-slate-400">15 - 50 µs</div>
              </div>
              <div className="p-2 rounded-xl bg-slate-900/80 border border-rose-500/20">
                <div className="text-rose-400 font-bold">0.4%</div>
                <div className="text-slate-400">&gt; 50 µs</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Secondary Grid: Pipeline Flow Topology (Left) + Byte Accounting & Quorum (Right) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Dynamic Pipeline Topology (6 cols) */}
        <div
          className={`lg:col-span-6 rounded-2xl border p-5 shadow-2xl transition-all duration-300 space-y-4 ${
            isDark ? 'bg-[#111420]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between border-b border-slate-800 pb-3">
            <div className="flex items-center gap-2">
              <Zap className="w-4 h-4 text-cyan-400" />
              <h3 className="text-sm font-bold">Zero-Alloc Ingestion & VRL Pipeline Topology</h3>
            </div>
            {onOpenVrl && (
              <button
                onClick={onOpenVrl}
                className="text-[11px] text-cyan-400 hover:text-cyan-300 font-mono flex items-center gap-1 transition"
              >
                <span>Open VRL Sandbox</span>
                <ChevronRight className="w-3 h-3" />
              </button>
            )}
          </div>

          {/* Interactive Pipeline Stages */}
          <div className="grid grid-cols-4 gap-2 text-xs">
            {/* Stage 1 */}
            <div className="p-3 rounded-xl bg-slate-900/80 border border-cyan-500/30 space-y-1">
              <span className="text-[10px] text-cyan-400 font-mono font-bold">STAGE 1</span>
              <div className="font-bold text-slate-200">UDP Ingest</div>
              <p className="text-[10px] text-slate-400 leading-tight">Zero-alloc Tokio workers</p>
              <div className="text-[10px] font-mono text-cyan-300 pt-1">312k EPS</div>
            </div>

            {/* Stage 2 */}
            <div className="p-3 rounded-xl bg-slate-900/80 border border-indigo-500/30 space-y-1">
              <span className="text-[10px] text-indigo-400 font-mono font-bold">STAGE 2</span>
              <div className="font-bold text-slate-200">VRL Remap</div>
              <p className="text-[10px] text-slate-400 leading-tight">Dynamic regex & parsing</p>
              <div className="text-[10px] font-mono text-indigo-300 pt-1">8.42 µs avg</div>
            </div>

            {/* Stage 3 */}
            <div className="p-3 rounded-xl bg-slate-900/80 border border-emerald-500/30 space-y-1">
              <span className="text-[10px] text-emerald-400 font-mono font-bold">STAGE 3</span>
              <div className="font-bold text-slate-200">Byte Audit</div>
              <p className="text-[10px] text-slate-400 leading-tight">FIELD / LITERAL / RESIDUE</p>
              <div className="text-[10px] font-mono text-emerald-300 pt-1">94.8% closure</div>
            </div>

            {/* Stage 4 */}
            <div className="p-3 rounded-xl bg-slate-900/80 border border-purple-500/30 space-y-1">
              <span className="text-[10px] text-purple-400 font-mono font-bold">STAGE 4</span>
              <div className="font-bold text-slate-200">OCSF Egress</div>
              <p className="text-[10px] text-slate-400 leading-tight">Classes 4001, 2004, 3001</p>
              <div className="text-[10px] font-mono text-purple-300 pt-1">99.8% validity</div>
            </div>
          </div>

          {/* Quick Vendor Distribution */}
          <div className="p-3 rounded-xl bg-slate-900/50 border border-slate-800 space-y-2">
            <span className="text-[11px] font-mono text-slate-400 uppercase tracking-wider">
              Ingress Traffic by Firewall Vendor
            </span>
            <div className="flex items-center gap-3 text-xs font-mono">
              <span className="flex items-center gap-1.5 text-slate-300">
                <span className="w-2.5 h-2.5 rounded-full bg-cyan-400" /> Palo Alto (38%)
              </span>
              <span className="flex items-center gap-1.5 text-slate-300">
                <span className="w-2.5 h-2.5 rounded-full bg-indigo-400" /> Fortinet (31%)
              </span>
              <span className="flex items-center gap-1.5 text-slate-300">
                <span className="w-2.5 h-2.5 rounded-full bg-emerald-400" /> Cisco ASA (21%)
              </span>
              <span className="flex items-center gap-1.5 text-slate-300">
                <span className="w-2.5 h-2.5 rounded-full bg-amber-400" /> AWS VPC (10%)
              </span>
            </div>
          </div>
        </div>

        {/* Right: Byte Accounting & Section 65B Quorum (6 cols) */}
        <div
          className={`lg:col-span-6 rounded-2xl border p-5 shadow-2xl transition-all duration-300 space-y-4 ${
            isDark ? 'bg-[#111420]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between border-b border-slate-800 pb-3">
            <div className="flex items-center gap-2">
              <Binary className="w-4 h-4 text-emerald-400" />
              <h3 className="text-sm font-bold">Byte Accounting & Section 65B Witness Quorum</h3>
            </div>
            {onOpenAccounting && (
              <button
                onClick={onOpenAccounting}
                className="text-[11px] text-emerald-400 hover:text-emerald-300 font-mono flex items-center gap-1 transition"
              >
                <span>Full Accounting Inspector</span>
                <ChevronRight className="w-3 h-3" />
              </button>
            )}
          </div>

          {/* Interactive Character Byte Stream Sample */}
          <div className="p-3 rounded-xl bg-black/80 border border-slate-800 space-y-2">
            <div className="flex items-center justify-between text-[11px] font-mono text-slate-400">
              <span>Raw Stream Byte Classification (§3.1)</span>
              <span className="text-emerald-400 font-bold">Closure: 94.8% PASS</span>
            </div>
            <div className="text-xs font-mono leading-relaxed select-none">
              <span className="bg-blue-500/20 text-blue-300 px-1 py-0.5 rounded">srcip=</span>
              <span className="bg-emerald-500/30 text-emerald-300 font-bold px-1 py-0.5 rounded border-b border-emerald-500">10.10.20.45</span>
              <span className="bg-blue-500/20 text-blue-300 px-1 py-0.5 rounded ml-1">dstip=</span>
              <span className="bg-emerald-500/30 text-emerald-300 font-bold px-1 py-0.5 rounded border-b border-emerald-500">203.0.113.25</span>
              <span className="bg-blue-500/20 text-blue-300 px-1 py-0.5 rounded ml-1">proto=</span>
              <span className="bg-emerald-500/30 text-emerald-300 font-bold px-1 py-0.5 rounded border-b border-emerald-500">6</span>
              <span className="bg-blue-500/20 text-blue-300 px-1 py-0.5 rounded ml-1">action=</span>
              <span className="bg-emerald-500/30 text-emerald-300 font-bold px-1 py-0.5 rounded border-b border-emerald-500">accept</span>
              <span className="bg-rose-500/20 text-rose-300 px-1 py-0.5 rounded ml-1">vd=root</span>
            </div>
          </div>

          {/* 2-of-3 Witness Signatures */}
          <div className="grid grid-cols-3 gap-2 text-[10px] font-mono">
            <div className="p-2.5 rounded-xl bg-slate-900/80 border border-emerald-500/40 space-y-1">
              <div className="flex items-center justify-between text-emerald-400 font-bold">
                <span>NTRO GATEWAY</span>
                <CheckCircle2 className="w-3 h-3" />
              </div>
              <p className="text-slate-400">ed25519:7a4f91b...</p>
              <div className="text-emerald-300 font-semibold">SIGNED</div>
            </div>

            <div className="p-2.5 rounded-xl bg-slate-900/80 border border-emerald-500/40 space-y-1">
              <div className="flex items-center justify-between text-emerald-400 font-bold">
                <span>CERT-In NODE</span>
                <CheckCircle2 className="w-3 h-3" />
              </div>
              <p className="text-slate-400">ed25519:92bfe10...</p>
              <div className="text-emerald-300 font-semibold">SIGNED</div>
            </div>

            <div className="p-2.5 rounded-xl bg-slate-900/80 border border-slate-700/60 space-y-1">
              <div className="flex items-center justify-between text-slate-400 font-bold">
                <span>JUDICIARY VAULT</span>
                <Lock className="w-3 h-3" />
              </div>
              <p className="text-slate-500">ed25519:3819dc7...</p>
              <div className="text-slate-400">STANDBY</div>
            </div>
          </div>
        </div>
      </div>

      {/* Bottom Master: Section 65B Cryptographic Ledger & OCSF Event Feed */}
      <div
        className={`rounded-2xl border p-5 shadow-2xl transition-all duration-300 space-y-4 ${
          isDark ? 'bg-[#111420]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
        }`}
      >
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-800 pb-3">
          <div>
            <div className="flex items-center gap-2">
              <ShieldCheck className="w-4 h-4 text-emerald-400" />
              <h3 className="text-sm font-bold text-slate-200">
                Section 65B Forensic Event Ledger & OCSF Egress Stream
              </h3>
            </div>
            <p className="text-xs text-slate-400">
              Every row commits a cryptographic BLAKE3 leaf into cold Parquet storage for Indian Evidence Act admissibility
            </p>
          </div>

          {/* Search & Filters */}
          <div className="flex flex-wrap items-center gap-2">
            {/* Vendor Filter */}
            <div className="flex items-center gap-1 p-0.5 rounded-xl bg-slate-900/80 border border-slate-800 text-xs">
              {['All', 'Palo Alto', 'Fortinet', 'Cisco'].map((v) => (
                <button
                  key={v}
                  onClick={() => setSelectedVendorFilter(v)}
                  className={`px-2 py-1 rounded-lg transition font-medium ${
                    selectedVendorFilter === v
                      ? 'bg-cyan-600 text-white font-bold shadow'
                      : 'text-slate-400 hover:text-white'
                  }`}
                >
                  {v}
                </button>
              ))}
            </div>

            {/* Search Input */}
            <div className="relative">
              <Search className="w-3.5 h-3.5 text-slate-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
              <input
                type="text"
                value={searchFilter}
                onChange={(e) => setSearchFilter(e.target.value)}
                placeholder="Filter IP, Hash, CVE..."
                className="pl-8 pr-3 py-1 rounded-xl text-xs bg-slate-900/80 border border-slate-800 text-slate-200 placeholder:text-slate-500 focus:outline-none focus:border-cyan-500 transition w-40"
              />
            </div>
          </div>
        </div>

        {/* High-Density Forensic Table */}
        <div className="overflow-x-auto rounded-xl border border-slate-800">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-slate-900/90 text-slate-400 border-b border-slate-800 uppercase text-[10px] tracking-wider select-none">
              <tr>
                <th className="py-2.5 px-3">Timestamp</th>
                <th className="py-2.5 px-3">Vendor / Sensor</th>
                <th className="py-2.5 px-3">OCSF Class</th>
                <th className="py-2.5 px-3">Action</th>
                <th className="py-2.5 px-3">Source IP</th>
                <th className="py-2.5 px-3">Destination IP</th>
                <th className="py-2.5 px-3">BLAKE3 Hash</th>
                <th className="py-2.5 px-3 text-right">Sec 65B Audit</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 bg-black/40">
              {filteredEvents.slice(0, 10).map((evt) => {
                const isDeny = evt.activity_label === 'Deny';
                const isAllow = evt.activity_label === 'Allow';

                return (
                  <tr key={evt.id} className="hover:bg-slate-800/40 transition">
                    <td className="py-2 px-3 text-slate-400 whitespace-nowrap">{evt.formattedTime}</td>
                    <td className="py-2 px-3 text-slate-300 font-bold whitespace-nowrap">
                      {evt.vendor.replace(' PA-5250', '').replace(' 5585', '')}
                    </td>
                    <td className="py-2 px-3 text-cyan-400 font-bold">4001 Network</td>
                    <td className="py-2 px-3">
                      <span
                        className={`px-2 py-0.5 rounded text-[10px] font-bold ${
                          isDeny
                            ? 'bg-rose-500/20 text-rose-400 border border-rose-500/30'
                            : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                        }`}
                      >
                        {evt.activity_label}
                      </span>
                    </td>
                    <td className="py-2 px-3 text-slate-300 whitespace-nowrap">
                      {evt.src_ip} <span className="text-[10px] text-slate-500">({evt.src_country_code})</span>
                    </td>
                    <td className="py-2 px-3 text-slate-300 whitespace-nowrap">
                      {evt.dst_ip}:{evt.dst_port}
                    </td>
                    <td className="py-2 px-3 text-cyan-300/80 font-mono text-[11px] truncate max-w-[140px]">
                      {evt.provenance_hash}
                    </td>
                    <td className="py-2 px-3 text-right">
                      <button
                        onClick={() => setSelectedEventForAudit(evt)}
                        className="px-2 py-1 rounded bg-emerald-950/60 text-emerald-400 border border-emerald-500/40 hover:bg-emerald-900/60 text-[10px] font-bold transition inline-flex items-center gap-1"
                      >
                        <ShieldCheck className="w-3 h-3" />
                        <span>Verify Proof</span>
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
