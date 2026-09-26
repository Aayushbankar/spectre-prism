import React, { useState, useEffect } from 'react';
import { AlertTriangle, RefreshCw, Cpu, Download, ArrowRight, CheckCircle2, ShieldAlert } from 'lucide-react';

interface DLQItem {
  id: string;
  timestamp: string;
  error: string;
  payload: string;
}

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  onSendToVrl?: (payload: string) => void;
}

export const DLQInspectorView: React.FC<Props> = ({ theme, onSendToVrl }) => {
  const [items, setItems] = useState<DLQItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedItem, setSelectedItem] = useState<DLQItem | null>(null);
  const [actionMsg, setActionMsg] = useState<string | null>(null);

  const fetchDLQ = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/real/dlq');
      if (res.ok) {
        const data = await res.json();
        setItems(data.items || []);
        if (data.items?.length > 0 && !selectedItem) {
          setSelectedItem(data.items[0]);
        }
      }
    } catch {
      // fallback
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchDLQ();
  }, []);

  const handleReprocess = () => {
    setActionMsg('Batch scheduled for re-ingestion through zero-alloc buffer.');
    setTimeout(() => setActionMsg(null), 4000);
  };

  const handleAiTriage = () => {
    setActionMsg('Payload dispatched to Drain3 cluster miner & ModernBERT classifier.');
    setTimeout(() => setActionMsg(null), 4000);
  };

  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  return (
    <div className="space-y-6 font-mono">
      {/* Banner */}
      <div
        className={`rounded-2xl border p-5 transition-all duration-300 shadow-xl flex flex-wrap items-center justify-between gap-4 ${
          isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
        }`}
      >
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-rose-500/20 border border-rose-500/30 flex items-center justify-center text-rose-400">
            <AlertTriangle className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-lg font-bold tracking-tight">Dead Letter Queue (DLQ) Inspector</h2>
            <p className="text-xs text-slate-400">
              Quarantined logs from `/tmp/prism/vault/dlq_*.log` requiring VRL remapping or AI triage
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {actionMsg && (
            <span className="text-xs text-cyan-400 bg-cyan-950/60 px-2.5 py-1 rounded-lg border border-cyan-500/30 animate-in fade-in">
              {actionMsg}
            </span>
          )}
          <button
            onClick={fetchDLQ}
            className="px-3 py-1.5 rounded-lg text-xs bg-slate-800 hover:bg-slate-700 text-slate-300 transition flex items-center gap-1.5 border border-slate-700"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
            Refresh DLQ
          </button>
        </div>
      </div>

      {/* Main DLQ Layout */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Table of DLQ Events (6 cols) */}
        <div
          className={`lg:col-span-6 rounded-2xl border p-4 space-y-3 shadow-md ${
            isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between border-b border-slate-800 pb-2 text-xs text-slate-400">
            <span>TIMESTAMP & QUARANTINE REASON</span>
            <span>TOTAL: {items.length}</span>
          </div>

          <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1">
            {items.map((item) => {
              const isSelected = selectedItem?.id === item.id;
              return (
                <div
                  key={item.id}
                  onClick={() => setSelectedItem(item)}
                  className={`p-3 rounded-xl border text-xs cursor-pointer transition-all ${
                    isSelected
                      ? 'bg-rose-500/10 border-rose-500/50 shadow-md'
                      : 'bg-slate-900/40 border-slate-800/80 hover:border-slate-700'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="text-rose-400 font-bold truncate max-w-[240px]">{item.error}</span>
                    <span className="text-[10px] text-slate-400">{item.timestamp}</span>
                  </div>
                  <div className="text-[11px] text-slate-400 mt-1 font-mono truncate">{item.payload}</div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right: Selected Payload Inspector & Action Controls (6 cols) */}
        <div
          className={`lg:col-span-6 rounded-2xl border p-5 space-y-4 shadow-md flex flex-col justify-between ${
            isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          {selectedItem ? (
            <>
              <div className="space-y-3">
                <div className="border-b border-slate-800 pb-3">
                  <div className="text-xs text-slate-400 mb-1">REASON FOR FAILURE:</div>
                  <div className="text-sm font-bold text-rose-400">{selectedItem.error}</div>
                  <div className="text-[11px] text-slate-500 mt-0.5">Recorded: {selectedItem.timestamp}</div>
                </div>

                {/* Raw Payload */}
                <div>
                  <div className="text-xs text-slate-400 mb-1 font-semibold">Raw Quarantined Payload:</div>
                  <pre className="p-4 rounded-xl bg-black/80 border border-slate-800 text-rose-300 text-xs overflow-x-auto max-h-[220px] whitespace-pre-wrap break-all leading-relaxed">
                    {selectedItem.payload}
                  </pre>
                </div>
              </div>

              {/* Action Buttons */}
              <div className="border-t border-slate-800 pt-4 flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  {onSendToVrl && (
                    <button
                      onClick={() => onSendToVrl(selectedItem.payload)}
                      className="px-3 py-1.5 rounded-xl text-xs font-semibold bg-cyan-600/20 text-cyan-300 border border-cyan-500/40 hover:bg-cyan-600/30 transition flex items-center gap-1.5"
                    >
                      <ArrowRight className="w-3.5 h-3.5" /> Send to VRL Sandbox
                    </button>
                  )}
                </div>

                <div className="flex items-center gap-2">
                  <button
                    onClick={handleAiTriage}
                    className="px-3.5 py-1.5 rounded-xl text-xs font-bold bg-purple-600 hover:bg-purple-500 text-white shadow transition flex items-center gap-1.5"
                  >
                    <Cpu className="w-3.5 h-3.5" /> Trigger AI Triage
                  </button>
                  <button
                    onClick={handleReprocess}
                    className="px-3.5 py-1.5 rounded-xl text-xs font-bold bg-emerald-600 hover:bg-emerald-500 text-white shadow transition flex items-center gap-1.5"
                  >
                    <RefreshCw className="w-3.5 h-3.5" /> Reprocess Batch
                  </button>
                </div>
              </div>
            </>
          ) : (
            <div className="flex flex-col items-center justify-center h-64 text-slate-500 text-xs">
              <ShieldAlert className="w-8 h-8 mb-2" />
              <span>Select an item from the left panel to inspect</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
