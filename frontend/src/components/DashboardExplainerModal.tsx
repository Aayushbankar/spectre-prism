import React from 'react';
import {
  X,
  BookOpen,
  Zap,
  LayoutDashboard,
  Globe,
  FileCheck2,
  Cpu,
  Layers,
  ArrowRight,
  ShieldCheck,
  CheckCircle2,
} from 'lucide-react';

interface Props {
  isOpen: boolean;
  onClose: () => void;
}

export const DashboardExplainerModal: React.FC<Props> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4 animate-in fade-in duration-200">
      <div className="relative w-full max-w-4xl max-h-[90vh] overflow-y-auto rounded-3xl border border-slate-700 bg-[#0e111a] p-6 sm:p-8 shadow-2xl text-slate-100 space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-slate-800 pb-4">
          <div className="flex items-center gap-3">
            <div className="w-11 h-11 rounded-2xl bg-cyan-500/20 border border-cyan-500/40 flex items-center justify-center text-cyan-400">
              <BookOpen className="w-6 h-6" />
            </div>
            <div>
              <h2 className="text-xl font-bold tracking-tight text-white">
                PRISM Universal Log Parsing · Operator & Evaluator Guide
              </h2>
              <p className="text-xs text-slate-400">
                Understanding the Telemetry, Architecture Planes, and Reference Designs (SIH PS 26156)
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 transition"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {/* Core Mission Banner */}
        <div className="p-4 rounded-2xl bg-gradient-to-r from-cyan-950/40 to-indigo-950/40 border border-cyan-500/30 text-xs leading-relaxed space-y-2">
          <div className="flex items-center gap-2 text-cyan-400 font-bold uppercase tracking-wider">
            <ShieldCheck className="w-4 h-4" /> The Problem & Solution
          </div>
          <p className="text-slate-200">
            Enterprise networks ingest hundreds of gigabytes of raw syslog, JSON, and CSV firewalls (Fortinet, Cisco ASA, Palo Alto) daily. Traditional SIEMs choke on high ingestion rates and lack legally admissible integrity.
          </p>
          <p className="text-slate-300">
            <strong>PRISM</strong> solves this with a <strong>zero-packet-allocation Rust Ingest Plane</strong> (&gt;300,000 EPS), a <strong>VRL Normalizer</strong> converting formats into the <strong>OCSF Class 4001 (Network Activity)</strong> standard, and an immutable <strong>Parquet 2.0 Vault with BLAKE3 Merkle Trees</strong> compliant with <strong>Section 65B of the Indian Evidence Act</strong>.
          </p>
        </div>

        {/* 3 Reference Designs Explained */}
        <div className="space-y-4">
          <h3 className="text-sm font-bold text-slate-300 uppercase tracking-wider font-mono">
            How The 3 Reference Designs Are Adapted
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Design 1 */}
            <div className="p-4 rounded-2xl border border-cyan-500/30 bg-slate-900/60 space-y-2.5">
              <div className="flex items-center gap-2 text-cyan-400 font-bold text-xs">
                <Zap className="w-4 h-4" /> Design 1: Ingest Flow
              </div>
              <p className="text-[11px] text-slate-300 leading-relaxed">
                Adapted from <code className="text-cyan-300 font-mono">ideainso3.jpeg</code>. Displays the live <strong>Sankey diagram</strong> showing how raw logs flow from destination origins (China, India, USA) through parsing engines (Palo Alto VRL, FortiGate, Cisco ASA) to final OCSF status (200 OK, 404 Blocked, 503 DLQ).
              </p>
              <div className="text-[10px] text-slate-400 font-mono bg-black/40 p-2 rounded-lg">
                Key SLI: 94/100 Health Score, 92.3% Error Budget.
              </div>
            </div>

            {/* Design 2 */}
            <div className="p-4 rounded-2xl border border-purple-500/30 bg-slate-900/60 space-y-2.5">
              <div className="flex items-center gap-2 text-purple-400 font-bold text-xs">
                <LayoutDashboard className="w-4 h-4" /> Design 2: Executive Analytics
              </div>
              <p className="text-[11px] text-slate-300 leading-relaxed">
                Adapted from <code className="text-purple-300 font-mono">dashboard2.jpeg</code>. Provides C-level observability with sparkline KPI cards, real-time buffer progress meters, smooth spline EPS throughput curves, protocol donut (UDP vs TCP), and latency distribution bars.
              </p>
              <div className="text-[10px] text-slate-400 font-mono bg-black/40 p-2 rounded-lg">
                Key Metric: p99 parse latency 9.56 µs (&lt; 25 µs target).
              </div>
            </div>

            {/* Design 3 */}
            <div className="p-4 rounded-2xl border border-amber-500/30 bg-slate-900/60 space-y-2.5">
              <div className="flex items-center gap-2 text-amber-400 font-bold text-xs">
                <Globe className="w-4 h-4" /> Design 3: Tactical Cyber Command
              </div>
              <p className="text-[11px] text-slate-300 leading-relaxed">
                Adapted from <code className="text-amber-300 font-mono">dashboard_1.jpeg</code>. High-tech tactical HUD featuring the <strong>Global Threat Vector Map</strong>, ballistic trajectory arcs, threat category radar, and the <strong>Section 65B Forensic Event Ledger</strong> with BLAKE3 verification.
              </p>
              <div className="text-[10px] text-slate-400 font-mono bg-black/40 p-2 rounded-lg">
                Legal Feature: Court-admissible BLAKE3 cryptographic history.
              </div>
            </div>
          </div>
        </div>

        {/* Interactive Features / How to Test */}
        <div className="space-y-3">
          <h3 className="text-sm font-bold text-slate-300 uppercase tracking-wider font-mono">
            How to Test PRISM Interactively
          </h3>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
              <div className="font-bold text-emerald-400 flex items-center gap-1.5">
                <CheckCircle2 className="w-4 h-4" /> 1. Inject Live Traffic
              </div>
              <p className="text-slate-400 text-[11px]">
                Click <strong>+200 Fortinet</strong>, <strong>+200 Cisco</strong>, or <strong>+200 Palo Alto</strong> in the top controller strip to send real packets to the UDP listener and watch the EPS meters update.
              </p>
            </div>

            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
              <div className="font-bold text-rose-400 flex items-center gap-1.5">
                <CheckCircle2 className="w-4 h-4" /> 2. Trigger the AI Self-Healing Loop
              </div>
              <p className="text-slate-400 text-[11px]">
                Click <strong>Trigger DLQ Anomaly</strong> to inject an unknown payload. Watch it enter the Dead Letter Queue, get clustered by Drain3, and appear in the <strong>HitL Gatekeeper</strong> for 1-click approval!
              </p>
            </div>

            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
              <div className="font-bold text-cyan-400 flex items-center gap-1.5">
                <CheckCircle2 className="w-4 h-4" /> 3. Test Rules in VRL Sandbox
              </div>
              <p className="text-slate-400 text-[11px]">
                Switch to the <strong>VRL Sandbox</strong> tab, pick a firewall preset, and click <strong>Execute VRL Parse</strong> to verify real OCSF Class 4001 output and cryptographic hash generation.
              </p>
            </div>

            <div className="p-3 rounded-xl bg-slate-900/80 border border-slate-800 space-y-1">
              <div className="font-bold text-amber-400 flex items-center gap-1.5">
                <CheckCircle2 className="w-4 h-4" /> 4. Verify Section 65B Evidence
              </div>
              <p className="text-slate-400 text-[11px]">
                In the Tactical Command view, click <strong>Verify 65B</strong> on any row in the Forensic Ledger to verify the BLAKE3 hash against <code className="text-amber-300">ledger.log</code> and Parquet batch cold storage.
              </p>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="border-t border-slate-800 pt-4 flex items-center justify-between">
          <span className="text-xs text-slate-500 font-mono">
            Smart India Hackathon 2026 • Problem Statement 26156 • PRISM Engine
          </span>
          <button
            onClick={onClose}
            className="px-5 py-2 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-bold text-xs transition"
          >
            Got It, Let&apos;s Explore
          </button>
        </div>
      </div>
    </div>
  );
};
