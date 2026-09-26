import React from 'react';
import {
  Search,
  Bell,
  Download,
  Filter,
  Play,
  Pause,
  ChevronDown,
  User,
  Shield,
  Layers,
  Zap,
  LayoutDashboard,
  Globe,
} from 'lucide-react';
import type { DashboardView, DashboardTheme } from '../types';

interface Props {
  currentView: DashboardView;
  onViewChange: (view: DashboardView) => void;
  theme: DashboardTheme;
  isLiveStreaming: boolean;
  onToggleLiveStream: () => void;
}

export const Header: React.FC<Props> = ({
  currentView,
  onViewChange,
  theme,
  isLiveStreaming,
  onToggleLiveStream,
}) => {
  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  const viewLabels: Record<DashboardView, { title: string; subtitle: string }> = {
    pipeline: {
      title: 'Traffic & Ingestion Flow',
      subtitle: 'Design 1 (ideainso3.jpeg) · Zero-Alloc Ingest & OCSF Class 4001 Egress Flow',
    },
    analytics: {
      title: 'Executive Analytics & Telemetry',
      subtitle: 'Design 2 (dashboard2.jpeg) · Real-Time Throughput & Firewall Distributions',
    },
    tactical: {
      title: 'Tactical Cyber Command Center',
      subtitle: 'Design 3 (dashboard_1.jpeg) · Global Attack Vector Map & Section 65B Forensic Ledger',
    },
    all: {
      title: 'PRISM Unified Command Center',
      subtitle: 'All 3 Adapted Architectures · Seamless End-to-End Log Parsing Observability',
    },
  };

  return (
    <header
      className={`border-b px-6 py-4 transition-all duration-300 ${
        isCyber
          ? 'border-amber-500/20 bg-black/90 text-amber-50'
          : isDark
          ? 'border-slate-800 bg-[#12141d]/90 text-slate-100 backdrop-blur-md'
          : 'border-slate-200 bg-white/95 text-slate-800 shadow-sm'
      }`}
    >
      <div className="flex flex-wrap items-center justify-between gap-4">
        {/* Left: View Title & Subtitle */}
        <div>
          <div className="flex items-center gap-3">
            <h1 className="text-xl font-bold tracking-tight">{viewLabels[currentView].title}</h1>
            <span
              className={`text-[10px] font-mono px-2 py-0.5 rounded font-bold ${
                isCyber
                  ? 'bg-amber-500/20 text-amber-400 border border-amber-500/40'
                  : 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30'
              }`}
            >
              PRISM v1.9
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-0.5">{viewLabels[currentView].subtitle}</p>
        </div>

        {/* View Switcher Tabs (Quick Access to D1, D2, D3, All) */}
        <div className="flex items-center gap-1 p-1 rounded-xl bg-slate-900/80 border border-slate-800 text-xs">
          <button
            onClick={() => onViewChange('pipeline')}
            className={`px-3 py-1.5 rounded-lg font-medium transition flex items-center gap-1.5 ${
              currentView === 'pipeline'
                ? 'bg-cyan-600 text-white shadow'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Zap className="w-3.5 h-3.5" />
            <span>Flow (D1)</span>
          </button>

          <button
            onClick={() => onViewChange('analytics')}
            className={`px-3 py-1.5 rounded-lg font-medium transition flex items-center gap-1.5 ${
              currentView === 'analytics'
                ? 'bg-purple-600 text-white shadow'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <LayoutDashboard className="w-3.5 h-3.5" />
            <span>Analytics (D2)</span>
          </button>

          <button
            onClick={() => onViewChange('tactical')}
            className={`px-3 py-1.5 rounded-lg font-medium transition flex items-center gap-1.5 ${
              currentView === 'tactical'
                ? 'bg-amber-500 text-black font-bold shadow'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Globe className="w-3.5 h-3.5" />
            <span>Tactical (D3)</span>
          </button>

          <button
            onClick={() => onViewChange('all')}
            className={`px-3 py-1.5 rounded-lg font-medium transition flex items-center gap-1.5 ${
              currentView === 'all'
                ? 'bg-emerald-600 text-white shadow'
                : 'text-slate-400 hover:text-white'
            }`}
          >
            <Layers className="w-3.5 h-3.5" />
            <span>Unified</span>
          </button>
        </div>

        {/* Right: Actions, Live Stream Toggle, Search & User Avatar */}
        <div className="flex items-center gap-3">
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
              placeholder="Search IP, Hash, CVE..."
              className="w-48 lg:w-56 pl-9 pr-3 py-1.5 rounded-xl text-xs bg-slate-900/80 border border-slate-800 text-slate-200 placeholder:text-slate-500 focus:outline-none focus:border-cyan-500 transition"
            />
          </div>

          {/* Export / Download */}
          <button
            onClick={() => alert('Exporting PRISM OCSF NDJSON Batch (Section 65B Merkle Rooted)...')}
            className="p-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white border border-slate-700 transition"
            title="Download OCSF NDJSON Payload"
          >
            <Download className="w-4 h-4" />
          </button>

          {/* Notifications with badge (matches "2" in dashboard2.jpeg) */}
          <div className="relative">
            <button className="p-2 rounded-xl bg-slate-800 text-slate-300 hover:text-white border border-slate-700 transition">
              <Bell className="w-4 h-4" />
            </button>
            <span className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-rose-500 text-white text-[10px] font-bold flex items-center justify-center">
              2
            </span>
          </div>

          {/* User Profile (matches Malena Bayer in dashboard2.jpeg) */}
          <div className="flex items-center gap-2 pl-2 border-l border-slate-800">
            <div className="w-8 h-8 rounded-full bg-gradient-to-tr from-amber-400 to-rose-500 p-0.5 shadow">
              <div className="w-full h-full rounded-full bg-slate-900 flex items-center justify-center text-xs font-bold text-amber-300">
                MB
              </div>
            </div>
            <div className="hidden xl:block text-left text-xs leading-tight">
              <div className="font-bold text-slate-200">Malena Bayer</div>
              <div className="text-[10px] text-slate-400">SecOps Lead</div>
            </div>
          </div>
        </div>
      </div>
    </header>
  );
};
