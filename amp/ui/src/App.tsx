import React, { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';
import { FileExplorer } from './components/FileExplorer';
import { KnowledgeGraph } from './components/KnowledgeGraph';
import { Analytics } from './components/Analytics';

type ViewType = 'explorer' | 'graph' | 'analytics';

function App() {
  const [activeView, setActiveView] = useState<ViewType>('explorer');

  const renderContent = () => {
    switch (activeView) {
      case 'explorer':
        return <FileExplorer />;
      case 'graph':
        return <KnowledgeGraph />;
      case 'analytics':
        return <Analytics />;
      default:
        return <FileExplorer />;
    }
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-background-dark text-slate-300 selection:bg-red-900 selection:text-white">
      {/* Grid texture background */}
      <div className="fixed inset-0 pointer-events-none opacity-30" style={{
        backgroundImage: 'linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px)',
        backgroundSize: '24px 24px'
      }} />
      
      <Header activeView={activeView} onViewChange={setActiveView} />
      
      <main className="flex-1 flex overflow-hidden relative z-10">
        <Sidebar activeView={activeView} onViewChange={setActiveView} />
        <section className="flex-1 flex flex-col bg-background-dark relative overflow-hidden">
          {renderContent()}
        </section>
      </main>
      
      {/* Footer status bar */}
      <footer className="h-8 border-t border-border-dark bg-panel-dark flex items-center px-4 justify-between font-mono text-[10px] text-slate-500 z-30">
        <div className="flex items-center space-x-6">
          <div className="flex items-center text-amber-500/80">
            <span className="w-2 h-2 rounded-full bg-amber-500 animate-pulse mr-2 shadow-[0_0_8px_rgba(251,191,36,0.6)]"></span>
            <span className="tracking-wider font-bold">SYSTEM ONLINE</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 bg-slate-700 rounded-full"></span>
            UTF-8
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 bg-slate-700 rounded-full"></span>
            TypeScript 5.0.2
          </div>
        </div>
        <div className="flex items-center space-x-4 text-slate-600">
          <span className="hover:text-slate-400 cursor-default">Ln 1, Col 1</span>
          <span className="hover:text-slate-400 cursor-default">Spaces: 2</span>
          <span className="text-primary hover:text-white hover:underline cursor-pointer transition-colors">
            Live Share
          </span>
        </div>
      </footer>
    </div>
  );
}

export default App;
