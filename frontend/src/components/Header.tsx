import React from 'react';
import {
  Search,
  Bell,
  Download,
  Play,
  Pause,
  Zap,
  Activity,
  Globe,
  LayoutDashboard,
  HelpCircle,
  Cpu,
  AlertTriangle,
  FileCode,
  Binary,
  ShieldCheck,
} from 'lucide-react';
import type { DashboardView, DashboardTheme } from '../types';

interface Props {
  currentView: DashboardView;
  onViewChange: (view: DashboardView) => void;
  theme: DashboardTheme;
  isLiveStreaming: boolean;
  onToggleLiveStream: () => void;
  onOpenExplainer?: () => void;
}

export const Header: React.FC<Props> = ({
  currentView,
  onViewChange,
  theme,
  isLiveStreaming,
  onToggleLiveStream,
  onOpenExplainer,
}) => {
  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  const viewLabels: Record<DashboardView, { title: string; subtitle: string }> = {
    command: {
      title: 'Sovereign SOC Command Center',
      subtitle: 'Unified Operational Cockpit · Live Cyber Radar, 300k EPS Telemetry & Section 65B Audit Stream',
    },
    flow: {
      title: 'Pipeline Topology & Normalization',
      subtitle: 'Tokio Zero-Alloc Ring Buffers · VRL Rules · Dynamic OCSF Class Egress',
    },
    analytics: {
      title: 'Engine Telemetry & Performance SLAs',
      subtitle: 'Throughput Trends · Multi-Vendor Distribution · Latency Micro-Histograms',
    },
    threats: {
      title: 'Tactical Threat Radar & Ballistics',
      subtitle: 'Real-Time Attack Vector Tracking · Geolocation Intel · Gateway Interception',
    },
    accounting: {
      title: 'Byte Accounting & Legal Vault (§3.1)',
      subtitle: 'FIELD / LITERAL / RESIDUE Tracking · Closure Ratio & 2-of-3 Witness Quorum',
    },
    gatekeeper: {
      title: 'Autonomous AI Gatekeeper (HitL)',
      subtitle: 'Drain3 Template Mining & ModernBERT Rule Classification Staging Area',
    },
    dlq: {
      title: 'Dead Letter Queue (DLQ) Quarantine',
      subtitle: 'Anomalous Payload Triage · Root-Cause Analysis & Automated Remediation',
    },
    vrl: {
      title: 'Interactive VRL Sandbox & Validator',
      subtitle: 'Vector Remap Language · Live OCSF Schema Normalization & Proof Verification',
    },
  };

  const navButtons: Array<{ id: DashboardView; label: string; icon: React.ComponentType<{ className?: string }> }> = [
    { id: 'command', label: 'Command', icon: LayoutDashboard },
    { id: 'flow', label: 'Topology', icon: Zap },
    { id: 'analytics', label: 'Telemetry', icon: Activity },
    { id: 'threats', label: 'Radar', icon: Globe },
    { id: 'accounting', label: 'Vault', icon: Binary },
    { id: 'gatekeeper', label: 'AI Gate', icon: Cpu },
    { id: 'dlq', label: 'DLQ', icon: AlertTriangle },
    { id: 'vrl', label: 'Sandbox', icon: FileCode },
  ];

  const currentMeta = viewLabels[currentView] || viewLabels.command;

  return (
    <header
      className={`border-b px-6 py-4 transition-all duration-300 select-none ${
        isCyber
          ? 'border-amber-500/20 bg-black/95 text-amber-50'
          : isDark
          ? 'border-slate-800 bg-[#0f111a]/95 text-slate-100 backdrop-blur-md'
          : 'border-slate-200 bg-white/95 text-slate-800 shadow-sm'
      }`}
    >
      <div className="flex flex-wrap items-center justify-between gap-4">
        {/* Left: View Title & Subtitle */}
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-xl font-extrabold tracking-tight">{currentMeta.title}</h1>
            <span
              className={`text-[10px] font-mono px-2 py-0.5 rounded font-bold ${
                isCyber
                  ? 'bg-amber-500/20 text-amber-400 border border-amber-500/40'
                  : 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30'
              }`}
            >
              PRISM v1.9-PROD
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-0.5 font-medium">{currentMeta.subtitle}</p>
        </div>

        {/* View Switcher Tabs (Quick Access to Core Workspaces) */}
        <div className="flex flex-wrap items-center gap-1 p-1 rounded-xl bg-slate-900/90 border border-slate-800 text-xs font-semibold">
          {navButtons.map((btn) => {
            const Icon = btn.icon;
            const isActive = currentView === btn.id;
            return (
              <button
                key={btn.id}
                onClick={() => onViewChange(btn.id)}
                className={`px-3 py-1.5 rounded-lg transition flex items-center gap-1.5 ${
                  isActive
                    ? isCyber
                      ? 'bg-amber-500 text-black font-bold shadow'
                      : 'bg-cyan-600 text-white font-bold shadow'
                    : 'text-slate-400 hover:text-white'
                }`}
              >
                <Icon className="w-3.5 h-3.5" />
                <span>{btn.label}</span>
              </button>
            );
          })}
        </div>

        {/* Right: Actions, Guide, Live Stream Toggle, Search & User Avatar */}
        <div className="flex items-center gap-2.5">
          {/* Architecture Guide Button */}
          {onOpenExplainer && (
            <button
              onClick={onOpenExplainer}
              className="px-2.5 py-1.5 rounded-xl border border-cyan-500/40 bg-cyan-950/40 text-cyan-300 hover:bg-cyan-950/80 text-xs font-semibold flex items-center gap-1.5 transition"
              title="Architecture & Compliance Guide"
            >
              <HelpCircle className="w-3.5 h-3.5 text-cyan-400" />
              <span className="hidden sm:inline">Guide</span>
            </button>
          )}

          {/* Live Stream Simulation Toggle */}
          <button
            onClick={onToggleLiveStream}
            className={`px-3 py-1.5 rounded-xl border text-xs font-semibold flex items-center gap-1.5 transition ${
              isLiveStreaming
                ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40 hover:bg-emerald-500/30'
                : 'bg-slate-800 text-slate-400 border-slate-700 hover:text-white'
            }`}
            title="Toggle Live Stream Simulation"
          >
            {isLiveStreaming ? <Pause className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5" />}
            <span>{isLiveStreaming ? 'Streaming' : 'Paused'}</span>
          </button>

          {/* Search Bar */}
          <div className="relative hidden md:block">
            <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
            <input
              type="text"
              placeholder="Search IP, Hash, VRL..."
              className="w-44 lg:w-52 pl-9 pr-3 py-1.5 rounded-xl text-xs bg-slate-900/90 border border-slate-800 text-slate-200 placeholder:text-slate-500 focus:outline-none focus:border-cyan-500 transition"
            />
          </div>

          {/* Export / Download */}
          <button
            onClick={() => alert('Exporting PRISM OCSF NDJSON Batch (Section 65B Merkle Rooted)...')}
            className="p-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white border border-slate-700 transition"
            title="Download OCSF Forensic Bundle"
          >
            <Download className="w-4 h-4" />
          </button>

          {/* Notifications with badge */}
          <div className="relative">
            <button className="p-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white border border-slate-700 transition">
              <Bell className="w-4 h-4" />
            </button>
            <span className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-rose-500 text-white text-[10px] font-bold flex items-center justify-center">
              2
            </span>
          </div>

          {/* User Profile */}
          <div className="flex items-center gap-2 pl-2 border-l border-slate-800">
            <div className="w-8 h-8 rounded-full bg-gradient-to-tr from-cyan-400 to-indigo-500 p-0.5 shadow">
              <div className="w-full h-full rounded-full bg-slate-900 flex items-center justify-center text-xs font-bold text-cyan-300">
                NS
              </div>
            </div>
            <div className="hidden xl:block text-left text-xs leading-tight">
              <div className="font-bold text-slate-200">NTRO SecOps</div>
              <div className="text-[10px] text-slate-400">Chief Operator</div>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
};
