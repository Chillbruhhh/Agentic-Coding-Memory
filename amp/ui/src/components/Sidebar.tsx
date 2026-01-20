import React from 'react';
import { HiChartBar } from 'react-icons/hi';
import { FaFolder, FaFolderOpen } from 'react-icons/fa';
import { BiNetworkChart } from 'react-icons/bi';
import { PiGearFineLight } from 'react-icons/pi';

type ViewType = 'explorer' | 'graph' | 'analytics' | 'settings';

interface SidebarProps {
  activeView: ViewType;
  onViewChange: (view: ViewType) => void;
}

export const Sidebar: React.FC<SidebarProps> = ({ activeView, onViewChange }) => {
  const navItems = [
    { id: 'explorer' as ViewType, icon: FaFolder, openIcon: FaFolderOpen, label: 'File Explorer' },
    { id: 'graph' as ViewType, icon: BiNetworkChart, label: 'Knowledge Graph' },
    { id: 'analytics' as ViewType, icon: HiChartBar, label: 'Analytics' },
    { id: 'settings' as ViewType, icon: PiGearFineLight, label: 'Settings' },
  ];

  return (
    <aside className="w-16 border-r border-border-dark bg-panel-dark flex flex-col items-center py-6 gap-6 z-40 relative">
      {/* Subtle gradient overlay */}
      <div className="absolute inset-0 pointer-events-none bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-white/5 via-transparent to-transparent opacity-20"></div>
      
      {navItems.map((item) => {
        const isActive = activeView === item.id;
        // Use openIcon if available and active, otherwise use regular icon
        const Icon = isActive && item.openIcon ? item.openIcon : item.icon;
        
        return (
          <button
            key={item.id}
            onClick={() => onViewChange(item.id)}
            className={`group relative p-3 rounded transition-all ${
              isActive
                ? 'bg-red-950/40 text-primary shadow-[0_0_10px_rgba(239,68,68,0.1)] border border-primary/30'
                : 'text-slate-500 hover:text-primary hover:bg-red-900/20'
            }`}
          >
            <Icon className="w-5 h-5" />
            
            {/* Tooltip */}
            <span className="absolute left-16 bg-red-950 border border-red-900 text-red-100 text-xs py-1 px-2 rounded opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity whitespace-nowrap z-50">
              {item.label}
            </span>
          </button>
        );
      })}
    </aside>
  );
};
