import React, { useState, useEffect } from 'react';
import type { DashboardView, DashboardTheme } from './types';
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';
import { FlowPipelineView } from './components/FlowPipelineView';
import { ExecutiveAnalyticsView } from './components/ExecutiveAnalyticsView';
import { TacticalCommandView } from './components/TacticalCommandView';
import { LiveBackendBridge } from './components/LiveBackendBridge';
import { GatekeeperHITLView } from './components/GatekeeperHITLView';
import { DLQInspectorView } from './components/DLQInspectorView';
import { VRLPlaygroundView } from './components/VRLPlaygroundView';
import { ExecutiveBriefing, type SystemScenario } from './components/ExecutiveBriefing';
import { DashboardExplainerModal } from './components/DashboardExplainerModal';

export function App() {
  const [currentView, setCurrentView] = useState<DashboardView>('all');
  const [theme, setTheme] = useState<DashboardTheme>('dark');
  const [isLiveStreaming, setIsLiveStreaming] = useState<boolean>(true);
  const [scenario, setScenario] = useState<SystemScenario>('baseline');
  const [isExplainerOpen, setIsExplainerOpen] = useState<boolean>(false);
  const [isBackendLive, setIsBackendLive] = useState<boolean>(false);
  const [liveEps, setLiveEps] = useState<number>(85620);

  const [vrlInitialPayload, setVrlInitialPayload] = useState<string | undefined>(undefined);
  const [vrlInitialRule, setVrlInitialRule] = useState<string | undefined>(undefined);

  const handleSendToVrl = (payload: string) => {
    setVrlInitialPayload(payload);
    setCurrentView('vrl');
  };

  const handleTestRuleInVrl = (ruleCode: string) => {
    setVrlInitialRule(ruleCode);
    setCurrentView('vrl');
  };

  const isCyber = theme === 'cyber';
  const isLight = theme === 'light';

  return (
    <div
      className={`min-h-screen flex transition-colors duration-300 ${
        isCyber
          ? 'bg-[#08090d] text-amber-50 selection:bg-amber-500 selection:text-black font-sans'
          : isLight
          ? 'bg-[#f4f5f9] text-slate-800 font-sans'
          : 'bg-[#0f111a] text-slate-100 font-sans'
      }`}
    >
      {/* Explainer Modal */}
      <DashboardExplainerModal
        isOpen={isExplainerOpen}
        onClose={() => setIsExplainerOpen(false)}
      />

      {/* Left Sidebar Navigation */}
      <Sidebar
        currentView={currentView}
        onViewChange={setCurrentView}
        theme={theme}
        onThemeChange={setTheme}
      />

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0 overflow-y-auto max-h-screen">
        <Header
          currentView={currentView}
          onViewChange={setCurrentView}
          theme={theme}
          isLiveStreaming={isLiveStreaming}
          onToggleLiveStream={() => setIsLiveStreaming(!isLiveStreaming)}
          onOpenExplainer={() => setIsExplainerOpen(true)}
        />

        <main className="flex-1 p-6 space-y-6 max-w-[1600px] w-full mx-auto">
          {/* Executive Briefing: Explains what the dashboard is saying in plain English */}
          <ExecutiveBriefing
            theme={theme}
            scenario={scenario}
            onScenarioChange={setScenario}
            onOpenExplainer={() => setIsExplainerOpen(true)}
            onOpenGatekeeper={() => setCurrentView('gatekeeper')}
            isBackendLive={isBackendLive}
            eps={liveEps}
          />

          {/* Real Backend Bridge & Live Traffic Controller */}
          <LiveBackendBridge
            theme={theme}
            onLiveMetricsUpdate={(metrics) => {
              setIsBackendLive(true);
              if (metrics.eps > 0) setLiveEps(metrics.eps);
            }}
          />

          {/* View 1: Ingest & Traffic Flow (Design 1 - ideainso3.jpeg) */}
          {currentView === 'pipeline' && (
            <FlowPipelineView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {/* View 2: Executive Analytics & Operations (Design 2 - dashboard2.jpeg) */}
          {currentView === 'analytics' && (
            <ExecutiveAnalyticsView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {/* View 3: Tactical Cyber Command & Threat Map (Design 3 - dashboard_1.jpeg) */}
          {currentView === 'tactical' && (
            <TacticalCommandView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {/* View 4: Human-in-the-Loop (HitL) Gatekeeper */}
          {currentView === 'gatekeeper' && (
            <GatekeeperHITLView theme={theme} onTestRule={handleTestRuleInVrl} />
          )}

          {/* View 5: Dead Letter Queue (DLQ) Inspector */}
          {currentView === 'dlq' && (
            <DLQInspectorView theme={theme} onSendToVrl={handleSendToVrl} />
          )}

          {/* View 6: Interactive VRL Sandbox & Validator */}
          {currentView === 'vrl' && (
            <VRLPlaygroundView
              theme={theme}
              initialPayload={vrlInitialPayload}
              initialRule={vrlInitialRule}
            />
          )}

          {/* View 7: Unified View (All 3 Reference Designs Combined) */}
          {currentView === 'all' && (
            <div className="space-y-12">
              {/* Section 1: Ingest Flow (ideainso3.jpeg) */}
              <section className="space-y-3">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded text-[11px] font-bold bg-cyan-500/20 text-cyan-400 border border-cyan-500/30">
                      DESIGN 1 · ideainso3.jpeg
                    </span>
                    <h2 className="text-lg font-bold">Traffic Flow & Pipeline Overview</h2>
                  </div>
                  <span className="text-xs text-slate-400 font-mono">Zero-Alloc UDP Buffer → OCSF Egress</span>
                </div>
                <FlowPipelineView theme={theme} isLiveStreaming={isLiveStreaming} />
              </section>

              {/* Section 2: Executive Analytics (dashboard2.jpeg) */}
              <section className="space-y-3">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded text-[11px] font-bold bg-purple-500/20 text-purple-400 border border-purple-500/30">
                      DESIGN 2 · dashboard2.jpeg
                    </span>
                    <h2 className="text-lg font-bold">Executive Analytics & Real-Time Telemetry</h2>
                  </div>
                  <span className="text-xs text-slate-400 font-mono">Multi-Theme • Live Spline EPS</span>
                </div>
                <ExecutiveAnalyticsView theme={theme} isLiveStreaming={isLiveStreaming} />
              </section>

              {/* Section 3: Tactical Cyber Command (dashboard_1.jpeg) */}
              <section className="space-y-3">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded text-[11px] font-bold bg-amber-500/20 text-amber-400 border border-amber-500/30">
                      DESIGN 3 · dashboard_1.jpeg
                    </span>
                    <h2 className="text-lg font-bold">Tactical Cyber Command & Threat Intelligence</h2>
                  </div>
                  <span className="text-xs text-slate-400 font-mono">Global Map • Section 65B History</span>
                </div>
                <TacticalCommandView theme={theme} isLiveStreaming={isLiveStreaming} />
              </section>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
