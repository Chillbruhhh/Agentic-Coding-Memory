import React from 'react';
import { HiMinus, HiOutlineSquare2Stack, HiXMark } from 'react-icons/hi2';

export const CustomTitleBar: React.FC = () => {
  const handleMinimize = async () => {
    try {
      const { getCurrent } = await import('@tauri-apps/api/window');
      const appWindow = getCurrent();
      await appWindow.minimize();
    } catch (error) {
      console.error('Failed to minimize:', error);
    }
  };

  const handleMaximize = async () => {
    try {
      const { getCurrent } = await import('@tauri-apps/api/window');
      const appWindow = getCurrent();
      await appWindow.toggleMaximize();
    } catch (error) {
      console.error('Failed to maximize:', error);
    }
  };

  const handleClose = async () => {
    try {
      const { getCurrent } = await import('@tauri-apps/api/window');
      const appWindow = getCurrent();
      await appWindow.close();
    } catch (error) {
      console.error('Failed to close:', error);
    }
  };

  return (
    <div 
      className="h-8 bg-panel-dark border-b border-border-dark flex items-center justify-between px-4 select-none"
      data-tauri-drag-region
    >
      {/* Left side - App title */}
      <div className="flex items-center gap-3">
        <img 
          src="/logo/amp-favicon.png" 
          alt="AMP Logo" 
          className="w-4 h-4 object-contain"
        />
        <span className="text-xs font-bold text-slate-200 uppercase tracking-wider">
          Agentic Memory Protocol
        </span>
      </div>

      {/* Right side - Window controls */}
      <div className="flex items-center">
        <button
          onClick={handleMinimize}
          className="w-8 h-8 flex items-center justify-center hover:bg-white/10 transition-colors text-slate-400 hover:text-slate-200"
          title="Minimize"
          type="button"
        >
          <HiMinus size={14} />
        </button>
        <button
          onClick={handleMaximize}
          className="w-8 h-8 flex items-center justify-center hover:bg-white/10 transition-colors text-slate-400 hover:text-slate-200"
          title="Maximize"
          type="button"
        >
          <HiOutlineSquare2Stack size={12} />
        </button>
        <button
          onClick={handleClose}
          className="w-8 h-8 flex items-center justify-center hover:bg-red-600 transition-colors text-slate-400 hover:text-white"
          title="Close"
          type="button"
        >
          <HiXMark size={14} />
        </button>
      </div>
    </div>
  );
};
