import React, { useState } from 'react';
import { GEO_THREAT_POINTS } from '../mock/data';
import { ShieldAlert, Crosshair, MapPin } from 'lucide-react';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
}

export const WorldThreatMap: React.FC<Props> = ({ theme }) => {
  const [selectedPoint, setSelectedPoint] = useState<typeof GEO_THREAT_POINTS[0] | null>(GEO_THREAT_POINTS[0]);
  const isCyber = theme === 'cyber';

  return (
    <div className="relative w-full rounded-2xl border border-amber-500/30 bg-[#0d0e12] p-4 shadow-2xl overflow-hidden">
      {/* HUD Header */}
      <div className="flex items-center justify-between mb-3 border-b border-amber-500/20 pb-2">
        <div className="flex items-center gap-2">
          <Crosshair className="w-4 h-4 text-amber-400 animate-spin" style={{ animationDuration: '8s' }} />
          <span className="text-xs font-mono font-bold tracking-widest uppercase text-amber-400">
            GLOBAL TELEMETRY & ATTACK VECTOR RADAR
          </span>
        </div>
        <div className="flex items-center gap-3 text-[11px] font-mono text-slate-400">
          <span className="flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-rose-500 animate-ping" /> CRITICAL ORIGINS
          </span>
          <span className="text-amber-500/70">GRID: 8 SECTORS ACTIVE</span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 items-center">
        {/* SVG World Map Canvas (8 cols) */}
        <div className="lg:col-span-8 relative flex items-center justify-center min-h-[300px] bg-black/60 rounded-xl border border-amber-500/10 p-2 overflow-hidden">
          {/* Subtle radar scan lines */}
          <div className="absolute inset-0 pointer-events-none opacity-20 bg-[linear-gradient(rgba(245,158,11,0.05)_1px,transparent_1px),linear-gradient(90deg,rgba(245,158,11,0.05)_1px,transparent_1px)] bg-[size:24px_24px]" />

          <svg viewBox="0 0 1000 500" className="w-full h-auto select-none">
            <defs>
              {/* Gold gradient for continents */}
              <linearGradient id="cyberLandGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stopColor="#d97706" stopOpacity="0.4" />
                <stop offset="100%" stopColor="#78350f" stopOpacity="0.25" />
              </linearGradient>

              {/* Trajectory gradient */}
              <linearGradient id="attackArcGrad" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stopColor="#f43f5e" stopOpacity="0.8" />
                <stop offset="100%" stopColor="#fbbf24" stopOpacity="0.2" />
              </linearGradient>
            </defs>

            {/* Stylized Vector World Continents */}
            <g className="continents fill-[#d97706]/35 stroke-[#f59e0b]/40 stroke-[1.2]">
              {/* North America */}
              <path d="M 120 70 Q 240 60 280 120 Q 270 200 180 250 Q 150 220 110 150 Z" />
              <path d="M 210 110 L 260 110 L 250 140 Z" />
              {/* Greenland */}
              <path d="M 330 40 Q 380 40 370 80 Q 330 85 330 40 Z" />
              {/* South America */}
              <path d="M 270 260 Q 360 270 340 380 Q 290 440 260 410 Q 250 330 270 260 Z" />
              {/* Europe */}
              <path d="M 450 90 Q 550 80 540 160 Q 480 180 440 140 Z" />
              <path d="M 430 110 Q 450 100 450 125 Z" />
              {/* Africa */}
              <path d="M 450 190 Q 560 180 570 270 Q 520 380 480 370 Q 430 280 450 190 Z" />
              <path d="M 570 300 Q 590 300 580 340 Z" />
              {/* Asia & Russia */}
              <path d="M 550 70 Q 750 50 880 120 Q 840 230 720 220 Q 640 260 560 180 Z" />
              {/* India Subcontinent */}
              <path d="M 640 210 Q 710 210 680 300 Q 650 300 640 210 Z" fill="#f59e0b" fillOpacity="0.5" />
              {/* SE Asia & Japan */}
              <path d="M 740 230 Q 800 230 780 310 Q 730 270 740 230 Z" />
              <path d="M 830 150 Q 860 160 840 220 Z" />
              {/* Australia */}
              <path d="M 770 330 Q 890 320 870 410 Q 780 420 770 330 Z" />
            </g>

            {/* Central PRISM Defense Gateway in India (x: 670, y: 260) */}
            <g transform="translate(670, 260)">
              <circle r="16" fill="none" stroke="#10b981" strokeWidth="1.5" className="animate-ping" opacity="0.4" />
              <circle r="10" fill="#10b981" fillOpacity="0.2" stroke="#34d399" strokeWidth="2" />
              <circle r="4" fill="#34d399" />
              <text x="0" y="24" textAnchor="middle" className="text-[10px] font-mono font-bold fill-emerald-400">
                PRISM CORE
              </text>
            </g>

            {/* Animated Attack Arcs from threats to PRISM */}
            {GEO_THREAT_POINTS.map((pt) => {
              const startX = pt.x * 10;
              const startY = pt.y * 5;
              const endX = 670;
              const endY = 260;
              const midX = (startX + endX) / 2;
              const midY = Math.min(startY, endY) - 50;

              return (
                <path
                  key={`arc-${pt.id}`}
                  d={`M ${startX} ${startY} Q ${midX} ${midY} ${endX} ${endY}`}
                  fill="none"
                  stroke="url(#attackArcGrad)"
                  strokeWidth="1.8"
                  strokeDasharray="6 4"
                  className="animate-pulse"
                  opacity={pt.severity === 'Critical' ? 0.9 : 0.4}
                />
              );
            })}

            {/* Threat Origin Beacons */}
            {GEO_THREAT_POINTS.map((pt) => {
              const x = pt.x * 10;
              const y = pt.y * 5;
              const isSelected = selectedPoint?.id === pt.id;
              const beaconColor =
                pt.severity === 'Critical' ? '#f43f5e' : pt.severity === 'High' ? '#f59e0b' : '#38bdf8';

              return (
                <g
                  key={pt.id}
                  transform={`translate(${x}, ${y})`}
                  className="cursor-pointer group"
                  onClick={() => setSelectedPoint(pt)}
                >
                  {/* Pulsing ring */}
                  <circle r="12" fill="none" stroke={beaconColor} strokeWidth="1.5" className="animate-ping" opacity="0.6" />
                  <circle
                    r={isSelected ? 7 : 5}
                    fill={beaconColor}
                    stroke="#ffffff"
                    strokeWidth="1.5"
                    className="transition-all duration-200"
                    style={{ filter: `drop-shadow(0 0 8px ${beaconColor})` }}
                  />
                  {/* Tooltip on hover */}
                  <text
                    x="0"
                    y="-12"
                    textAnchor="middle"
                    className="text-[10px] font-mono font-bold fill-amber-300 opacity-0 group-hover:opacity-100 transition-opacity"
                  >
                    {pt.countryCode} ({pt.threatCount})
                  </text>
                </g>
              );
            })}
          </svg>
        </div>

        {/* Threat Inspector Sidebar (4 cols) */}
        <div className="lg:col-span-4 rounded-xl border border-amber-500/20 bg-black/60 p-4 space-y-4">
          <div className="flex items-center justify-between border-b border-amber-500/20 pb-2">
            <span className="text-xs font-mono font-bold text-amber-400 uppercase flex items-center gap-1.5">
              <ShieldAlert className="w-4 h-4 text-rose-500" /> Sector Details
            </span>
            <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-rose-500/20 text-rose-400 border border-rose-500/30">
              {selectedPoint?.severity} RISK
            </span>
          </div>

          {selectedPoint && (
            <div className="space-y-3 font-mono text-xs">
              <div className="flex justify-between">
                <span className="text-slate-400">Country:</span>
                <span className="text-amber-300 font-bold">{selectedPoint.country} ({selectedPoint.countryCode})</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Attacks Intercepted:</span>
                <span className="text-rose-400 font-bold">{selectedPoint.threatCount.toLocaleString()} EPS</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">Target Asset:</span>
                <span className="text-cyan-400 truncate max-w-[150px]">{selectedPoint.recentTarget}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-slate-400">OCSF Action:</span>
                <span className="text-emerald-400 font-bold">200 Denied & Vaulted</span>
              </div>
            </div>
          )}

          <div className="border-t border-amber-500/20 pt-3">
            <span className="text-[11px] font-mono text-slate-400 block mb-2">QUICK SECTOR FILTER:</span>
            <div className="flex flex-wrap gap-1.5">
              {GEO_THREAT_POINTS.map((pt) => (
                <button
                  key={pt.id}
                  onClick={() => setSelectedPoint(pt)}
                  className={`px-2 py-1 rounded text-[10px] font-mono transition-all ${
                    selectedPoint?.id === pt.id
                      ? 'bg-amber-500 text-black font-bold shadow-[0_0_8px_#f59e0b]'
                      : 'bg-slate-900 text-amber-400/80 border border-amber-500/20 hover:border-amber-400'
                  }`}
                >
                  {pt.countryCode}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
