import React, { useState } from 'react';
import type { DashboardView, DashboardTheme } from './types';
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';
import { CommandCenterView } from './components/CommandCenterView';
import { FlowPipelineView } from './components/FlowPipelineView';
import { ExecutiveAnalyticsView } from './components/ExecutiveAnalyticsView';
import { TacticalCommandView } from './components/TacticalCommandView';
import { LiveBackendBridge } from './components/LiveBackendBridge';
import { GatekeeperHITLView } from './components/GatekeeperHITLView';
import { DLQInspectorView } from './components/DLQInspectorView';
import { VRLPlaygroundView } from './components/VRLPlaygroundView';
import { ByteAccountingView } from './components/ByteAccountingView';
import { ExecutiveBriefing, type SystemScenario } from './components/ExecutiveBriefing';
import { DashboardExplainerModal } from './components/DashboardExplainerModal';

export function App() {
  const [currentView, setCurrentView] = useState<DashboardView>('command');
  const [theme, setTheme] = useState<DashboardTheme>('dark');
  const [isLiveStreaming, setIsLiveStreaming] = useState<boolean>(true);
  const [scenario, setScenario] = useState<SystemScenario>('baseline');
  const [isExplainerOpen, setIsExplainerOpen] = useState<boolean>(false);
  const [isBackendLive, setIsBackendLive] = useState<boolean>(false);
  const [liveEps, setLiveEps] = useState<number>(312480);

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
      {/* Architecture & Compliance Explainer Modal */}
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

        <main className="flex-1 p-6 space-y-6 max-w-[1650px] w-full mx-auto">
          {/* Executive Intelligence Advisory: Explains operational posture and active scenario */}
          <ExecutiveBriefing
            theme={theme}
            scenario={scenario}
            onScenarioChange={setScenario}
            onOpenExplainer={() => setIsExplainerOpen(true)}
            onOpenGatekeeper={() => setCurrentView('gatekeeper')}
            isBackendLive={isBackendLive}
            eps={liveEps}
          />

          {/* Real Backend Bridge & Live Traffic Injector */}
          <LiveBackendBridge
            theme={theme}
            onLiveMetricsUpdate={(metrics) => {
              setIsBackendLive(true);
              if (metrics.eps > 0) setLiveEps(metrics.eps);
            }}
          />

          {/* Flagship Operational Workspace 1: Master SOC Command Center */}
          {currentView === 'command' && (
            <CommandCenterView
              theme={theme}
              isLiveStreaming={isLiveStreaming}
              onOpenGatekeeper={() => setCurrentView('gatekeeper')}
              onOpenDLQ={() => setCurrentView('dlq')}
              onOpenVrl={() => setCurrentView('vrl')}
              onOpenAccounting={() => setCurrentView('accounting')}
            />
          )}

          {/* Deep-Dive Workspace 2: Pipeline Topology & Dynamic Normalization */}
          {currentView === 'flow' && (
            <FlowPipelineView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {/* Deep-Dive Workspace 3: Engine Telemetry & Performance SLAs */}
          {currentView === 'analytics' && (
            <ExecutiveAnalyticsView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {/* Deep-Dive Workspace 4: Tactical Threat Radar & Ballistics */}
          {currentView === 'threats' && (
            <TacticalCommandView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {/* Deep-Dive Workspace 5: Byte Accounting & Section 65B Evidence Vault (§3.1) */}
          {currentView === 'accounting' && (
            <ByteAccountingView theme={theme} />
          )}

          {/* Deep-Dive Workspace 6: Autonomous AI Gatekeeper (HitL) */}
          {currentView === 'gatekeeper' && (
            <GatekeeperHITLView theme={theme} onTestRule={handleTestRuleInVrl} />
          )}

          {/* Deep-Dive Workspace 7: Dead Letter Queue (DLQ) Quarantine */}
          {currentView === 'dlq' && (
            <DLQInspectorView theme={theme} onSendToVrl={handleSendToVrl} />
          )}

          {/* Deep-Dive Workspace 8: Interactive VRL Sandbox & Validator */}
          {currentView === 'vrl' && (
            <VRLPlaygroundView
              theme={theme}
              initialPayload={vrlInitialPayload}
              initialRule={vrlInitialRule}
            />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
