import React, { useState, useEffect } from 'react';

interface StatusBarProps {
  activeView: string;
  loading?: boolean;
}

export const StatusBar: React.FC<StatusBarProps> = ({ activeView, loading = false }) => {
  const [serverStatus, setServerStatus] = useState<'online' | 'offline' | 'checking'>('checking');
  const [dbStatus, setDbStatus] = useState<'connected' | 'disconnected' | 'checking'>('checking');
  const [language, setLanguage] = useState<string>('TypeScript');

  // Check server and database status
  useEffect(() => {
    const checkStatus = async () => {
      try {
        const response = await fetch('http://localhost:8105/health', {
          method: 'GET',
          signal: AbortSignal.timeout(2000)
        });
        
        if (response.ok) {
          setServerStatus('online');
          setDbStatus('connected');
        } else {
          setServerStatus('offline');
          setDbStatus('disconnected');
        }
      } catch (error) {
        setServerStatus('offline');
        setDbStatus('disconnected');
      }
    };

    checkStatus();
    const interval = setInterval(checkStatus, 10000); // Check every 10 seconds

    return () => clearInterval(interval);
  }, []);

  // Update language based on active view
  useEffect(() => {
    switch (activeView) {
      case 'explorer':
        setLanguage('TypeScript');
        break;
      case 'graph':
        setLanguage('3D Force');
        break;
      case 'sessions':
        setLanguage('Run Log');
        break;
      case 'analytics':
        setLanguage('React');
        break;
      default:
        setLanguage('TypeScript');
    }
  }, [activeView]);

  const getViewLabel = () => {
    switch (activeView) {
      case 'explorer':
        return 'File Explorer';
      case 'graph':
        return 'Knowledge Graph';
      case 'sessions':
        return 'Sessions';
      case 'analytics':
        return 'Analytics';
      default:
        return 'Unknown';
    }
  };

  return (
    <footer className="h-8 border-t border-border-dark bg-black/95 flex items-center px-4 justify-between font-mono text-[11px] z-30">
      <div className="flex items-center space-x-8">
        {/* Server Status */}
        <div className="flex items-center gap-2">
          <span className="tracking-[0.15em] font-bold uppercase text-white">
            SERVER:
          </span>
          <span className={`tracking-[0.15em] font-bold uppercase ${
            serverStatus === 'online' ? 'text-green-400' : 
            serverStatus === 'offline' ? 'text-red-400' : 
            'text-yellow-400'
          }`}>
            {serverStatus === 'online' ? 'ONLINE' : serverStatus === 'offline' ? 'OFFLINE' : 'CHECKING'}
          </span>
        </div>

        {/* Database Status */}
        <div className="flex items-center gap-2">
          <span className="tracking-[0.15em] font-bold uppercase text-white">
            DATABASE:
          </span>
          <span className={`tracking-[0.15em] font-bold uppercase ${
            dbStatus === 'connected' ? 'text-purple-400' : 
            dbStatus === 'disconnected' ? 'text-orange-400' : 
            'text-yellow-400'
          }`}>
            {dbStatus === 'connected' ? 'CONNECTED' : dbStatus === 'disconnected' ? 'DISCONNECTED' : 'CHECKING'}
          </span>
        </div>

        {/* Separator */}
        <div className="h-4 w-px bg-slate-700"></div>

        {/* Language/Technology */}
        <div className="flex items-center gap-2 text-slate-400">
          <span className="w-1.5 h-1.5 bg-slate-600 rounded-full"></span>
          <span className="tracking-wider">{language}</span>
        </div>

        {/* Current View */}
        <div className="flex items-center gap-2 text-slate-400">
          <span className="w-1.5 h-1.5 bg-slate-600 rounded-full"></span>
          <span className="tracking-wider">{getViewLabel()}</span>
        </div>

        {/* Loading Indicator */}
        {loading && (
          <div className="flex items-center gap-2">
            <span className="tracking-[0.15em] font-bold uppercase text-white">LOADING:</span>
            <span className="tracking-[0.15em] font-bold uppercase text-blue-400">ACTIVE</span>
          </div>
        )}
      </div>

      <div className="flex items-center space-x-6 text-slate-500">
        <span className="hover:text-slate-400 cursor-default tracking-wider">UTF-8</span>
        <span className="hover:text-slate-400 cursor-default tracking-wider">CRLF</span>
        <span className="text-primary/80 hover:text-primary hover:underline cursor-pointer transition-colors tracking-wider font-bold">
          AMP v1.0.0
        </span>
      </div>
    </footer>
  );
};

