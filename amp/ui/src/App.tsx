import { useState } from 'react';
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
  const [pendingProjectId, setPendingProjectId] = useState<string | undefined>(undefined);
  const [loading, _setLoading] = useState(false);

  // Switching views via the sidebar clears any pending project filter so the
  // next graph view opens with "All projects" by default.
  const handleViewChange = (view: ViewType) => {
    setPendingProjectId(undefined);
    setActiveView(view);
  };

  // Codebase cards can deep-link into the graph with a specific project pre-selected.
  const handleNavigateToGraph = (projectId?: string) => {
    setPendingProjectId(projectId);
    setActiveView('graph');
  };

  const renderContent = () => {
    switch (activeView) {
      case 'explorer':
        return <FileExplorer onNavigateToGraph={handleNavigateToGraph} />;
      case 'graph':
        return <KnowledgeGraph initialProjectId={pendingProjectId} />;
      case 'artifacts':
        return <Artifacts />;
      case 'sessions':
        return <Sessions />;
      case 'analytics':
        return <Analytics />;
      case 'settings':
        return <Settings />;
      default:
        return <FileExplorer onNavigateToGraph={handleNavigateToGraph} />;
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
        <Sidebar activeView={activeView} onViewChange={handleViewChange} />
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
