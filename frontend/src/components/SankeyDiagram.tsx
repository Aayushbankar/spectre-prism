import React, { useState, useMemo } from 'react';
import { SANKEY_NODES, SANKEY_LINKS } from '../mock/data';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
}

export const SankeyDiagram: React.FC<Props> = ({ theme }) => {
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
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
    // To position link endpoints nicely along the node height, keep track of offsets
    const sourceOffsets = new Map<string, number>();
    const targetOffsets = new Map<string, number>();

    return SANKEY_LINKS.map((link) => {
      const src = nodeMap.get(link.source);
      const tgt = nodeMap.get(link.target);
      if (!src || !tgt) return null;

      const srcOff = sourceOffsets.get(link.source) || 0;
      const tgtOff = targetOffsets.get(link.target) || 0;

      // Link ribbon height proportional to value
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

      // Cubic Bezier curve
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

  const isDark = theme === 'dark';
  const isCyber = theme === 'cyber';

  return (
    <div className="relative w-full overflow-x-auto rounded-xl border border-slate-700/50 bg-slate-900/60 p-4 shadow-2xl backdrop-blur-md">
      {/* Column Headers */}
      <div className="grid grid-cols-3 text-center text-xs font-semibold tracking-wider uppercase mb-2">
        <div className={isCyber ? 'text-amber-400' : isDark ? 'text-slate-400' : 'text-slate-600'}>
          Destination / Origin
        </div>
        <div className={isCyber ? 'text-amber-400' : isDark ? 'text-slate-400' : 'text-slate-600'}>
          PRISM Engine / Parser Plane
        </div>
        <div className={isCyber ? 'text-amber-400' : isDark ? 'text-slate-400' : 'text-slate-600'}>
          Egress / OCSF Compliance Status
        </div>
      </div>

      <svg
        viewBox={`0 0 ${width} ${height}`}
        className="w-full h-auto select-none overflow-visible"
        style={{ minWidth: '750px' }}
      >
        <defs>
          <linearGradient id="cyanRibbon" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#0ea5e9" stopOpacity="0.6" />
            <stop offset="100%" stopColor="#10b981" stopOpacity="0.75" />
          </linearGradient>
          <linearGradient id="amberRibbon" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#f59e0b" stopOpacity="0.5" />
            <stop offset="100%" stopColor="#f97316" stopOpacity="0.75" />
          </linearGradient>
          <linearGradient id="roseRibbon" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#ec4899" stopOpacity="0.5" />
            <stop offset="100%" stopColor="#f43f5e" stopOpacity="0.75" />
          </linearGradient>
        </defs>

        {/* Links / Ribbons */}
        <g className="links">
          {linksWithPaths.map((link, idx) => {
            if (!link) return null;
            const isHighlighted =
              hoveredNode === link.source ||
              hoveredNode === link.target ||
              (hoveredLink?.source === link.source && hoveredLink?.target === link.target);
            const isDimmed =
              (hoveredNode && hoveredNode !== link.source && hoveredNode !== link.target) ||
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
                  opacity: isHighlighted ? 1 : isDimmed ? 0.12 : 0.65,
                  filter: isHighlighted ? 'drop-shadow(0 0 8px rgba(16, 185, 129, 0.7))' : 'none',
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
            const isHovered = hoveredNode === node.id;
            return (
              <g
                key={node.id}
                className="cursor-pointer group"
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
                    filter: isHovered ? `drop-shadow(0 0 10px ${node.color})` : 'none',
                    opacity: hoveredNode && !isHovered ? 0.4 : 1,
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
                  <tspan className="font-semibold">{node.id}</tspan>
                  <tspan className="text-[10px] opacity-70"> · {node.value.toLocaleString()}</tspan>
                </text>
              </g>
            );
          })}
        </g>

        {/* Column 1: Engine Nodes */}
        <g className="nodes-col1">
          {colNodes.c1.map((node) => {
            const isHovered = hoveredNode === node.id;
            return (
              <g
                key={node.id}
                className="cursor-pointer group"
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
                    filter: isHovered ? 'drop-shadow(0 0 10px #38bdf8)' : 'none',
                    opacity: hoveredNode && !isHovered ? 0.4 : 1,
                  }}
                />
                {/* Node Label in Middle */}
                <text
                  x={node.x + node.w / 2}
                  y={node.y + node.h / 2}
                  textAnchor="middle"
                  dominantBaseline="middle"
                  className="text-[10px] font-semibold fill-white pointer-events-none drop-shadow"
                >
                  {node.value.toLocaleString()}
                </text>
                <text
                  x={node.x + node.w / 2}
                  y={node.y - 6}
                  textAnchor="middle"
                  className={`text-[10px] font-medium transition-colors ${
                    isCyber ? 'fill-amber-300' : isDark ? 'fill-slate-300' : 'fill-slate-700'
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
            const isHovered = hoveredNode === node.id;
            return (
              <g
                key={node.id}
                className="cursor-pointer group"
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
                    filter: isHovered ? `drop-shadow(0 0 12px ${node.color})` : 'none',
                    opacity: hoveredNode && !isHovered ? 0.4 : 1,
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
                  <tspan className="font-bold">{node.label.split('·')[0]}</tspan>
                  <tspan className="text-[10px] opacity-80"> {node.label.split('·')[1] || ''}</tspan>
                  <tspan className="block text-[10px] font-mono font-bold fill-emerald-400">
                    {' '}({node.value.toLocaleString()})
                  </tspan>
                </text>
              </g>
            );
          })}
        </g>
      </svg>

      {/* Sankey Subtitle note */}
      <div className="mt-3 flex items-center justify-between text-xs text-slate-400 border-t border-slate-800/80 pt-2 px-2">
        <span className="flex items-center gap-2">
          <span className="inline-block w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse" />
          <span>Real-time Stream: ribbon thickness indicates live event volume flowing through zero-alloc buffers</span>
        </span>
        <span className="text-slate-500 font-mono text-[11px]">VRL Engine v1.9 • OCSF Class 4001 • Merkle Tree Audited</span>
      </div>
    </div>
  );
};
