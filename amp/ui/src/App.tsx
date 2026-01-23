import React, { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { FileExplorer } from './components/FileExplorer';
import { KnowledgeGraph } from './components/KnowledgeGraph';
import { Sessions } from './components/Sessions';
import { Artifacts } from './components/Artifacts';
import { Analytics } from './components/Analytics';
import { Settings } from './components/Settings';
import { CustomTitleBar } from './components/CustomTitleBar';
import { StatusBar } from './components/StatusBar';

type ViewType = 'explorer' | 'graph' | 'artifacts' | 'sessions' | 'analytics' | 'settings';

function App() {
  const [activeView, setActiveView] = useState<ViewType>('explorer');
  const [loading, setLoading] = useState(false);

  const renderContent = () => {
    switch (activeView) {
      case 'explorer':
        return <FileExplorer onNavigateToGraph={() => setActiveView('graph')} />;
      case 'graph':
        return <KnowledgeGraph />;
      case 'artifacts':
        return <Artifacts />;
      case 'sessions':
        return <Sessions />;
      case 'analytics':
        return <Analytics />;
      case 'settings':
        return <Settings />;
      default:
        return <FileExplorer onNavigateToGraph={() => setActiveView('graph')} />;
    }
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-background-dark text-slate-300 selection:bg-red-900 selection:text-white">
      {/* Custom Title Bar */}
      <CustomTitleBar />
      
      {/* Grid texture background */}
      <div className="fixed inset-0 pointer-events-none opacity-30 top-8" style={{
        backgroundImage: 'linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px), linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px)',
        backgroundSize: '24px 24px'
      }} />
      
      <main className="flex-1 flex overflow-hidden relative z-10">
        <Sidebar activeView={activeView} onViewChange={setActiveView} />
        <section className="flex-1 flex flex-col bg-background-dark relative overflow-hidden">
          {renderContent()}
        </section>
      </main>
      
      {/* Footer status bar */}
      <StatusBar activeView={activeView} loading={loading} />
    </div>
  );
}

export default App;
