import React from 'react';
import { HiFolder, HiLightningBolt } from 'react-icons/hi';
import { GiNetworkBars } from 'react-icons/gi';
import { GoWorkflow } from 'react-icons/go';
import { SiGraphql } from 'react-icons/si';
import { IoNotifications } from 'react-icons/io5';

type ViewType = 'explorer' | 'graph' | 'sessions' | 'analytics';

interface HeaderProps {
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
}

export const Header: React.FC<HeaderProps> = ({ activeView, onViewChange }) => {
  const tabs = [
    { id: 'explorer' as ViewType, icon: HiFolder, label: 'File Explorer' },
    { id: 'graph' as ViewType, icon: SiGraphql, label: 'Knowledge Graph' },
    { id: 'sessions' as ViewType, icon: GoWorkflow, label: 'Sessions' },
    { id: 'analytics' as ViewType, icon: GiNetworkBars, label: 'Analytics' },
  ];

  return (
    <header className="h-14 border-b border-border-dark flex items-center justify-between px-6 bg-panel-dark/80 backdrop-blur-md relative z-20 shadow-lg shadow-black/40">
      <div className="flex items-center space-x-6">
        {/* Brand */}
        <div className="flex items-center space-x-2 group cursor-pointer">
          <HiLightningBolt className="text-primary group-hover:text-white transition-colors w-5 h-5" />
          <span className="font-display font-bold text-lg tracking-tight text-white group-hover:text-primary transition-colors">
            AMP<span className="text-primary">Console</span>
          </span>
        </div>
        
        {/* Tab Navigation */}
        <nav className="ml-8 flex space-x-1">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            const isActive = activeView === tab.id;
            
            return (
              <button
                key={tab.id}
                onClick={() => onViewChange(tab.id)}
                className={`relative px-4 py-3 text-sm font-medium transition-all flex items-center gap-2 ${
                  isActive
                    ? 'text-primary border-b-2 border-primary bg-gradient-to-t from-red-900/20 to-transparent'
                    : 'text-slate-400 hover:text-white hover:bg-white/5 border-b-2 border-transparent'
                }`}
                style={isActive ? { textShadow: '0 0 10px rgba(239, 68, 68, 0.5)' } : {}}
              >
                <Icon className={`text-lg ${isActive ? '' : 'text-slate-500'}`} />
                {tab.label}
              </button>
            );
          })}
        </nav>
      </div>
      
      {/* Right side */}
      <div className="flex items-center space-x-5">
        {/* Project indicator */}
        <div className="flex items-center space-x-2 text-xs font-mono bg-black/40 px-3 py-1 rounded border border-border-dark shadow-inner">
          <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse shadow-[0_0_8px_rgba(239,68,68,0.6)]"></span>
          <span className="text-red-100/80">PROJ-RUST-01</span>
        </div>
        
        {/* Notifications */}
        <button className="p-2 rounded hover:bg-white/5 text-slate-400 hover:text-white transition-colors relative">
          <IoNotifications className="w-5 h-5" />
          <span className="absolute top-2 right-2 w-2 h-2 bg-primary rounded-full ring-2 ring-panel-dark"></span>
        </button>
        
        {/* User avatar */}
        <div className="w-8 h-8 rounded bg-gradient-to-br from-red-900 to-black border border-red-900/50 flex items-center justify-center text-red-200 font-bold text-xs shadow-lg">
          AD
        </div>
      </div>
    </header>
  );
};
