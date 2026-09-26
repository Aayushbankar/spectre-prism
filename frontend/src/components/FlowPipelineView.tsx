import React, { useState } from 'react';
import {
  Globe,
  Download,
  Users,
  AlertTriangle,
  CheckCircle,
  ShieldCheck,
  HeartPulse,
  Info,
  HelpCircle,
  ExternalLink,
} from 'lucide-react';
import { INITIAL_SUMMARY } from '../mock/data';
import { SankeyDiagram } from './SankeyDiagram';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  isLiveStreaming: boolean;
}

export const FlowPipelineView: React.FC<Props> = ({ theme, isLiveStreaming }) => {
  const [activeTooltip, setActiveTooltip] = useState<string | null>(null);

  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  return (
    <div className="space-y-6">
      {/* Top Banner: Ingestion Overview & Health Score */}
      <div
        className={`relative overflow-hidden rounded-2xl border p-6 transition-all duration-300 shadow-xl ${
          isCyber
            ? 'border-amber-500/30 bg-black/85 text-amber-50 shadow-amber-500/10'
            : isDark
            ? 'border-slate-800 bg-[#141724]/95 text-slate-100 shadow-2xl backdrop-blur-xl'
            : 'border-slate-200 bg-white/95 text-slate-800 shadow-lg'
        }`}
      >
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-center">
          {/* Left Summary Narrative (8 cols) */}
          <div className="lg:col-span-8 space-y-4">
            <div className="flex items-center gap-2.5">
              <span className="w-1.5 h-6 rounded-full bg-cyan-400 inline-block shadow-[0_0_12px_#22d3ee]" />
              <h2 className="text-xl font-bold tracking-tight">Dashboard Summary</h2>
              {isLiveStreaming && (
                <span className="ml-2 inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                  <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping" />
                  Live Ingestion Stream Active
                </span>
              )}
            </div>

            <p className={`text-sm leading-relaxed ${isDark ? 'text-slate-300' : 'text-slate-600'}`}>
              A total of <strong className="text-cyan-400 font-semibold">{INITIAL_SUMMARY.totalRequests.toLocaleString()}</strong> requests arrived from{' '}
              <strong className="text-cyan-400 font-semibold">{INITIAL_SUMMARY.uniqueVisitors.toLocaleString()}</strong> unique visitors, reaching{' '}
              <strong className="text-cyan-400 font-semibold">{INITIAL_SUMMARY.countriesReached}</strong> destination countries. In all,{' '}
              <strong className="text-cyan-400 font-semibold">{INITIAL_SUMMARY.dataTransferredMB} MB</strong> of traffic moved, with an average payload of{' '}
              <strong className="text-cyan-400 font-semibold">{INITIAL_SUMMARY.avgPayloadBytes} bytes</strong> and a largest single response of{' '}
              <strong className="text-cyan-400 font-semibold">{INITIAL_SUMMARY.largestResponseBytes} bytes</strong>. On the reliability front,{' '}
              <strong className="text-cyan-400 font-semibold">{INITIAL_SUMMARY.succeededRequests.toLocaleString()}</strong> requests succeeded against{' '}
              <strong className="text-rose-400 font-semibold">{INITIAL_SUMMARY.failedRequests.toLocaleString()}</strong> failures — of those,{' '}
              <strong className="text-rose-400 font-semibold">87</strong> were server errors (5xx) and{' '}
              <strong className="text-amber-400 font-semibold">118</strong> were not-found (404). The 5xx server errors are the more serious signal to watch, since they point to backend problems rather than client mistakes, and they weigh directly on availability (<strong className="text-emerald-400 font-semibold">{INITIAL_SUMMARY.availabilityPct}%</strong>).
            </p>

            {/* Operational Posture Indicators */}
            <div className="flex flex-wrap items-center gap-2 pt-2">
              <span className="px-3 py-1 rounded-full text-xs font-semibold bg-slate-800/90 text-emerald-400 border border-slate-700/80 shadow-inner flex items-center gap-1.5">
                <CheckCircle className="w-3.5 h-3.5" />
                Success {INITIAL_SUMMARY.successPct}%
              </span>
              <span className="px-3 py-1 rounded-full text-xs font-semibold bg-slate-800/90 text-rose-400 border border-slate-700/80 shadow-inner flex items-center gap-1.5">
                <AlertTriangle className="w-3.5 h-3.5" />
                Server errors 87
              </span>
              <span className="px-3 py-1 rounded-full text-xs font-semibold bg-slate-800/90 text-cyan-400 border border-slate-700/80 shadow-inner flex items-center gap-1.5">
                <Globe className="w-3.5 h-3.5" />
                Reach {INITIAL_SUMMARY.countriesReached} countries
              </span>
              <span className="px-3 py-1 rounded-full text-xs font-semibold bg-slate-800/90 text-purple-400 border border-slate-700/80 shadow-inner">
                OCSF Class 4001 Compliant
              </span>
            </div>
          </div>

          {/* Right Health Score Gauge & Indicators (4 cols) */}
          <div className="lg:col-span-4 flex flex-col items-center justify-center p-5 rounded-2xl bg-slate-900/60 border border-slate-800/80 relative">
            <div className="text-xs uppercase tracking-wider font-bold text-slate-400 mb-1 flex items-center gap-1.5">
              <HeartPulse className="w-4 h-4 text-emerald-400" />
              Health Score
            </div>

            <div className="flex items-baseline gap-1 my-1">
              <span className="text-5xl font-black text-emerald-400 drop-shadow-[0_0_20px_rgba(52,211,153,0.5)]">
                {INITIAL_SUMMARY.healthScore}
              </span>
              <span className="text-2xl font-semibold text-slate-400">/ 100</span>
            </div>

            <div className="text-xs font-extrabold text-emerald-400 tracking-widest uppercase mb-5">
              {INITIAL_SUMMARY.healthStatus}
            </div>

            {/* 4 Circular SLI Badges with clear subtext */}
            <div className="grid grid-cols-4 gap-2.5 w-full text-center">
              <div className="flex flex-col items-center group cursor-pointer" title="92.3% of total events successfully parsed into OCSF without error">
                <div className="w-10 h-10 rounded-full bg-emerald-500/20 border-2 border-emerald-500 flex items-center justify-center shadow-[0_0_12px_rgba(16,185,129,0.4)]">
                  <CheckCircle className="w-5 h-5 text-emerald-400" />
                </div>
                <span className="text-xs font-bold mt-1 text-slate-200">{INITIAL_SUMMARY.successPct}%</span>
                <span className="text-[10px] text-slate-400 font-medium">Success</span>
              </div>

              <div className="flex flex-col items-center group cursor-pointer" title="96.7% uptime across the zero-alloc UDP/TCP listener sockets">
                <div className="w-10 h-10 rounded-full bg-emerald-500/20 border-2 border-emerald-500 flex items-center justify-center shadow-[0_0_12px_rgba(16,185,129,0.4)]">
                  <ShieldCheck className="w-5 h-5 text-emerald-400" />
                </div>
                <span className="text-xs font-bold mt-1 text-slate-200">{INITIAL_SUMMARY.availabilityPct}%</span>
                <span className="text-[10px] text-slate-400 font-medium">Availability</span>
              </div>

              <div className="flex flex-col items-center group cursor-pointer" title="4.4% unmapped or 404 logs requiring rule update or triage">
                <div className="w-10 h-10 rounded-full bg-emerald-500/20 border-2 border-emerald-500 flex items-center justify-center shadow-[0_0_12px_rgba(16,185,129,0.4)]">
                  <span className="text-xs font-extrabold text-emerald-400">4.4%</span>
                </div>
                <span className="text-xs font-bold mt-1 text-slate-200">{INITIAL_SUMMARY.notFoundPct}%</span>
                <span className="text-[10px] text-slate-400 font-medium">Not Found</span>
              </div>

              <div className="flex flex-col items-center group cursor-pointer" title="92.3% remaining SLA error budget before breach">
                <div className="w-10 h-10 rounded-full bg-emerald-500/20 border-2 border-emerald-500 flex items-center justify-center shadow-[0_0_12px_rgba(16,185,129,0.4)]">
                  <span className="text-xs font-extrabold text-emerald-400">92%</span>
                </div>
                <span className="text-xs font-bold mt-1 text-slate-200">{INITIAL_SUMMARY.errorBudgetPct}%</span>
                <span className="text-[10px] text-slate-400 font-medium">Error Budget</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 4 Ingestion Stat Metric Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* TOTAL REQUESTS */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#141724]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
          style={{ borderLeft: '5px solid #06b6d4' }}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold tracking-wider uppercase text-slate-400">TOTAL REQUESTS</span>
            <Globe className="w-5 h-5 text-cyan-400" />
          </div>
          <div className="mt-2 text-3xl font-extrabold text-slate-100 tracking-tight">2669</div>
          <div className="mt-1 text-xs text-slate-400">zero-allocation UDP/TCP buffer chunks</div>
        </div>

        {/* DATA TRANSFERRED */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#141724]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
          style={{ borderLeft: '5px solid #3b82f6' }}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold tracking-wider uppercase text-slate-400">DATA TRANSFERRED</span>
            <Download className="w-5 h-5 text-blue-400" />
          </div>
          <div className="mt-2 text-3xl font-extrabold text-slate-100 tracking-tight">14.59 MB</div>
          <div className="mt-1 text-xs text-slate-400">response bytes sent to vault</div>
        </div>

        {/* UNIQUE VISITORS */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#141724]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
          style={{ borderLeft: '5px solid #a855f7' }}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold tracking-wider uppercase text-slate-400">UNIQUE VISITORS</span>
            <Users className="w-5 h-5 text-purple-400" />
          </div>
          <div className="mt-2 text-3xl font-extrabold text-slate-100 tracking-tight">1000</div>
          <div className="mt-1 text-xs text-slate-400">distributed firewall sources</div>
        </div>

        {/* ERROR RATE */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#141724]/95 border-slate-800' : 'bg-white border-slate-200'
          }`}
          style={{ borderLeft: '5px solid #f43f5e' }}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs font-bold tracking-wider uppercase text-slate-400">ERROR RATE</span>
            <AlertTriangle className="w-5 h-5 text-rose-400" />
          </div>
          <div className="mt-2 text-3xl font-extrabold text-slate-100 tracking-tight">7.7%</div>
          <div className="mt-1 text-xs text-rose-400 font-medium">205 failed • Routed to DLQ & HitL</div>
        </div>
      </div>

      {/* Traffic Flow Pipeline Topology Sankey */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="w-1.5 h-5 rounded-full bg-cyan-400 inline-block" />
            <h3 className="text-lg font-bold tracking-tight">
              Traffic Flow · Destination → Platform → Status
            </h3>
          </div>
          <span className="text-xs text-slate-400 font-mono">ribbon width = request volume</span>
        </div>

        {/* Interactive SVG Sankey */}
        <SankeyDiagram theme={theme} />
      </div>
    </div>
  );
};
