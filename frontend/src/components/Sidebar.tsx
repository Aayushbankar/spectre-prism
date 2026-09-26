import React from 'react';
import {
  Layers,
  LayoutDashboard,
  FileText,
  ShieldAlert,
  Sliders,
  HelpCircle,
  LogOut,
  Moon,
  Sun,
  Zap,
  Globe,
  Cpu,
  AlertTriangle,
  FileCode,
  Binary,
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

  const navItems = [
    { id: 'pipeline' as DashboardView, label: 'Ingest Flow', icon: Zap, badge: 'D1' },
    { id: 'analytics' as DashboardView, label: 'Analytics', icon: LayoutDashboard, badge: 'D2' },
    { id: 'tactical' as DashboardView, label: 'Tactical Map', icon: Globe, badge: 'D3' },
    { id: 'accounting' as DashboardView, label: 'Byte Accounting', icon: Binary, badge: '§3.1' },
    { id: 'gatekeeper' as DashboardView, label: 'HitL Gatekeeper', icon: Cpu, badge: 'AI' },
    { id: 'dlq' as DashboardView, label: 'DLQ Inspector', icon: AlertTriangle, badge: 'ERR' },
    { id: 'vrl' as DashboardView, label: 'VRL Sandbox', icon: FileCode, badge: 'TEST' },
    { id: 'all' as DashboardView, label: 'Unified View', icon: Layers, badge: 'ALL' },
  ];

  return (
    <aside
      className={`w-64 shrink-0 flex flex-col justify-between border-r p-5 transition-all duration-300 min-h-screen ${
        isCyber
          ? 'border-amber-500/20 bg-black text-amber-50'
          : isDark
          ? 'border-slate-800 bg-[#12141d] text-slate-200'
          : 'border-slate-200 bg-white text-slate-800 shadow-sm'
      }`}
    >
      {/* Top Branding */}
      <div className="space-y-6">
        <div className="flex items-center gap-3 px-1">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-cyan-500 to-purple-600 flex items-center justify-center text-white shadow-lg shadow-purple-500/20">
            <Zap className="w-5 h-5 fill-current" />
          </div>
          <div>
            <div className="flex items-center gap-1.5">
              <span className="font-extrabold tracking-tight text-base">PRISM</span>
              <span className="text-[10px] uppercase font-bold tracking-widest text-cyan-400 bg-cyan-950/80 px-1.5 py-0.5 rounded border border-cyan-800">
                SIH 26156
              </span>
            </div>
            <p className="text-[11px] text-slate-400 font-medium">Universal Log Parsing</p>
          </div>
        </div>

        {/* Main Navigation */}
        <nav className="space-y-1.5">
          <div className="text-[11px] font-bold uppercase tracking-wider text-slate-400 px-3 mb-2">
            Operations & Control
          </div>
          {navItems.map((item) => {
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
                      : 'bg-gradient-to-r from-purple-600 to-indigo-600 text-white shadow-lg shadow-purple-600/30'
                    : isDark
                    ? 'text-slate-300 hover:bg-slate-800/60 hover:text-white'
                    : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'
                }`}
              >
                <div className="flex items-center gap-2.5">
                  <Icon className="w-4 h-4" />
                  <span>{item.label}</span>
                </div>
                <span
                  className={`text-[9px] font-mono px-1.5 py-0.5 rounded ${
                    isActive
                      ? isCyber
                        ? 'bg-black/20 text-black font-bold'
                        : 'bg-white/20 text-white'
                      : 'bg-slate-800/40 text-slate-400'
                  }`}
                >
                  {item.badge}
                </span>
              </button>
            );
          })}
        </nav>

        {/* Security & Vault Compliance Links */}
        <div className="space-y-1 pt-3 border-t border-slate-800/60">
          <div className="text-[11px] font-bold uppercase tracking-wider text-slate-400 px-3 mb-2">
            Evidence Vault
          </div>
          <div className="flex items-center justify-between px-3 py-1.5 text-xs text-slate-400 rounded-lg">
            <span className="flex items-center gap-2">
              <FileText className="w-3.5 h-3.5 text-emerald-400" /> Sec 65B Parquet
            </span>
            <span className="w-2 h-2 rounded-full bg-emerald-400" />
          </div>
          <div className="flex items-center justify-between px-3 py-1.5 text-xs text-slate-400 rounded-lg">
            <span className="flex items-center gap-2">
              <Sliders className="w-3.5 h-3.5 text-cyan-400" /> VRL Hot Reloading
            </span>
            <span className="text-[10px] text-slate-500 font-mono">ACTIVE</span>
          </div>
        </div>
      </div>

      {/* Bottom Area: Theme Switcher */}
      <div className="space-y-3 pt-3 border-t border-slate-800/60">
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
