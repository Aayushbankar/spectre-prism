import React, { useState, useEffect } from 'react';
import type { DashboardView, DashboardTheme } from './types';
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';
import { FlowPipelineView } from './components/FlowPipelineView';
import { ExecutiveAnalyticsView } from './components/ExecutiveAnalyticsView';
import { TacticalCommandView } from './components/TacticalCommandView';

export function App() {
  const [currentView, setCurrentView] = useState<DashboardView>('all');
  const [theme, setTheme] = useState<DashboardTheme>('dark');
  const [isLiveStreaming, setIsLiveStreaming] = useState<boolean>(true);
  const [streamTick, setStreamTick] = useState<number>(0);

  // Background simulation ticker for live EPS & events
  useEffect(() => {
    if (!isLiveStreaming) return;
    const interval = setInterval(() => {
      setStreamTick((prev) => prev + 1);
    }, 2000);
    return () => clearInterval(interval);
  }, [isLiveStreaming]);

  const isCyber = theme === 'cyber';
  const isLight = theme === 'light';
  const isDark = theme === 'dark';

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
      {/* Left Sidebar */}
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
        />

        <main className="flex-1 p-6 space-y-8 max-w-[1600px] w-full mx-auto">
          {/* View Switch Rendering */}
          {currentView === 'pipeline' && (
            <FlowPipelineView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {currentView === 'analytics' && (
            <ExecutiveAnalyticsView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {currentView === 'tactical' && (
            <TacticalCommandView theme={theme} isLiveStreaming={isLiveStreaming} />
          )}

          {currentView === 'all' && (
            <div className="space-y-12">
              {/* Design 1 Section */}
              <section className="space-y-3">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded text-[11px] font-bold bg-cyan-500/20 text-cyan-400 border border-cyan-500/30">
                      DESIGN 1
                    </span>
                    <h2 className="text-lg font-bold">Traffic Flow & Pipeline Overview</h2>
                  </div>
                  <span className="text-xs text-slate-500">Adapted from ideainso3.jpeg</span>
                </div>
                <FlowPipelineView theme={theme} isLiveStreaming={isLiveStreaming} />
              </section>

              {/* Design 2 Section */}
              <section className="space-y-3">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded text-[11px] font-bold bg-purple-500/20 text-purple-400 border border-purple-500/30">
                      DESIGN 2
                    </span>
                    <h2 className="text-lg font-bold">Executive Analytics & Real-Time Telemetry</h2>
                  </div>
                  <span className="text-xs text-slate-500">Adapted from dashboard2.jpeg</span>
                </div>
                <ExecutiveAnalyticsView theme={theme} isLiveStreaming={isLiveStreaming} />
              </section>

              {/* Design 3 Section */}
              <section className="space-y-3">
                <div className="flex items-center justify-between border-b border-slate-800 pb-2">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded text-[11px] font-bold bg-amber-500/20 text-amber-400 border border-amber-500/30">
                      DESIGN 3
                    </span>
                    <h2 className="text-lg font-bold">Tactical Cyber Command & Threat Intelligence</h2>
                  </div>
                  <span className="text-xs text-slate-500">Adapted from dashboard_1.jpeg</span>
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
