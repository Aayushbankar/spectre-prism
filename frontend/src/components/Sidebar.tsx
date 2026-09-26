import React from 'react';
import {
  LayoutDashboard,
  Zap,
  Activity,
  Globe,
  Binary,
  Cpu,
  AlertTriangle,
  FileCode,
  Moon,
  Sun,
  ShieldCheck,
  Lock,
} from 'lucide-react';
import type { DashboardView, DashboardTheme } from '../types';

interface Props {
  currentView: DashboardView;
  onViewChange: (view: DashboardView) => void;
  theme: DashboardTheme;
  onThemeChange: (theme: DashboardTheme) => void;
}

export const Sidebar: React.FC<Props> = ({ currentView, onViewChange, theme, onThemeChange }) => {
  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  const navGroups = [
    {
      group: 'Mission Control',
      items: [
        { id: 'command' as DashboardView, label: 'Command Center', icon: LayoutDashboard, badge: 'SOC' },
        { id: 'flow' as DashboardView, label: 'Pipeline Topology', icon: Zap, badge: 'FLOW' },
        { id: 'analytics' as DashboardView, label: 'Engine Telemetry', icon: Activity, badge: 'SLA' },
        { id: 'threats' as DashboardView, label: 'Threat Radar', icon: Globe, badge: 'HUD' },
      ],
    },
    {
      group: 'Evidence & Compliance',
      items: [
        { id: 'accounting' as DashboardView, label: 'Byte Accounting & Vault', icon: Binary, badge: '§65B' },
        { id: 'dlq' as DashboardView, label: 'DLQ Quarantine', icon: AlertTriangle, badge: 'DLQ' },
      ],
    },
    {
      group: 'Autonomous Intelligence',
      items: [
        { id: 'gatekeeper' as DashboardView, label: 'HitL Gatekeeper', icon: Cpu, badge: 'AI' },
        { id: 'vrl' as DashboardView, label: 'VRL Sandbox', icon: FileCode, badge: 'TEST' },
      ],
    },
  ];

  return (
    <aside
      className={`w-64 shrink-0 flex flex-col justify-between border-r p-4 transition-all duration-300 min-h-screen select-none ${
        isCyber
          ? 'border-amber-500/20 bg-black text-amber-50'
          : isDark
          ? 'border-slate-800 bg-[#0f111a] text-slate-200'
          : 'border-slate-200 bg-white text-slate-800 shadow-sm'
      }`}
    >
      {/* Top Branding */}
      <div className="space-y-5">
        <div className="flex items-center gap-3 px-2 pt-1">
          <div className="relative w-10 h-10 rounded-xl bg-gradient-to-tr from-cyan-500 to-indigo-600 flex items-center justify-center text-white shadow-lg shadow-cyan-500/20">
            <Zap className="w-5 h-5 fill-current" />
            <span className="absolute -top-1 -right-1 w-2.5 h-2.5 rounded-full bg-emerald-400 border-2 border-[#0f111a]" />
          </div>
          <div>
            <div className="flex items-center gap-1.5">
              <span className="font-extrabold tracking-tight text-base text-white">PRISM</span>
              <span className="text-[9px] uppercase font-bold tracking-widest text-cyan-300 bg-cyan-950/90 px-1.5 py-0.5 rounded border border-cyan-800">
                NTRO SOC
              </span>
            </div>
            <p className="text-[10px] text-slate-400 font-mono tracking-tight">Sovereign Defense Engine</p>
          </div>
        </div>

        {/* Grouped Navigation */}
        <nav className="space-y-4">
          {navGroups.map((group) => (
            <div key={group.group} className="space-y-1">
              <div className="text-[10px] font-bold uppercase tracking-wider text-slate-500 px-3 mb-1 font-mono">
                {group.group}
              </div>
              {group.items.map((item) => {
                const Icon = item.icon;
                const isActive = currentView === item.id;
                return (
                  <button
                    key={item.id}
                    onClick={() => onViewChange(item.id)}
                    className={`w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-200 ${
                      isActive
                        ? isCyber
                          ? 'bg-amber-500 text-black font-bold shadow-[0_0_12px_#f59e0b]'
                          : 'bg-gradient-to-r from-cyan-600 to-indigo-600 text-white shadow-lg shadow-cyan-600/30'
                        : isDark
                        ? 'text-slate-300 hover:bg-slate-800/60 hover:text-white'
                        : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'
                    }`}
                  >
                    <div className="flex items-center gap-2.5">
                      <Icon className="w-4 h-4 shrink-0" />
                      <span className="truncate">{item.label}</span>
                    </div>
                    <span
                      className={`text-[9px] font-mono px-1.5 py-0.5 rounded shrink-0 ${
                        isActive
                          ? isCyber
                            ? 'bg-black/20 text-black font-bold'
                            : 'bg-white/20 text-white'
                          : 'bg-slate-800/60 text-slate-400'
                      }`}
                    >
                      {item.badge}
                    </span>
                  </button>
                );
              })}
            </div>
          ))}
        </nav>
      </div>

      {/* Bottom Area: Section 65B Status + Theme Switcher */}
      <div className="space-y-3 pt-3 border-t border-slate-800/80">
        {/* Compliance Card */}
        <div className="p-2.5 rounded-xl bg-slate-900/80 border border-slate-800/80 space-y-1.5 text-xs">
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5 text-emerald-400 text-[11px] font-bold">
              <ShieldCheck className="w-3.5 h-3.5" /> Sec 65B Quorum
            </span>
            <span className="text-[10px] font-mono text-emerald-400 bg-emerald-950/60 px-1 rounded border border-emerald-800">
              2/3 VERIFIED
            </span>
          </div>
          <div className="flex items-center justify-between text-[10px] text-slate-400 font-mono">
            <span>Merkle BLAKE3</span>
            <span className="text-cyan-400">492 Roots</span>
          </div>
          <div className="flex items-center justify-between text-[10px] text-slate-400 font-mono">
            <span>Buffer Fill</span>
            <span className="text-emerald-400">42% (0 Drops)</span>
          </div>
        </div>

        {/* 3-way Theme Selector */}
        <div className="p-1 rounded-xl bg-slate-900/80 border border-slate-800/80 flex items-center justify-between text-xs">
          <button
            onClick={() => onThemeChange('dark')}
            className={`flex-1 py-1 px-1 rounded-lg flex items-center justify-center gap-1 transition ${
              theme === 'dark' ? 'bg-slate-800 text-cyan-400 font-bold shadow' : 'text-slate-400 hover:text-white'
            }`}
            title="Dark Theme"
          >
            <Moon className="w-3 h-3" />
            <span className="text-[9px]">Dark</span>
          </button>
          <button
            onClick={() => onThemeChange('light')}
            className={`flex-1 py-1 px-1 rounded-lg flex items-center justify-center gap-1 transition ${
              theme === 'light' ? 'bg-white text-slate-900 font-bold shadow' : 'text-slate-400 hover:text-white'
            }`}
            title="Light Theme"
          >
            <Sun className="w-3 h-3" />
            <span className="text-[9px]">Light</span>
          </button>
          <button
            onClick={() => onThemeChange('cyber')}
            className={`flex-1 py-1 px-1 rounded-lg flex items-center justify-center gap-1 transition ${
              theme === 'cyber' ? 'bg-amber-500 text-black font-bold shadow' : 'text-slate-400 hover:text-white'
            }`}
            title="Tactical Cyber Theme"
          >
            <Zap className="w-3 h-3" />
            <span className="text-[9px]">Cyber</span>
          </button>
        </div>
      </div>
    </aside>
  );
};
