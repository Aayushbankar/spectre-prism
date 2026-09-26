import React, { useState } from 'react';
import { WorldThreatMap } from './WorldThreatMap';
import { SAMPLE_FORENSIC_EVENTS } from '../mock/data';
import type { OCSFEvent } from '../types';
import { MerkleAuditModal } from './MerkleAuditModal';
import { Shield, Terminal, Activity, FileText, Lock, Radio, AlertOctagon, CheckCircle2, XCircle } from 'lucide-react';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  isLiveStreaming: boolean;
}

export const TacticalCommandView: React.FC<Props> = ({ theme, isLiveStreaming }) => {
  const [selectedAuditEvt, setSelectedAuditEvt] = useState<OCSFEvent | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<string>('All');

  const isCyber = theme === 'cyber';

  return (
    <div className="space-y-6">
      {/* Audit Modal for Section 65B Compliance */}
      <MerkleAuditModal event={selectedAuditEvt} onClose={() => setSelectedAuditEvt(null)} />

      {/* Top Section: Ingestion Frequency Histogram + World Threat Vector Radar */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-5">
        {/* 1. Multi-series Frequency Histogram (3 cols) */}
        <div className="lg:col-span-4 rounded-2xl border border-amber-500/25 bg-[#0f1118] p-4 shadow-xl flex flex-col justify-between">
          <div className="flex items-center justify-between border-b border-amber-500/20 pb-2 mb-2">
            <span className="text-xs font-mono font-bold tracking-wider text-amber-400 uppercase flex items-center gap-1.5">
              <Activity className="w-4 h-4 text-amber-500" /> Ingestion & Drop Frequency
            </span>
            <span className="text-[10px] font-mono text-slate-500">PEAK: 325k EPS</span>
          </div>

          {/* Histogram SVG */}
          <div className="h-44 w-full flex items-end">
            <svg viewBox="0 0 280 150" className="w-full h-full overflow-visible">
              {/* Bars */}
              {[
                { x: 10, h1: 40, h2: 25 },
                { x: 26, h1: 55, h2: 30 },
                { x: 42, h1: 75, h2: 45 },
                { x: 58, h1: 110, h2: 80 },
                { x: 74, h1: 95, h2: 60 },
                { x: 90, h1: 130, h2: 90 },
                { x: 106, h1: 140, h2: 115 },
                { x: 122, h1: 100, h2: 70 },
                { x: 138, h1: 125, h2: 85 },
                { x: 154, h1: 135, h2: 105 },
                { x: 170, h1: 90, h2: 50 },
                { x: 186, h1: 115, h2: 75 },
                { x: 202, h1: 70, h2: 40 },
                { x: 218, h1: 85, h2: 55 },
                { x: 234, h1: 60, h2: 35 },
                { x: 250, h1: 45, h2: 20 },
              ].map((bar, i) => (
                <g key={i}>
                  <rect
                    x={bar.x}
                    y={150 - bar.h1}
                    width="11"
                    height={bar.h1}
                    rx="2"
                    fill="#d97706"
                    opacity="0.85"
                  />
                  <rect
                    x={bar.x + 3}
                    y={150 - bar.h2}
                    width="6"
                    height={bar.h2}
                    rx="1"
                    fill="#f43f5e"
                    opacity="0.9"
                  />
                </g>
              ))}

              {/* Trajectory line */}
              <polyline
                fill="none"
                stroke="#fbbf24"
                strokeWidth="2.5"
                points="10,120 42,90 74,60 106,30 138,40 170,70 202,95 234,110 260,125"
              />
            </svg>
          </div>

          <div className="flex items-center justify-between text-[10px] font-mono text-slate-400 border-t border-amber-500/10 pt-2 mt-2">
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded bg-amber-500" /> Ingest Throughput
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded bg-rose-500" /> ACL Denied
            </span>
          </div>
        </div>

        {/* 2. Global Threat Vector Map (5 cols) */}
        <div className="lg:col-span-5">
          <WorldThreatMap theme={theme} />
        </div>

        {/* 3. Dual-Threshold Telemetry Area Chart (3 cols) */}
        <div className="lg:col-span-3 rounded-2xl border border-amber-500/25 bg-[#0f1118] p-4 shadow-xl flex flex-col justify-between">
          <div className="flex items-center justify-between border-b border-amber-500/20 pb-2 mb-2">
            <span className="text-xs font-mono font-bold tracking-wider text-amber-400 uppercase">
              THREAT INTENSITY
            </span>
            <span className="text-[10px] font-mono text-emerald-400">98.4% SAFE</span>
          </div>

          <div className="space-y-3 font-mono text-xs">
            <div className="p-2.5 rounded-lg bg-black/60 border border-amber-500/20 flex items-center justify-between">
              <span className="text-slate-400">DDoS Volumetric:</span>
              <span className="font-bold text-amber-300">18.4 Gbps</span>
            </div>
            <div className="p-2.5 rounded-lg bg-black/60 border border-amber-500/20 flex items-center justify-between">
              <span className="text-slate-400">Zero-Day Signals:</span>
              <span className="font-bold text-rose-400">3 Flagged (HitL)</span>
            </div>
            <div className="p-2.5 rounded-lg bg-black/60 border border-amber-500/20 flex items-center justify-between">
              <span className="text-slate-400">ZSTD Compression:</span>
              <span className="font-bold text-cyan-400">8.4x Ratio</span>
            </div>
            <div className="p-2.5 rounded-lg bg-black/60 border border-amber-500/20 flex items-center justify-between">
              <span className="text-slate-400">Section 65B Audit:</span>
              <span className="font-bold text-emerald-400">PASS (32 Batches)</span>
            </div>
          </div>

          {/* Mini trendline */}
          <div className="h-16 w-full pt-2">
            <svg viewBox="0 0 200 50" className="w-full h-full overflow-visible">
              <path
                d="M 0 45 Q 30 10, 60 30 T 120 15 T 180 35 L 200 20"
                fill="none"
                stroke="#f59e0b"
                strokeWidth="2.5"
              />
              <path
                d="M 0 40 Q 40 30, 80 40 T 140 35 T 200 45"
                fill="none"
                stroke="#f43f5e"
                strokeWidth="2"
                strokeDasharray="3 3"
              />
            </svg>
          </div>
        </div>
      </div>

      {/* Middle Section: Threat Category Pie + Network Node Topology + Anomaly Velocity */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-12 gap-5">
        {/* Threat Category Donut (4 cols) */}
        <div className="lg:col-span-4 rounded-2xl border border-amber-500/25 bg-[#0f1118] p-5 shadow-xl">
          <div className="flex items-center justify-between border-b border-amber-500/20 pb-2 mb-4">
            <span className="text-xs font-mono font-bold text-amber-400 uppercase">
              THREAT SPECTRUM BREAKDOWN
            </span>
            <span className="text-[10px] font-mono text-slate-400">AI TRIAGE CLASSIFIED</span>
          </div>

          <div className="flex items-center justify-around">
            {/* Donut */}
            <div className="relative w-28 h-28 flex items-center justify-center">
              <svg viewBox="0 0 100 100" className="w-full h-full -rotate-90">
                {/* 40% Crimson: Brute Force */}
                <circle cx="50" cy="50" r="38" fill="none" stroke="#ef4444" strokeWidth="14" strokeDasharray="95 239" />
                {/* 25% Amber: Port Sweep */}
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#f59e0b"
                  strokeWidth="14"
                  strokeDasharray="60 239"
                  strokeDashoffset="-95"
                />
                {/* 20% Purple: DNS Amplification */}
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#a855f7"
                  strokeWidth="14"
                  strokeDasharray="48 239"
                  strokeDashoffset="-155"
                />
                {/* 15% Cyan: C2 Beaconing */}
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#06b6d4"
                  strokeWidth="14"
                  strokeDasharray="36 239"
                  strokeDashoffset="-203"
                />
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center font-mono">
                <span className="text-[9px] uppercase text-slate-400">Threats</span>
                <span className="text-sm font-extrabold text-amber-400">4,280</span>
              </div>
            </div>

            {/* Legend */}
            <div className="space-y-1.5 text-xs font-mono">
              <div className="flex items-center gap-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-[#ef4444]" />
                <span className="text-slate-400">Brute Force (40%)</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-[#f59e0b]" />
                <span className="text-slate-400">Port Sweep (25%)</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-[#a855f7]" />
                <span className="text-slate-400">DNS Amp (20%)</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-[#06b6d4]" />
                <span className="text-slate-400">C2 Beacon (15%)</span>
              </div>
            </div>
          </div>
        </div>

        {/* Network Node Topology with Orbital Core (4 cols) */}
        <div className="lg:col-span-4 rounded-2xl border border-amber-500/25 bg-[#0f1118] p-5 shadow-xl flex flex-col justify-between">
          <div className="flex items-center justify-between border-b border-amber-500/20 pb-2 mb-2">
            <span className="text-xs font-mono font-bold text-amber-400 uppercase">
              NODE TOPOLOGY & ZERO-ALLOC PIPELINE
            </span>
            <span className="text-[10px] font-mono text-emerald-400">MESH SYNCED</span>
          </div>

          {/* SVG Orbital Topology */}
          <div className="h-36 w-full flex items-center justify-center">
            <svg viewBox="0 0 200 120" className="w-full h-full">
              {/* Radar Rings */}
              <circle cx="100" cy="60" r="45" fill="none" stroke="#f59e0b" strokeWidth="0.8" opacity="0.3" strokeDasharray="3 3" />
              <circle cx="100" cy="60" r="28" fill="none" stroke="#f59e0b" strokeWidth="1" opacity="0.5" />

              {/* Connecting lines */}
              <line x1="100" y1="60" x2="35" y2="40" stroke="#fbbf24" strokeWidth="1.2" opacity="0.6" />
              <line x1="100" y1="60" x2="165" y2="35" stroke="#fbbf24" strokeWidth="1.2" opacity="0.6" />
              <line x1="100" y1="60" x2="70" y2="100" stroke="#fbbf24" strokeWidth="1.2" opacity="0.6" />
              <line x1="100" y1="60" x2="145" y2="95" stroke="#fbbf24" strokeWidth="1.2" opacity="0.6" />

              {/* Satellite nodes */}
              <circle cx="35" cy="40" r="5" fill="#f43f5e" />
              <circle cx="165" cy="35" r="6" fill="#38bdf8" />
              <circle cx="70" cy="100" r="5" fill="#a855f7" />
              <circle cx="145" cy="95" r="5" fill="#10b981" />

              {/* Center Orbital Core */}
              <circle cx="100" cy="60" r="12" fill="#d97706" fillOpacity="0.4" stroke="#f59e0b" strokeWidth="2" />
              <circle cx="100" cy="60" r="4" fill="#fbbf24" />
            </svg>
          </div>

          <div className="flex justify-between text-[10px] font-mono text-slate-400 border-t border-amber-500/10 pt-2">
            <span>5 Active Engines</span>
            <span className="text-amber-300 font-bold">TOKIO MPSC CHANNELS</span>
          </div>
        </div>

        {/* Dual Cumulative Telemetry Time-Series (4 cols) */}
        <div className="lg:col-span-4 rounded-2xl border border-amber-500/25 bg-[#0f1118] p-5 shadow-xl flex flex-col justify-between">
          <div className="flex items-center justify-between border-b border-amber-500/20 pb-2 mb-2">
            <span className="text-xs font-mono font-bold text-amber-400 uppercase">
              ANOMALY RATE OVER TIME
            </span>
            <span className="text-[10px] font-mono text-rose-400">p99 &lt; 25µs</span>
          </div>

          {/* Cumulative Area Chart */}
          <div className="h-36 w-full flex items-end">
            <svg viewBox="0 0 240 100" className="w-full h-full overflow-visible">
              <defs>
                <linearGradient id="goldCyberGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#f59e0b" stopOpacity="0.5" />
                  <stop offset="100%" stopColor="#f59e0b" stopOpacity="0.0" />
                </linearGradient>
              </defs>
              <polygon
                points="0,95 20,80 40,85 70,60 100,65 130,40 160,50 190,25 220,35 240,15 240,100 0,100"
                fill="url(#goldCyberGrad)"
              />
              <polyline
                fill="none"
                stroke="#fbbf24"
                strokeWidth="2.5"
                points="0,95 20,80 40,85 70,60 100,65 130,40 160,50 190,25 220,35 240,15"
              />
              <polyline
                fill="none"
                stroke="#f43f5e"
                strokeWidth="1.5"
                strokeDasharray="3 3"
                points="0,90 30,85 60,75 100,70 140,55 180,45 220,40 240,30"
              />
            </svg>
          </div>

          <div className="flex justify-between text-[10px] font-mono text-slate-400 border-t border-amber-500/10 pt-2">
            <span>Threshold: 1,000 EPS DLQ</span>
            <span className="text-emerald-400">CURRENT: 134 EPS</span>
          </div>
        </div>
      </div>

      {/* Bottom Section: Forensic Event Ledger (Section 65B Indian Evidence Act compliant) */}
      <div className="rounded-2xl border border-amber-500/30 bg-[#0c0d12] p-5 shadow-2xl space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-amber-500/20 pb-3">
          <div className="flex items-center gap-2">
            <Lock className="w-5 h-5 text-emerald-400" />
            <div>
              <h3 className="text-base font-bold font-mono text-amber-400 tracking-tight">
                Forensic Event Ledger · Section 65B Cryptographic Proof
              </h3>
              <p className="text-xs text-slate-400 font-mono">
                OCSF Class 4001 Events linked to Parquet 2.0 Vault with BLAKE3 256-bit Immutable Provenance Hashes
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <span className="px-3 py-1 rounded-full text-xs font-mono bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 flex items-center gap-1.5">
              <CheckCircle2 className="w-3.5 h-3.5" /> SEC 65B EVIDENCE ADMISSIBLE
            </span>
          </div>
        </div>

        {/* Ledger Table */}
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs font-mono">
            <thead>
              <tr className="border-b border-amber-500/20 text-slate-400 text-[11px] uppercase tracking-wider">
                <th className="py-2.5 px-3">Time (UTC)</th>
                <th className="py-2.5 px-3">Severity</th>
                <th className="py-2.5 px-3">Source → Destination</th>
                <th className="py-2.5 px-3">Firewall Vendor</th>
                <th className="py-2.5 px-3">Action</th>
                <th className="py-2.5 px-3">BLAKE3 Provenance Hash</th>
                <th className="py-2.5 px-3 text-right">Audit Proof</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-amber-500/10">
              {SAMPLE_FORENSIC_EVENTS.map((evt) => (
                <tr key={evt.id} className="hover:bg-amber-500/5 transition-colors">
                  <td className="py-3 px-3 text-slate-300 whitespace-nowrap">{evt.formattedTime}</td>
                  <td className="py-3 px-3">
                    <span
                      className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
                        evt.severity === 'Critical'
                          ? 'bg-rose-500/20 text-rose-400 border border-rose-500/40'
                          : evt.severity === 'High'
                          ? 'bg-amber-500/20 text-amber-400 border border-amber-500/40'
                          : 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/40'
                      }`}
                    >
                      {evt.severity}
                    </span>
                  </td>
                  <td className="py-3 px-3 text-slate-200">
                    <div className="font-semibold">
                      {evt.src_ip} → {evt.dst_ip}:{evt.dst_port}
                    </div>
                    <div className="text-[10px] text-slate-400">
                      {evt.src_country} · {evt.protocol}
                    </div>
                  </td>
                  <td className="py-3 px-3 text-slate-300">{evt.vendor}</td>
                  <td className="py-3 px-3">
                    <span
                      className={`inline-flex items-center gap-1 font-semibold ${
                        evt.activity_label === 'Allow'
                          ? 'text-emerald-400'
                          : evt.activity_label === 'Deny'
                          ? 'text-rose-400'
                          : 'text-amber-400'
                      }`}
                    >
                      {evt.activity_label === 'Allow' ? (
                        <CheckCircle2 className="w-3.5 h-3.5" />
                      ) : (
                        <XCircle className="w-3.5 h-3.5" />
                      )}
                      {evt.activity_label}
                    </span>
                  </td>
                  <td className="py-3 px-3">
                    <div className="max-w-[190px] truncate text-slate-400 font-mono text-[11px] bg-black/60 px-2 py-1 rounded border border-slate-800">
                      {evt.provenance_hash}
                    </div>
                  </td>
                  <td className="py-3 px-3 text-right">
                    <button
                      onClick={() => setSelectedAuditEvt(evt)}
                      className="px-2.5 py-1 rounded bg-amber-500/20 border border-amber-500/40 text-amber-300 hover:bg-amber-500/30 text-[11px] font-semibold transition"
                    >
                      Verify 65B
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
