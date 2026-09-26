import React, { useState, useMemo } from 'react';
import { SANKEY_NODES, SANKEY_LINKS } from '../mock/data';
import { Filter, X, Info } from 'lucide-react';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
}

export const SankeyDiagram: React.FC<Props> = ({ theme }) => {
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [hoveredLink, setHoveredLink] = useState<{ source: string; target: string } | null>(null);

  const width = 1100;
  const height = 480;
  const paddingX = 40;
  const paddingY = 30;
  const nodeWidth = 24;

  const col0X = paddingX + 90;
  const col1X = width / 2 - nodeWidth / 2;
  const col2X = width - paddingX - nodeWidth - 90;

  // Process nodes & calculate positions
  const { nodeMap, colNodes } = useMemo(() => {
    const col0 = SANKEY_NODES.filter((n) => n.col === 0);
    const col1 = SANKEY_NODES.filter((n) => n.col === 1);
    const col2 = SANKEY_NODES.filter((n) => n.col === 2);

    const calcColPositions = (nodes: typeof SANKEY_NODES, x: number) => {
      const totalVal = nodes.reduce((acc, n) => acc + n.value, 0);
      const availableHeight = height - paddingY * 2 - (nodes.length - 1) * 12;
      let currY = paddingY;

      return nodes.map((node) => {
        const nodeHeight = Math.max(16, (node.value / totalVal) * availableHeight);
        const y = currY;
        currY += nodeHeight + 12;
        return {
          ...node,
          x,
          y,
          h: nodeHeight,
          w: nodeWidth,
        };
      });
    };

    const c0 = calcColPositions(col0, col0X);
    const c1 = calcColPositions(col1, col1X);
    const c2 = calcColPositions(col2, col2X);

    const map = new Map<string, (typeof c0)[0]>();
    [...c0, ...c1, ...c2].forEach((n) => map.set(n.id, n));

    return {
      nodeMap: map,
      colNodes: { c0, c1, c2 },
    };
  }, [width, height, col0X, col1X, col2X]);

  // Compute Bezier paths for links
  const linksWithPaths = useMemo(() => {
    const sourceOffsets = new Map<string, number>();
    const targetOffsets = new Map<string, number>();

    return SANKEY_LINKS.map((link) => {
      const src = nodeMap.get(link.source);
      const tgt = nodeMap.get(link.target);
      if (!src || !tgt) return null;

      const srcOff = sourceOffsets.get(link.source) || 0;
      const tgtOff = targetOffsets.get(link.target) || 0;

      const srcFraction = link.value / src.value;
      const tgtFraction = link.value / tgt.value;
      const ribbonHeightSrc = Math.max(2, srcFraction * src.h);
      const ribbonHeightTgt = Math.max(2, tgtFraction * tgt.h);
      const ribbonH = Math.max(2, Math.min(ribbonHeightSrc, ribbonHeightTgt));

      const x0 = src.x + src.w;
      const y0 = src.y + srcOff + ribbonH / 2;
      const x1 = tgt.x;
      const y1 = tgt.y + tgtOff + ribbonH / 2;

      sourceOffsets.set(link.source, srcOff + ribbonHeightSrc);
      targetOffsets.set(link.target, tgtOff + ribbonHeightTgt);

      const dx = (x1 - x0) * 0.5;
      const path = `M ${x0} ${y0} C ${x0 + dx} ${y0}, ${x1 - dx} ${y1}, ${x1} ${y1}`;

      return {
        ...link,
        path,
        strokeWidth: Math.max(2, ribbonH),
        x0,
        y0,
        x1,
        y1,
      };
    }).filter(Boolean);
  }, [nodeMap]);

  const activeFocus = selectedNode || hoveredNode;

  const isDark = theme === 'dark';
  const isCyber = theme === 'cyber';

  return (
    <div className="relative w-full overflow-x-auto rounded-2xl border border-slate-700/50 bg-[#0c101c]/90 p-5 shadow-2xl backdrop-blur-md font-sans">
      {/* Top Filter & Explanatory Banner */}
      <div className="flex flex-wrap items-center justify-between gap-3 mb-4 border-b border-slate-800 pb-3">
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono font-bold text-cyan-400 uppercase tracking-wider flex items-center gap-1.5">
            <Info className="w-4 h-4" /> Understanding Traffic Flow:
          </span>
          <span className="text-xs text-slate-300">
            Raw logs stream left-to-right from firewall origins into PRISM normalization engines, emerging as OCSF Class 4001 validated events.
          </span>
        </div>

        {selectedNode && (
          <div className="flex items-center gap-2 text-xs font-mono bg-cyan-950/80 text-cyan-300 px-3 py-1 rounded-xl border border-cyan-500/40">
            <span>Filter Active: <strong>{selectedNode}</strong></span>
            <button
              onClick={() => setSelectedNode(null)}
              className="p-0.5 hover:text-white rounded transition"
              title="Clear Filter"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        )}
      </div>

      {/* Pipeline Stage Column Headers */}
      <div className="grid grid-cols-3 text-center text-xs font-bold tracking-wider uppercase mb-3">
        <div className={isCyber ? 'text-amber-400' : isDark ? 'text-slate-300' : 'text-slate-700'}>
          Destination / Origin
        </div>
        <div className={isCyber ? 'text-amber-400' : isDark ? 'text-slate-300' : 'text-slate-700'}>
          PRISM Ingestion & Parser Plane
        </div>
        <div className={isCyber ? 'text-amber-400' : isDark ? 'text-slate-300' : 'text-slate-700'}>
          OCSF Compliance & Egress Status
        </div>
      </div>

      <svg
        viewBox={`0 0 ${width} ${height}`}
        className="w-full h-auto select-none overflow-visible"
        style={{ minWidth: '800px' }}
      >
        <defs>
          <linearGradient id="cyanRibbon" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#0ea5e9" stopOpacity="0.6" />
            <stop offset="100%" stopColor="#10b981" stopOpacity="0.8" />
          </linearGradient>
          <linearGradient id="amberRibbon" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#f59e0b" stopOpacity="0.5" />
            <stop offset="100%" stopColor="#f97316" stopOpacity="0.8" />
          </linearGradient>
          <linearGradient id="roseRibbon" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#ec4899" stopOpacity="0.5" />
            <stop offset="100%" stopColor="#f43f5e" stopOpacity="0.8" />
          </linearGradient>
        </defs>

        {/* Links / Ribbons */}
        <g className="links">
          {linksWithPaths.map((link, idx) => {
            if (!link) return null;
            const isHighlighted =
              activeFocus === link.source ||
              activeFocus === link.target ||
              (hoveredLink?.source === link.source && hoveredLink?.target === link.target);
            const isDimmed =
              (activeFocus && activeFocus !== link.source && activeFocus !== link.target) ||
              (hoveredLink && (hoveredLink.source !== link.source || hoveredLink.target !== link.target));

            const strokeColor =
              link.target === 'Status_200'
                ? 'url(#cyanRibbon)'
                : link.target === 'Status_404'
                ? 'url(#amberRibbon)'
                : link.target === 'Status_503'
                ? 'url(#roseRibbon)'
                : isCyber
                ? 'rgba(245, 158, 11, 0.4)'
                : 'rgba(14, 165, 233, 0.35)';

            return (
              <path
                key={`link-${idx}`}
                d={link.path}
                fill="none"
                stroke={strokeColor}
                strokeWidth={link.strokeWidth}
                strokeLinecap="round"
                className="transition-all duration-300 cursor-pointer"
                style={{
                  opacity: isHighlighted ? 1 : isDimmed ? 0.08 : 0.65,
                  filter: isHighlighted ? 'drop-shadow(0 0 10px rgba(16, 185, 129, 0.8))' : 'none',
                }}
                onMouseEnter={() => setHoveredLink({ source: link.source, target: link.target })}
                onMouseLeave={() => setHoveredLink(null)}
              >
                <title>{`${link.source} → ${link.target}: ${link.value.toLocaleString()} events`}</title>
              </path>
            );
          })}
        </g>

        {/* Column 0: Origin Nodes */}
        <g className="nodes-col0">
          {colNodes.c0.map((node) => {
            const isSelected = selectedNode === node.id;
            const isHovered = activeFocus === node.id;
            return (
              <g
                key={node.id}
                className="cursor-pointer group"
                onClick={() => setSelectedNode(isSelected ? null : node.id)}
                onMouseEnter={() => setHoveredNode(node.id)}
                onMouseLeave={() => setHoveredNode(null)}
              >
                <rect
                  x={node.x}
                  y={node.y}
                  width={node.w}
                  height={node.h}
                  rx={4}
                  fill={node.color}
                  className="transition-all duration-200"
                  style={{
                    filter: isHovered || isSelected ? `drop-shadow(0 0 12px ${node.color})` : 'none',
                    opacity: activeFocus && !isHovered ? 0.35 : 1,
                  }}
                />
                {/* Text Label on Left */}
                <text
                  x={node.x - 10}
                  y={node.y + node.h / 2}
                  textAnchor="end"
                  dominantBaseline="middle"
                  className={`text-[11px] font-medium transition-colors ${
                    isCyber
                      ? 'fill-amber-200 group-hover:fill-amber-400'
                      : isDark
                      ? 'fill-slate-300 group-hover:fill-cyan-400'
                      : 'fill-slate-700 group-hover:fill-slate-900'
                  }`}
                >
                  <tspan className="mr-1">{node.flag} </tspan>
                  <tspan className="font-bold">{node.id}</tspan>
                  <tspan className="text-[10px] opacity-75 font-mono"> · {node.value.toLocaleString()}</tspan>
                </text>
              </g>
            );
          })}
        </g>

        {/* Column 1: Engine Nodes */}
        <g className="nodes-col1">
          {colNodes.c1.map((node) => {
            const isHovered = activeFocus === node.id;
            return (
              <g
                key={node.id}
                className="cursor-pointer group"
                onClick={() => setSelectedNode(selectedNode === node.id ? null : node.id)}
                onMouseEnter={() => setHoveredNode(node.id)}
                onMouseLeave={() => setHoveredNode(null)}
              >
                <rect
                  x={node.x}
                  y={node.y}
                  width={node.w}
                  height={node.h}
                  rx={4}
                  fill={isCyber ? '#d97706' : '#0284c7'}
                  className="transition-all duration-200"
                  style={{
                    filter: isHovered ? 'drop-shadow(0 0 12px #38bdf8)' : 'none',
                    opacity: activeFocus && !isHovered ? 0.35 : 1,
                  }}
                />
                {/* Number inside */}
                <text
                  x={node.x + node.w / 2}
                  y={node.y + node.h / 2}
                  textAnchor="middle"
                  dominantBaseline="middle"
                  className="text-[10px] font-mono font-bold fill-white pointer-events-none drop-shadow"
                >
                  {node.value.toLocaleString()}
                </text>
                {/* Node Title above */}
                <text
                  x={node.x + node.w / 2}
                  y={node.y - 6}
                  textAnchor="middle"
                  className={`text-[10px] font-bold tracking-tight transition-colors ${
                    isCyber ? 'fill-amber-300' : isDark ? 'fill-slate-200' : 'fill-slate-800'
                  }`}
                >
                  {node.label}
                </text>
              </g>
            );
          })}
        </g>

        {/* Column 2: Status Egress Nodes */}
        <g className="nodes-col2">
          {colNodes.c2.map((node) => {
            const isHovered = activeFocus === node.id;
            return (
              <g
                key={node.id}
                className="cursor-pointer group"
                onClick={() => setSelectedNode(selectedNode === node.id ? null : node.id)}
                onMouseEnter={() => setHoveredNode(node.id)}
                onMouseLeave={() => setHoveredNode(null)}
              >
                <rect
                  x={node.x}
                  y={node.y}
                  width={node.w}
                  height={node.h}
                  rx={4}
                  fill={node.color}
                  className="transition-all duration-200"
                  style={{
                    filter: isHovered ? `drop-shadow(0 0 14px ${node.color})` : 'none',
                    opacity: activeFocus && !isHovered ? 0.35 : 1,
                  }}
                />
                {/* Text Label on Right */}
                <text
                  x={node.x + node.w + 10}
                  y={node.y + node.h / 2}
                  textAnchor="start"
                  dominantBaseline="middle"
                  className={`text-[11px] font-medium transition-colors ${
                    isCyber
                      ? 'fill-amber-200 group-hover:fill-amber-400'
                      : isDark
                      ? 'fill-slate-200 group-hover:fill-emerald-400'
                      : 'fill-slate-800 group-hover:fill-slate-900'
                  }`}
                >
                  <tspan className="font-extrabold">{node.label.split('·')[0]}</tspan>
                  <tspan className="text-[10px] opacity-80"> {node.label.split('·')[1] || ''}</tspan>
                  <tspan className="block text-[10px] font-mono font-bold fill-emerald-400">
                    {' '}({node.value.toLocaleString()} events)
                  </tspan>
                </text>
              </g>
            );
          })}
        </g>
      </svg>

      {/* Sankey Footer Instructions */}
      <div className="mt-4 flex flex-wrap items-center justify-between text-xs text-slate-400 border-t border-slate-800/80 pt-3 px-2">
        <span className="flex items-center gap-2">
          <span className="inline-block w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse" />
          <span>Interactive: Click or hover any node/ribbon to inspect exact packet routing and volume</span>
        </span>
        <span className="text-slate-500 font-mono text-[11px]">
          OCSF Class 4001: 96.2% allowed • 3.8% filtered to DLQ
        </span>
      </div>
    </div>
  );
};
