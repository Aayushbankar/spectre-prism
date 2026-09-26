import React from 'react';
import {
  X,
  BookOpen,
  Zap,
  Activity,
  Globe,
  Binary,
  Cpu,
  ShieldCheck,
  CheckCircle2,
  FileCheck2,
  Lock,
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
                PRISM Sovereign SOC · Architecture & Operational Guide
              </h2>
              <p className="text-xs text-slate-400">
                Universal Ingestion, Semantic Normalization & Section 65B Legal Evidence Engine (SIH PS-26156 / NTRO)
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
            <ShieldCheck className="w-4 h-4" /> The Problem & National Security Solution
          </div>
          <p className="text-slate-200">
            Critical infrastructure security centers ingest petabytes of unstandardized firewall logs (Fortinet, Cisco ASA, Palo Alto, AWS VPC) daily. Traditional systems drop packets during surges and cannot withstand legal scrutiny in court.
          </p>
          <p className="text-slate-300">
            <strong>PRISM</strong> delivers an air-gapped, zero-allocation Rust Ingest Plane (&gt;300,000 EPS), compiles dynamic VRL parsers to the OCSF v1.9 schema, guarantees lossless byte accounting (&gt;90% closure ratio), and commits every event into an immutable Parquet vault with BLAKE3 Merkle roots and a 2-of-3 Ed25519 witness quorum compliant with Section 65B of the Indian Evidence Act.
          </p>
        </div>

        {/* 5 Architecture Planes */}
        <div className="space-y-4">
          <h3 className="text-sm font-bold text-slate-300 uppercase tracking-wider font-mono">
            PRISM Five-Plane Technical Architecture
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Plane 1: Ingest */}
            <div className="p-4 rounded-2xl border border-cyan-500/30 bg-slate-900/60 space-y-2">
              <div className="flex items-center gap-2 text-cyan-400 font-bold text-xs">
                <Zap className="w-4 h-4" /> 1. Ingest Plane (Tokio Ring Buffers)
              </div>
              <p className="text-[11px] text-slate-300 leading-relaxed">
                Asynchronous UDP 514/5514 listeners allocate zero heap memory per packet. Packets land directly into circular chunk buffers, handling &gt;300,000 EPS bursts with microsecond latencies.
              </p>
            </div>

            {/* Plane 2: Data Plane */}
            <div className="p-4 rounded-2xl border border-indigo-500/30 bg-slate-900/60 space-y-2">
              <div className="flex items-center gap-2 text-indigo-400 font-bold text-xs">
                <Activity className="w-4 h-4" /> 2. Data Plane (Dynamic VRL Normalizer)
              </div>
              <p className="text-[11px] text-slate-300 leading-relaxed">
                High-performance Vector Remap Language (VRL) converts raw syslog strings into structured OCSF Classes (4001 Network Activity, 2004 Security Finding, 3001 Authentication, 5001 Proxy).
              </p>
            </div>

            {/* Plane 3: Byte Accounting */}
            <div className="p-4 rounded-2xl border border-emerald-500/30 bg-slate-900/60 space-y-2">
              <div className="flex items-center gap-2 text-emerald-400 font-bold text-xs">
                <Binary className="w-4 h-4" /> 3. Byte Accounting & Fidelity Engine (§3.1)
              </div>
              <p className="text-[11px] text-slate-300 leading-relaxed">
                Every raw log byte is deterministically tagged into FIELD, LITERAL, or RESIDUE. A closure ratio &gt;90% proves mathematical integrity and zero information loss.
              </p>
            </div>

            {/* Plane 4: Vault & Section 65B */}
            <div className="p-4 rounded-2xl border border-amber-500/30 bg-slate-900/60 space-y-2">
              <div className="flex items-center gap-2 text-amber-400 font-bold text-xs">
                <FileCheck2 className="w-4 h-4" /> 4. Vault Plane (Section 65B Evidence)
              </div>
              <p className="text-[11px] text-slate-300 leading-relaxed">
                Cold storage in Apache Parquet 2.0 with ZSTD compression + cryptographic BLAKE3 Merkle Trees + 2-of-3 Ed25519 witness quorum (NTRO, CERT-In, Judiciary) providing legal chain-of-custody.
              </p>
            </div>
          </div>

          {/* Plane 5: Autonomous AI Control Plane */}
          <div className="p-4 rounded-2xl border border-purple-500/30 bg-slate-900/60 space-y-2">
            <div className="flex items-center gap-2 text-purple-400 font-bold text-xs">
              <Cpu className="w-4 h-4" /> 5. Autonomous AI Plane (Drain3 + ModernBERT + HitL Gatekeeper)
            </div>
            <p className="text-[11px] text-slate-300 leading-relaxed">
              When unseen log syntax lands in the Dead Letter Queue (DLQ), Drain3 clusters log templates, ModernBERT classifies the vendor and semantics, and an AI agent synthesizes a candidate VRL rule. The rule is held in the Human-in-the-Loop (HitL) Gatekeeper for 5-gate safety validation before hot-reloading without dropping packets.
            </p>
          </div>
        </div>

        {/* Footer */}
        <div className="pt-2 border-t border-slate-800 flex justify-end">
          <button
            onClick={onClose}
            className="px-5 py-2 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-bold text-xs transition"
          >
            Acknowledge & Enter Command Center
          </button>
        </div>
      </div>
    </div>
  );
};
