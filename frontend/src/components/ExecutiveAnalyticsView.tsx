import React, { useState } from 'react';
import { ArrowDownRight, ArrowUpRight, TrendingUp, Radio, Info } from 'lucide-react';
import {
  DEVICE_BREAKDOWN,
  REALTIME_BARS,
  SITE_TRAFFIC_SERIES,
  VENDOR_TRAFFIC_DONUT,
  TIME_ON_SITE_BARS,
  TOP_PAGES_RANKED,
} from '../mock/data';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  isLiveStreaming: boolean;
}

export const ExecutiveAnalyticsView: React.FC<Props> = ({ theme, isLiveStreaming }) => {
  const [hoveredMonth, setHoveredMonth] = useState<number | null>(7); // Default to Aug (peak index 7)
  const [timeframe, setTimeframe] = useState<'week' | 'month' | 'year'>('week');
  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  // Helpers for SVG Sparklines
  const renderSparkline = (data: number[], isPositive: boolean) => {
    const min = Math.min(...data);
    const max = Math.max(...data);
    const range = max - min || 1;
    const w = 120;
    const h = 42;
    const points = data.map((val, i) => {
      const x = (i / (data.length - 1)) * w;
      const y = h - ((val - min) / range) * (h - 8) - 4;
      return `${x},${y}`;
    });
    const strokeColor = isPositive ? '#10b981' : '#f43f5e';
    const fillColor = isPositive ? 'rgba(16, 185, 129, 0.15)' : 'rgba(244, 63, 94, 0.15)';
    const areaPoints = [`0,${h}`, ...points, `${w},${h}`].join(' ');

    return (
      <svg width={w} height={h} className="overflow-visible">
        <polygon points={areaPoints} fill={fillColor} />
        <polyline
          fill="none"
          stroke={strokeColor}
          strokeWidth="2.5"
          strokeLinecap="round"
          strokeLinejoin="round"
          points={points.join(' ')}
        />
        {data.length > 0 && (
          <circle
            cx={w}
            cy={h - ((data[data.length - 1] - min) / range) * (h - 8) - 4}
            r="3.5"
            fill={strokeColor}
            className="animate-pulse"
          />
        )}
      </svg>
    );
  };

  // Spline Chart calculations (Jan - Dec)
  const chartW = 760;
  const chartH = 220;
  const maxTrafficVal = 850;
  const getY = (val: number) => chartH - (val / maxTrafficVal) * (chartH - 40) - 20;
  const getX = (idx: number) => 40 + (idx / (SITE_TRAFFIC_SERIES.length - 1)) * (chartW - 80);

  const createSmoothPath = (key: 'newVisitors' | 'returning') => {
    const points = SITE_TRAFFIC_SERIES.map((d, i) => ({ x: getX(i), y: getY(d[key]) }));
    if (points.length < 2) return '';
    let path = `M ${points[0].x} ${points[0].y}`;
    for (let i = 0; i < points.length - 1; i++) {
      const p0 = points[i];
      const p1 = points[i + 1];
      const cx1 = p0.x + (p1.x - p0.x) / 2;
      const cy1 = p0.y;
      const cx2 = p0.x + (p1.x - p0.x) / 2;
      const cy2 = p1.y;
      path += ` C ${cx1} ${cy1}, ${cx2} ${cy2}, ${p1.x} ${p1.y}`;
    }
    return path;
  };

  const newVisitorsPath = createSmoothPath('newVisitors');
  const returningPath = createSmoothPath('returning');
  const newVisitorsArea = `${newVisitorsPath} L ${getX(SITE_TRAFFIC_SERIES.length - 1)} ${chartH} L ${getX(0)} ${chartH} Z`;
  const returningArea = `${returningPath} L ${getX(SITE_TRAFFIC_SERIES.length - 1)} ${chartH} L ${getX(0)} ${chartH} Z`;

  return (
    <div className="space-y-6">
      {/* Top Explanatory Banner for C-Level View */}
      <div
        className={`p-4 rounded-2xl border text-xs leading-relaxed flex items-center justify-between gap-4 ${
          isDark
            ? 'bg-[#141724]/70 border-slate-800 text-slate-300'
            : 'bg-white/90 border-slate-200 text-slate-700 shadow-sm'
        }`}
      >
        <div className="flex items-center gap-2">
          <Info className="w-4 h-4 text-purple-400 shrink-0" />
          <span>
            <strong>Executive Analytics Perspective:</strong> Real-time overview of network session EPS, active sensor endpoints, zero-allocation microsecond parse latency, and firewall vendor distributions.
          </span>
        </div>
        <div className="flex items-center gap-1 shrink-0 font-mono text-[11px]">
          <span className="text-slate-400">Timeframe:</span>
          {(['week', 'month', 'year'] as const).map((t) => (
            <button
              key={t}
              onClick={() => setTimeframe(t)}
              className={`px-2 py-0.5 rounded capitalize ${
                timeframe === t ? 'bg-purple-600 text-white font-bold' : 'text-slate-400 hover:text-white'
              }`}
            >
              {t}
            </button>
          ))}
        </div>
      </div>

      {/* Top Row: 3 Ingestion KPI Cards with Sparklines + 1 Protocol Donut */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* KPI 1: Sessions */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium">
            <span className="font-semibold text-slate-300">Sessions (EPS)</span>
            <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">Last week ▾</span>
          </div>
          <div className="mt-3 flex items-baseline justify-between">
            <div>
              <span className="text-3xl font-extrabold tracking-tight">856</span>
              <span className="text-xs text-slate-400 ml-1">k events/s</span>
            </div>
            <div className="flex items-center gap-1 text-xs font-semibold text-rose-500">
              <ArrowDownRight className="w-4 h-4" />
              <span>-12%</span>
            </div>
          </div>
          <div className="mt-1 text-[11px] text-slate-400 leading-tight">
            Ingestion throughput across 12 worker threads
          </div>
          <div className="mt-3 flex justify-end">{renderSparkline([98, 92, 95, 89, 91, 84, 86], false)}</div>
        </div>

        {/* KPI 2: Users */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium">
            <span className="font-semibold text-slate-300">Sensor Nodes</span>
            <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">Last week ▾</span>
          </div>
          <div className="mt-3 flex items-baseline justify-between">
            <div>
              <span className="text-3xl font-extrabold tracking-tight">523</span>
              <span className="text-xs text-slate-400 ml-1">online</span>
            </div>
            <div className="flex items-center gap-1 text-xs font-semibold text-emerald-400">
              <ArrowUpRight className="w-4 h-4" />
              <span>+23%</span>
            </div>
          </div>
          <div className="mt-1 text-[11px] text-slate-400 leading-tight">
            Active firewalls streaming syslog over UDP/TCP
          </div>
          <div className="mt-3 flex justify-end">{renderSparkline([410, 425, 460, 470, 490, 510, 523], true)}</div>
        </div>

        {/* KPI 3: Time spent */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium">
            <span className="font-semibold text-slate-300">Parse Latency (p99)</span>
            <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">Last week ▾</span>
          </div>
          <div className="mt-3 flex items-baseline justify-between">
            <div>
              <span className="text-3xl font-extrabold tracking-tight">9.56</span>
              <span className="text-xs text-slate-400 ml-1">µs</span>
            </div>
            <div className="flex items-center gap-1 text-xs font-semibold text-cyan-400">
              <ArrowUpRight className="w-4 h-4" />
              <span>+8%</span>
            </div>
          </div>
          <div className="mt-1 text-[11px] text-slate-400 leading-tight">
            Zero-alloc VRL router runtime (well under 25µs SLA)
          </div>
          <div className="mt-3 flex justify-end">{renderSparkline([7.2, 8.1, 7.8, 8.9, 8.4, 9.1, 9.56], true)}</div>
        </div>

        {/* KPI 4: Devices Donut */}
        <div
          className={`rounded-2xl border p-5 relative overflow-hidden transition-all duration-300 shadow-md flex flex-col justify-between ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium">
            <span className="font-semibold text-slate-300">Transport Protocols</span>
            <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">Today ▾</span>
          </div>

          <div className="my-2 flex items-center justify-around">
            {/* Donut ring */}
            <div className="relative w-20 h-20 flex items-center justify-center">
              <svg viewBox="0 0 100 100" className="w-full h-full -rotate-90">
                <circle cx="50" cy="50" r="38" fill="none" stroke="#f97316" strokeWidth="12" strokeDasharray="117 239" />
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#38bdf8"
                  strokeWidth="12"
                  strokeDasharray="86 239"
                  strokeDashoffset="-117"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#a855f7"
                  strokeWidth="12"
                  strokeDasharray="36 239"
                  strokeDashoffset="-203"
                />
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center">
                <span className="text-[10px] uppercase text-slate-400 font-bold">Total</span>
                <span className="text-sm font-extrabold">{DEVICE_BREAKDOWN.total}</span>
              </div>
            </div>

            {/* Legend */}
            <div className="space-y-1 text-[11px]">
              <div className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-[#f97316]" />
                <span className="text-slate-400">UDP Syslog</span>
                <span className="font-semibold text-slate-200 ml-auto">49%</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-[#38bdf8]" />
                <span className="text-slate-400">TCP / TLS</span>
                <span className="font-semibold text-slate-200 ml-auto">36%</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-[#a855f7]" />
                <span className="text-slate-400">NetFlow v9</span>
                <span className="font-semibold text-slate-200 ml-auto">15%</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Middle Row: Real-time Data (Left) + Site Traffic Spline Chart (Right) */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Real-time Data Card (4 cols) */}
        <div
          className={`lg:col-span-4 rounded-2xl border p-5 transition-all duration-300 shadow-md space-y-4 ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between">
            <h3 className="text-base font-bold tracking-tight">Real-time Buffer Gauges</h3>
            <span className="flex items-center gap-1 text-[11px] text-emerald-400 font-semibold">
              <Radio className="w-3.5 h-3.5 animate-pulse" /> Live
            </span>
          </div>

          <div className="space-y-4 pt-1">
            {REALTIME_BARS.map((item, idx) => {
              const pct = Math.round((item.current / item.max) * 100);
              return (
                <div key={idx} className="space-y-1.5">
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-slate-400 font-medium">{item.label}</span>
                    <div className="flex items-baseline gap-1.5">
                      <span className="font-bold text-slate-100 text-sm">{item.current}</span>
                      <span className="text-[10px] text-slate-500">max {item.max}</span>
                    </div>
                  </div>
                  <div className="w-full h-2 rounded-full bg-slate-800/80 overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all duration-500"
                      style={{
                        width: `${pct}%`,
                        backgroundColor: item.color,
                        boxShadow: `0 0 8px ${item.color}88`,
                      }}
                    />
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Site Traffic Spline Chart (8 cols) */}
        <div
          className={`lg:col-span-8 rounded-2xl border p-5 transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex flex-wrap items-center justify-between gap-3 mb-4">
            <div>
              <h3 className="text-base font-bold tracking-tight">Site Traffic · Ingestion Telemetry</h3>
              <p className="text-xs text-slate-400">Comparing Raw Ingested Events vs Validated OCSF Events</p>
            </div>
            <div className="flex items-center gap-4 text-xs font-medium">
              <span className="flex items-center gap-1.5 text-slate-400">
                <span className="w-2.5 h-2.5 rounded-full bg-[#a855f7]" /> Raw Ingested (EPS)
              </span>
              <span className="flex items-center gap-1.5 text-slate-400">
                <span className="w-2.5 h-2.5 rounded-full bg-[#06b6d4]" /> Normalized OCSF
              </span>
              <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">
                This year ▾
              </span>
            </div>
          </div>

          {/* Interactive SVG Spline Area Chart */}
          <div className="relative w-full overflow-hidden">
            <svg viewBox={`0 0 ${chartW} ${chartH}`} className="w-full h-auto overflow-visible select-none">
              <defs>
                <linearGradient id="purpleGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#a855f7" stopOpacity="0.35" />
                  <stop offset="100%" stopColor="#a855f7" stopOpacity="0.0" />
                </linearGradient>
                <linearGradient id="cyanGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor="#06b6d4" stopOpacity="0.35" />
                  <stop offset="100%" stopColor="#06b6d4" stopOpacity="0.0" />
                </linearGradient>
              </defs>

              {/* Grid lines */}
              {[200, 400, 600, 800, 1000].map((val) => {
                const y = getY(val);
                return (
                  <g key={val}>
                    <line x1="30" y1={y} x2={chartW - 30} y2={y} stroke="#334155" strokeDasharray="3 3" opacity="0.3" />
                    <text x="25" y={y + 3} textAnchor="end" className="text-[9px] fill-slate-500 font-mono">
                      {val}k
                    </text>
                  </g>
                );
              })}

              <path d={newVisitorsArea} fill="url(#purpleGrad)" />
              <path d={returningArea} fill="url(#cyanGrad)" />

              <path d={newVisitorsPath} fill="none" stroke="#a855f7" strokeWidth="3" />
              <path d={returningPath} fill="none" stroke="#06b6d4" strokeWidth="3" />

              {/* Points & Interactive Tooltip */}
              {SITE_TRAFFIC_SERIES.map((d, i) => {
                const x = getX(i);
                const yNew = getY(d.newVisitors);
                const isHovered = hoveredMonth === i;

                return (
                  <g key={d.month} className="cursor-pointer" onMouseEnter={() => setHoveredMonth(i)}>
                    {isHovered && (
                      <line x1={x} y1="10" x2={x} y2={chartH} stroke="#94a3b8" strokeWidth="1" strokeDasharray="2 2" />
                    )}

                    <circle cx={x} cy={yNew} r={isHovered ? 6 : 3.5} fill="#a855f7" stroke="#ffffff" strokeWidth="2" />

                    {isHovered && (
                      <g transform={`translate(${x}, ${yNew - 40})`}>
                        <rect
                          x="-35"
                          y="-18"
                          width="70"
                          height="32"
                          rx="6"
                          fill="#0f172a"
                          stroke="#64748b"
                          strokeWidth="1"
                          filter="drop-shadow(0 4px 6px rgba(0,0,0,0.5))"
                        />
                        <text x="0" y="-3" textAnchor="middle" className="text-[11px] font-bold fill-white">
                          {d.newVisitors}k EPS
                        </text>
                        <text x="0" y="8" textAnchor="middle" className="text-[9px] font-medium fill-slate-400">
                          {d.month} 16
                        </text>
                      </g>
                    )}

                    <text x={x} y={chartH + 15} textAnchor="middle" className="text-[10px] fill-slate-400">
                      {d.month}
                    </text>
                  </g>
                );
              })}
            </svg>
          </div>
        </div>
      </div>

      {/* Bottom Row: Traffic from Vendors + Latency Distribution + Top Rules */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-12 gap-6">
        {/* 1. Firewall Vendor Breakdown (4 cols) */}
        <div
          className={`lg:col-span-4 rounded-2xl border p-5 transition-all duration-300 shadow-md flex flex-col justify-between ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium">
            <span className="font-bold text-slate-100 text-sm">Traffic from Firewalls</span>
            <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">Last week ▾</span>
          </div>

          <div className="my-4 flex items-center justify-around">
            <div className="relative w-28 h-28 flex items-center justify-center">
              <svg viewBox="0 0 100 100" className="w-full h-full -rotate-90">
                <circle cx="50" cy="50" r="38" fill="none" stroke="#ef4444" strokeWidth="14" strokeDasharray="107 239" />
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#f97316"
                  strokeWidth="14"
                  strokeDasharray="48 239"
                  strokeDashoffset="-107"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#eab308"
                  strokeWidth="14"
                  strokeDasharray="36 239"
                  strokeDashoffset="-155"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#3b82f6"
                  strokeWidth="14"
                  strokeDasharray="26 239"
                  strokeDashoffset="-191"
                />
                <circle
                  cx="50"
                  cy="50"
                  r="38"
                  fill="none"
                  stroke="#06b6d4"
                  strokeWidth="14"
                  strokeDasharray="22 239"
                  strokeDashoffset="-217"
                />
              </svg>
              <div className="absolute inset-0 flex flex-col items-center justify-center">
                <span className="text-[10px] uppercase text-slate-400 font-bold">Total</span>
                <span className="text-base font-extrabold">{VENDOR_TRAFFIC_DONUT.total}</span>
              </div>
            </div>

            <div className="space-y-1.5 text-xs">
              {VENDOR_TRAFFIC_DONUT.items.map((item) => (
                <div key={item.label} className="flex items-center gap-2">
                  <span className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: item.color }} />
                  <span className="text-slate-400 truncate max-w-[90px]">{item.label}</span>
                  <span className="font-semibold text-slate-200 ml-auto">
                    {item.count} / {item.percentage}%
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* 2. Latency Distribution (Bar Chart) (4 cols) */}
        <div
          className={`lg:col-span-4 rounded-2xl border p-5 transition-all duration-300 shadow-md flex flex-col justify-between ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium">
            <span className="font-bold text-slate-100 text-sm">Parse Latency Distribution</span>
            <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">Last 14 days ▾</span>
          </div>

          <div className="mt-4 flex items-end justify-between h-44 px-2">
            {TIME_ON_SITE_BARS.map((bar, idx) => {
              const heightPct = Math.round((bar.value / 14) * 100);
              return (
                <div key={idx} className="flex flex-col items-center gap-1 group relative">
                  {bar.isPeak && (
                    <div className="absolute -top-7 px-1.5 py-0.5 rounded bg-slate-800 border border-slate-600 text-[10px] font-bold text-white shadow">
                      {bar.value} µs
                    </div>
                  )}
                  <div
                    className="w-3 sm:w-3.5 rounded-t-lg transition-all duration-300 group-hover:opacity-100"
                    style={{
                      height: `${heightPct}%`,
                      backgroundColor: bar.isPeak ? '#aa3bff' : '#6d28d9',
                      opacity: bar.isPeak ? 1 : 0.6,
                      boxShadow: bar.isPeak ? '0 0 10px rgba(170, 59, 255, 0.6)' : 'none',
                    }}
                  />
                  <span className="text-[9px] text-slate-500 font-mono mt-1">{bar.day.split(' ')[0]}</span>
                </div>
              );
            })}
          </div>
        </div>

        {/* 3. Top Rules Ranked (4 cols) */}
        <div
          className={`lg:col-span-4 rounded-2xl border p-5 transition-all duration-300 shadow-md ${
            isDark ? 'bg-[#181a26]/90 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between text-xs text-slate-400 font-medium mb-3">
            <span className="font-bold text-slate-100 text-sm">Top Ingestion Rules</span>
            <span className="text-[11px] bg-slate-800/60 px-2 py-0.5 rounded border border-slate-700/50">Last month ▾</span>
          </div>

          <div className="space-y-2.5 max-h-48 overflow-y-auto pr-1">
            {TOP_PAGES_RANKED.map((page, idx) => (
              <div key={idx} className="space-y-1">
                <div className="flex items-center justify-between text-xs">
                  <span className="text-slate-300 font-mono truncate max-w-[170px]">{page.path}</span>
                  <span className="font-semibold text-slate-200">{page.count.toLocaleString()}</span>
                </div>
                <div className="w-full h-1.5 rounded-full bg-slate-800/80 overflow-hidden">
                  <div
                    className="h-full rounded-full transition-all duration-500"
                    style={{ width: `${page.percentage}%`, backgroundColor: page.color }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
