import React, { useState, useEffect } from 'react';
import { ShieldCheck, Check, X, Code, RefreshCw, Cpu, CheckCircle2, AlertCircle, Play } from 'lucide-react';

interface Rule {
  name: string;
  path: string;
  status: 'active' | 'pending' | 'approved';
  content: string;
}

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  onTestRule?: (code: string) => void;
}

export const GatekeeperHITLView: React.FC<Props> = ({ theme, onTestRule }) => {
  const [rules, setRules] = useState<Rule[]>([]);
  const [selectedRule, setSelectedRule] = useState<Rule | null>(null);
  const [loading, setLoading] = useState(false);
  const [actionMessage, setActionMessage] = useState<string | null>(null);

  const fetchRules = async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/real/rules');
      if (res.ok) {
        const data = await res.json();
        setRules(data.rules || []);
        if (data.rules?.length > 0 && !selectedRule) {
          setSelectedRule(data.rules[0]);
        }
      }
    } catch {
      // fallback
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchRules();
  }, []);

  const handleApprove = async (ruleName: string) => {
    try {
      const res = await fetch('/api/real/rules/approve', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ruleName }),
      });
      const data = await res.json();
      setActionMessage(data.message || `Rule ${ruleName} approved and deployed!`);
      setTimeout(() => setActionMessage(null), 4000);
      fetchRules();
    } catch (err: any) {
      setActionMessage(`Error: ${err.message}`);
    }
  };

  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  return (
    <div className="space-y-6 font-mono">
      {/* Header Banner */}
      <div
        className={`rounded-2xl border p-5 transition-all duration-300 shadow-xl flex flex-wrap items-center justify-between gap-4 ${
          isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
        }`}
      >
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-purple-500/20 border border-purple-500/30 flex items-center justify-center text-purple-400">
            <Cpu className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-lg font-bold tracking-tight">Human-in-the-Loop (HitL) Gatekeeper</h2>
            <p className="text-xs text-slate-400">
              AI-generated VRL Parsers mined via Drain3 & ModernBERT awaiting SecOps validation
            </p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {actionMessage && (
            <span className="text-xs text-emerald-400 bg-emerald-950/60 px-2.5 py-1 rounded-lg border border-emerald-500/30 animate-in fade-in">
              {actionMessage}
            </span>
          )}
          <button
            onClick={fetchRules}
            className="px-3 py-1.5 rounded-lg text-xs bg-slate-800 hover:bg-slate-700 text-slate-300 transition flex items-center gap-1.5 border border-slate-700"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
            Refresh Queue
          </button>
        </div>
      </div>

      {/* Rules Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
        {/* Left: Rule List (5 cols) */}
        <div
          className={`lg:col-span-5 rounded-2xl border p-4 space-y-3 shadow-md ${
            isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          <div className="flex items-center justify-between border-b border-slate-800 pb-2 text-xs text-slate-400">
            <span>RULE IDENTIFIER</span>
            <span>STATUS</span>
          </div>

          <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1">
            {rules.map((rule) => {
              const isSelected = selectedRule?.name === rule.name;
              return (
                <div
                  key={rule.name}
                  onClick={() => setSelectedRule(rule)}
                  className={`p-3 rounded-xl border text-xs cursor-pointer transition-all ${
                    isSelected
                      ? 'bg-purple-500/10 border-purple-500/50 shadow-md'
                      : 'bg-slate-900/40 border-slate-800/80 hover:border-slate-700'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span className="font-bold text-slate-200 truncate max-w-[200px]">{rule.name}</span>
                    <span
                      className={`px-2 py-0.5 rounded text-[10px] uppercase font-bold ${
                        rule.status === 'approved'
                          ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                          : rule.status === 'pending'
                          ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30 animate-pulse'
                          : 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/30'
                      }`}
                    >
                      {rule.status}
                    </span>
                  </div>
                  <div className="text-[11px] text-slate-400 mt-1 truncate">{rule.path}</div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Right: Selected Rule Inspector & Action Buttons (7 cols) */}
        <div
          className={`lg:col-span-7 rounded-2xl border p-5 space-y-4 shadow-md flex flex-col justify-between ${
            isDark ? 'bg-[#141724]/95 border-slate-800 text-slate-100' : 'bg-white border-slate-200 text-slate-900'
          }`}
        >
          {selectedRule ? (
            <>
              <div className="space-y-3">
                <div className="flex items-center justify-between border-b border-slate-800 pb-3">
                  <div>
                    <h3 className="text-sm font-bold text-slate-100">{selectedRule.name}</h3>
                    <p className="text-[11px] text-slate-400">{selectedRule.path}</p>
                  </div>
                  <span className="text-xs px-2.5 py-1 rounded-lg bg-slate-800 border border-slate-700 text-slate-300">
                    OCSF Class 4001 Compliant
                  </span>
                </div>

                {/* VRL Code Preview */}
                <div>
                  <div className="flex items-center justify-between text-xs text-slate-400 mb-1">
                    <span className="flex items-center gap-1.5">
                      <Code className="w-3.5 h-3.5 text-cyan-400" /> Vector Remap Language (VRL) Spec
                    </span>
                    <span className="text-[10px] text-slate-500">Zero-alloc engine</span>
                  </div>
                  <pre className="p-4 rounded-xl bg-black/80 border border-slate-800 text-emerald-400 text-xs overflow-x-auto max-h-[300px] leading-relaxed">
                    {selectedRule.content}
                  </pre>
                </div>
              </div>

              {/* Action Buttons */}
              <div className="border-t border-slate-800 pt-4 flex flex-wrap items-center justify-between gap-3">
                <div className="flex items-center gap-2">
                  {onTestRule && (
                    <button
                      onClick={() => onTestRule(selectedRule.content)}
                      className="px-3.5 py-2 rounded-xl text-xs font-semibold bg-cyan-600/20 text-cyan-300 border border-cyan-500/40 hover:bg-cyan-600/30 transition flex items-center gap-1.5"
                    >
                      <Play className="w-3.5 h-3.5" /> Test in VRL Sandbox
                    </button>
                  )}
                </div>

                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleApprove(selectedRule.name)}
                    className="px-4 py-2 rounded-xl text-xs font-bold bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-600/30 transition flex items-center gap-1.5"
                  >
                    <CheckCircle2 className="w-4 h-4" /> Approve & Hot-Deploy
                  </button>
                </div>
              </div>
            </>
          ) : (
            <div className="flex flex-col items-center justify-center h-64 text-slate-500 text-xs">
              <AlertCircle className="w-8 h-8 mb-2" />
              <span>Select a rule from the left panel to inspect</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
