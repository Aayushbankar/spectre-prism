import React, { useState, useEffect } from 'react';
import {
  Server,
  Activity,
  Cpu,
  Database,
  Send,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  Radio,
  FileCheck2,
  RefreshCw,
} from 'lucide-react';

interface RealStatus {
  isBackendLive: boolean;
  metrics: {
    eps: number;
    processed: number;
    drops: number;
    dlq: number;
    telemetry: {
      fortinet: number;
      cisco: number;
      paloalto: number;
      latency_us: number;
    };
  };
  aiOnline: boolean;
  aiHeartbeat: string | null;
  ledgerTail: string;
  ledgerCount: number;
  dlqFilesCount: number;
}

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  onLiveMetricsUpdate?: (metrics: RealStatus['metrics']) => void;
}

export const LiveBackendBridge: React.FC<Props> = ({ theme, onLiveMetricsUpdate }) => {
  const [status, setStatus] = useState<RealStatus>({
    isBackendLive: false,
    metrics: {
      eps: 0,
      processed: 0,
      drops: 0,
      dlq: 0,
      telemetry: { fortinet: 0, cisco: 0, paloalto: 0, latency_us: 0 },
    },
    aiOnline: false,
    aiHeartbeat: null,
    ledgerTail: 'No batches committed yet',
    ledgerCount: 0,
    dlqFilesCount: 0,
  });

  const [injecting, setInjecting] = useState<string | null>(null);
  const [lastMessage, setLastMessage] = useState<string | null>(null);

  // Poll real backend status every 1.5s
  useEffect(() => {
    let isMounted = true;
    const fetchStatus = async () => {
      try {
        const res = await fetch('/api/real/status');
        if (res.ok) {
          const data: RealStatus = await res.json();
          if (isMounted) {
            setStatus(data);
            if (onLiveMetricsUpdate && data.isBackendLive) {
              onLiveMetricsUpdate(data.metrics);
            }
          }
        }
      } catch {
        // standalone / offline fallback
      }
    };

    fetchStatus();
    const interval = setInterval(fetchStatus, 1500);
    return () => {
      isMounted = false;
      clearInterval(interval);
    };
  }, [onLiveMetricsUpdate]);

  const handleInject = async (vendor: 'fortinet' | 'cisco' | 'paloalto' | 'anomaly') => {
    setInjecting(vendor);
    try {
      const res = await fetch('/api/real/stream/inject', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ vendor, count: 200, port: 514 }),
      });
      const data = await res.json();
      setLastMessage(data.message || `Injected 200 ${vendor} packets`);
      setTimeout(() => setLastMessage(null), 4000);
    } catch (err: any) {
      setLastMessage(`Error injecting: ${err.message}`);
    } finally {
      setInjecting(null);
    }
  };

  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  return (
    <div
      className={`rounded-2xl border p-5 transition-all duration-300 shadow-xl ${
        isCyber
          ? 'border-amber-500/30 bg-black/90 text-amber-50'
          : isDark
          ? 'border-slate-800 bg-[#141724]/95 text-slate-100'
          : 'border-slate-200 bg-white text-slate-900'
      }`}
    >
      {/* Title & Live Status Bar */}
      <div className="flex flex-wrap items-center justify-between gap-4 border-b border-slate-800/80 pb-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-cyan-500/20 border border-cyan-500/30 flex items-center justify-center text-cyan-400">
            <Server className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="text-base font-bold tracking-tight">PRISM Real Backend Telemetry</h3>
              <span
                className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-mono font-semibold ${
                  status.isBackendLive
                    ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30'
                    : 'bg-amber-500/10 text-amber-400 border border-amber-500/30'
                }`}
              >
                <span
                  className={`w-2 h-2 rounded-full ${
                    status.isBackendLive ? 'bg-emerald-400 animate-ping' : 'bg-amber-400'
                  }`}
                />
                {status.isBackendLive ? 'LIVE /tmp/prism_metrics.json CONNECTED' : 'BACKEND IDLE / DEMO MODE'}
              </span>
            </div>
            <p className="text-xs text-slate-400 font-mono">
              Zero-Alloc Ingest Buffer • Section 65B Vault • Drain3/Laya AI Loop
            </p>
          </div>
        </div>

        {/* AI Brain Status Pill */}
        <div className="flex items-center gap-2 font-mono text-xs">
          <span className="text-slate-400 flex items-center gap-1">
            <Cpu className="w-4 h-4 text-purple-400" /> AI Control Plane:
          </span>
          <span
            className={`px-2.5 py-1 rounded-lg font-bold flex items-center gap-1.5 ${
              status.aiOnline
                ? 'bg-purple-500/20 text-purple-300 border border-purple-500/30'
                : 'bg-slate-800 text-slate-400'
            }`}
          >
            {status.aiOnline ? (
              <>
                <Radio className="w-3.5 h-3.5 text-purple-400 animate-pulse" />
                Drain3 & ModernBERT ONLINE
              </>
            ) : (
              'Standby'
            )}
          </span>
        </div>
      </div>

      {/* Metrics Row from Actual /tmp/prism_metrics.json */}
      <div className="grid grid-cols-2 sm:grid-cols-4 lg:grid-cols-6 gap-3 my-4">
        {/* Real EPS */}
        <div className="p-3 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
          <span className="text-[11px] font-mono text-slate-400 block">REAL INGEST EPS</span>
          <div className="text-2xl font-extrabold text-cyan-400 font-mono">
            {status.metrics.eps.toLocaleString()}
          </div>
          <span className="text-[10px] text-slate-500 font-mono">events / sec</span>
        </div>

        {/* Processed Events */}
        <div className="p-3 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
          <span className="text-[11px] font-mono text-slate-400 block">TOTAL PROCESSED</span>
          <div className="text-2xl font-extrabold text-emerald-400 font-mono">
            {status.metrics.processed.toLocaleString()}
          </div>
          <span className="text-[10px] text-slate-500 font-mono">normalized events</span>
        </div>

        {/* Ingest Drops */}
        <div className="p-3 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
          <span className="text-[11px] font-mono text-slate-400 block">BUFFER DROPS</span>
          <div className="text-2xl font-extrabold text-amber-400 font-mono">
            {status.metrics.drops.toLocaleString()}
          </div>
          <span className="text-[10px] text-slate-500 font-mono">zero-alloc buffer</span>
        </div>

        {/* DLQ Count */}
        <div className="p-3 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
          <span className="text-[11px] font-mono text-slate-400 block">DLQ QUARANTINED</span>
          <div className="text-2xl font-extrabold text-rose-400 font-mono">
            {status.metrics.dlq.toLocaleString()}
          </div>
          <span className="text-[10px] text-slate-500 font-mono">awaiting AI triage</span>
        </div>

        {/* Fortinet Telemetry */}
        <div className="p-3 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
          <span className="text-[11px] font-mono text-slate-400 block">FORTINET LOGS</span>
          <div className="text-2xl font-extrabold text-slate-200 font-mono">
            {status.metrics.telemetry.fortinet.toLocaleString()}
          </div>
          <span className="text-[10px] text-slate-500 font-mono">key-value VRL</span>
        </div>

        {/* Palo Alto Telemetry */}
        <div className="p-3 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1">
          <span className="text-[11px] font-mono text-slate-400 block">PALO ALTO LOGS</span>
          <div className="text-2xl font-extrabold text-slate-200 font-mono">
            {status.metrics.telemetry.paloalto.toLocaleString()}
          </div>
          <span className="text-[10px] text-slate-500 font-mono">CSV VRL mapping</span>
        </div>
      </div>

      {/* Merkle Ledger Tail & Section 65B Proof */}
      <div className="p-3 rounded-xl bg-black/60 border border-slate-800 flex flex-wrap items-center justify-between gap-3 text-xs font-mono">
        <div className="flex items-center gap-2">
          <FileCheck2 className="w-4 h-4 text-emerald-400" />
          <span className="text-slate-400">Latest Merkle Vault Root (/tmp/prism/vault/ledger.log):</span>
          <span className="text-emerald-400 font-bold max-w-xs truncate">{status.ledgerTail}</span>
        </div>
        <span className="text-slate-400">
          Committed Batches: <strong className="text-white">{status.ledgerCount}</strong>
        </span>
      </div>

      {/* Real Ingest Stream Controller Buttons (Actionable buttons for PRISM!) */}
      <div className="mt-4 pt-3 border-t border-slate-800/80 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono text-slate-400 font-semibold uppercase">
            Ingest Traffic Injector:
          </span>
          {lastMessage && (
            <span className="text-xs font-mono text-emerald-400 bg-emerald-950/60 px-2 py-0.5 rounded border border-emerald-500/30 animate-in fade-in">
              {lastMessage}
            </span>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {/* Inject Fortinet */}
          <button
            disabled={injecting !== null}
            onClick={() => handleInject('fortinet')}
            className="px-3 py-1.5 rounded-lg text-xs font-mono font-semibold bg-orange-500/20 text-orange-300 border border-orange-500/40 hover:bg-orange-500/30 transition flex items-center gap-1.5 disabled:opacity-50"
          >
            <Send className="w-3.5 h-3.5" />
            +200 Fortinet Logs
          </button>

          {/* Inject Cisco ASA */}
          <button
            disabled={injecting !== null}
            onClick={() => handleInject('cisco')}
            className="px-3 py-1.5 rounded-lg text-xs font-mono font-semibold bg-yellow-500/20 text-yellow-300 border border-yellow-500/40 hover:bg-yellow-500/30 transition flex items-center gap-1.5 disabled:opacity-50"
          >
            <Send className="w-3.5 h-3.5" />
            +200 Cisco ASA
          </button>

          {/* Inject Palo Alto */}
          <button
            disabled={injecting !== null}
            onClick={() => handleInject('paloalto')}
            className="px-3 py-1.5 rounded-lg text-xs font-mono font-semibold bg-blue-500/20 text-blue-300 border border-blue-500/40 hover:bg-blue-500/30 transition flex items-center gap-1.5 disabled:opacity-50"
          >
            <Send className="w-3.5 h-3.5" />
            +200 Palo Alto
          </button>

          {/* Inject Anomaly (DLQ Trigger) */}
          <button
            disabled={injecting !== null}
            onClick={() => handleInject('anomaly')}
            className="px-3 py-1.5 rounded-lg text-xs font-mono font-semibold bg-rose-500/20 text-rose-300 border border-rose-500/40 hover:bg-rose-500/30 transition flex items-center gap-1.5 disabled:opacity-50"
            title="Inject an unknown alien log to trigger the Dead Letter Queue and AI Triage loop!"
          >
            <AlertTriangle className="w-3.5 h-3.5" />
            Trigger DLQ Anomaly
          </button>
        </div>
      </div>
    </div>
  );
};
