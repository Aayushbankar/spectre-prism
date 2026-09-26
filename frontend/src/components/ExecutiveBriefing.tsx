import React from 'react';
import {
  ShieldAlert,
  Sparkles,
  Info,
  Radio,
  Zap,
  TrendingUp,
  AlertOctagon,
  Cpu,
  ChevronRight,
  HelpCircle,
} from 'lucide-react';

export type SystemScenario = 'baseline' | 'attack' | 'ai_triage';

interface Props {
  theme: 'dark' | 'light' | 'cyber';
  scenario: SystemScenario;
  onScenarioChange: (scenario: SystemScenario) => void;
  onOpenExplainer: () => void;
  onOpenGatekeeper?: () => void;
  isBackendLive: boolean;
  eps: number;
}

export const ExecutiveBriefing: React.FC<Props> = ({
  theme,
  scenario,
  onScenarioChange,
  onOpenExplainer,
  onOpenGatekeeper,
  isBackendLive,
  eps,
}) => {
  const isCyber = theme === 'cyber';
  const isDark = theme === 'dark' || isCyber;

  const scenarioDetails = {
    baseline: {
      posture: 'OPERATIONAL · NORMAL INGESTION',
      postureColor: 'text-emerald-400 bg-emerald-500/10 border-emerald-500/30',
      headline: 'PRISM is ingesting heterogeneous firewall logs with zero packet allocation.',
      narrative:
        'All ingestion worker threads are processing Fortinet, Cisco ASA, and Palo Alto streams without drops. p99 parse latency is maintained at 9.56 µs. Continuous Merkle tree batching is hashing raw events into immutable Parquet cold storage for Indian Evidence Act Section 65B compliance.',
      actionNeeded: 'None. Ingestion health is optimal (94/100). Error budget is preserved at 92.3%.',
    },
    attack: {
      posture: 'ALERT · THREAT BURST & VOLUMETRIC SURGE',
      postureColor: 'text-rose-400 bg-rose-500/10 border-rose-500/30',
      headline: 'DDoS volumetric scan & brute-force sweep detected from foreign endpoints.',
      narrative:
        'Traffic volume spiked to 148,000 EPS. Heuristic routing identified high-frequency SYN floods on port 443 and DNS amplification attempts targeting core gateway 10.0.5.50. VRL rules are enforcing automated Deny actions, and forensic hashes are logged to cold vault for law enforcement chain-of-custody.',
      actionNeeded: 'ACL rate-limiting recommended on Ingress Plane. Inspect high-severity alerts in Tactical Map.',
    },
    ai_triage: {
      posture: 'AI LOOP ACTIVE · NEW LOG PATTERN DETECTED',
      postureColor: 'text-purple-400 bg-purple-500/10 border-purple-500/30',
      headline: 'Unseen firewall syntax routed to Dead Letter Queue (DLQ). AI mined new parser.',
      narrative:
        'A batch of unknown web proxy logs was quarantined into `/tmp/prism/vault/dlq.log`. The PRISM AI Brain extracted dynamic parameters using Drain3 log template clustering and classified the vendor via Laya ModernBERT. A candidate VRL parser has been synthesized and awaits Human-in-the-Loop approval.',
      actionNeeded: 'Review and approve candidate VRL parser in HitL Gatekeeper to hot-reload into pipeline.',
    },
  };

  const curr = scenarioDetails[scenario];

  return (
    <div
      className={`rounded-2xl border p-5 transition-all duration-300 shadow-xl ${
        isCyber
          ? 'border-amber-500/30 bg-black/90 text-amber-50'
          : isDark
          ? 'border-slate-800 bg-[#131622]/95 text-slate-100'
          : 'border-slate-200 bg-white text-slate-900 shadow-md'
      }`}
    >
      <div className="flex flex-wrap items-center justify-between gap-4 border-b border-slate-800/80 pb-4">
        {/* Left: Intelligence Posture */}
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-cyan-500 to-indigo-600 flex items-center justify-center text-white shadow-lg shadow-indigo-500/20">
            <Sparkles className="w-5 h-5" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="text-xs font-mono font-bold uppercase tracking-wider text-slate-400">
                SOC Intelligence Briefing
              </span>
              <span className={`px-2.5 py-0.5 rounded-full text-[11px] font-mono font-bold border ${curr.postureColor}`}>
                {curr.posture}
              </span>
            </div>
            <h3 className="text-base font-bold tracking-tight mt-0.5">{curr.headline}</h3>
          </div>
        </div>

        {/* Right: Quick Explainer & Scenario Selector */}
        <div className="flex flex-wrap items-center gap-2">
          {/* Guide Me Button */}
          <button
            onClick={onOpenExplainer}
            className="px-3 py-1.5 rounded-xl text-xs font-semibold bg-cyan-600/20 text-cyan-300 border border-cyan-500/30 hover:bg-cyan-600/30 transition flex items-center gap-1.5"
            title="Open guided explainer for the PRISM dashboard"
          >
            <HelpCircle className="w-4 h-4 text-cyan-400" />
            <span>How This Dashboard Works</span>
          </button>

          {/* Scenario Buttons */}
          <div className="flex items-center p-1 rounded-xl bg-slate-900 border border-slate-800 text-xs font-mono">
            <button
              onClick={() => onScenarioChange('baseline')}
              className={`px-2.5 py-1 rounded-lg transition ${
                scenario === 'baseline'
                  ? 'bg-emerald-600 text-white font-bold shadow'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              1. Normal Stream
            </button>
            <button
              onClick={() => onScenarioChange('attack')}
              className={`px-2.5 py-1 rounded-lg transition ${
                scenario === 'attack'
                  ? 'bg-rose-600 text-white font-bold shadow'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              2. Threat Surge
            </button>
            <button
              onClick={() => onScenarioChange('ai_triage')}
              className={`px-2.5 py-1 rounded-lg transition ${
                scenario === 'ai_triage'
                  ? 'bg-purple-600 text-white font-bold shadow'
                  : 'text-slate-400 hover:text-white'
              }`}
            >
              3. AI Self-Healing
            </button>
          </div>
        </div>
      </div>

      {/* Narrative Explanation */}
      <div className="mt-4 grid grid-cols-1 lg:grid-cols-12 gap-4 items-center">
        <div className="lg:col-span-8 text-xs leading-relaxed text-slate-300">
          <p>{curr.narrative}</p>
        </div>

        {/* Action Callout */}
        <div className="lg:col-span-4 p-3 rounded-xl bg-slate-900/60 border border-slate-800 space-y-1 text-xs">
          <span className="text-[10px] font-mono text-slate-400 uppercase font-semibold block">
            Recommended Action:
          </span>
          <p className="text-slate-200 font-medium">{curr.actionNeeded}</p>
          {scenario === 'ai_triage' && onOpenGatekeeper && (
            <button
              onClick={onOpenGatekeeper}
              className="mt-1 px-2.5 py-1 rounded-lg bg-purple-600 hover:bg-purple-500 text-white text-[11px] font-bold flex items-center gap-1 transition"
            >
              Review Rule in HitL Gatekeeper <ChevronRight className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
