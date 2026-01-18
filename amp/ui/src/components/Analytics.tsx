import { useAnalytics } from '../hooks/useAnalytics';
import React from 'react';
import { HiTrendingUp, HiExclamation, HiRefresh } from 'react-icons/hi';
import { BiLineChart } from 'react-icons/bi';

export const Analytics: React.FC = () => {
  const { analytics, loading, error, refetch } = useAnalytics();

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-slate-400">Loading analytics...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-red-400">Error: {error}</div>
      </div>
    );
  }

  if (!analytics) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-slate-400">No analytics data available</div>
      </div>
    );
  }
  return (
    <div className="flex-1 overflow-y-auto p-6 max-w-7xl mx-auto space-y-6 w-full">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-display font-bold text-stone-100 mb-1 uppercase tracking-wider flex items-center gap-2">
            <span className="w-1 h-6 bg-primary inline-block"></span>
            Sector Analytics
          </h2>
          <p className="text-stone-500 text-sm font-mono">:: MONITORING CORE SYSTEMS ::</p>
        </div>
        <div className="flex items-center bg-[#171514] p-1 border border-stone-800 shadow-inner">
          <button className="px-3 py-1.5 text-xs font-bold font-mono transition-all bg-[#2a2522] border border-stone-700 text-primary shadow-sm uppercase">
            1h
          </button>
          <button className="px-3 py-1.5 text-xs font-medium font-mono transition-all text-stone-500 hover:text-stone-300 hover:bg-stone-800 uppercase">
            6h
          </button>
          <button className="px-3 py-1.5 text-xs font-medium font-mono transition-all text-stone-500 hover:text-stone-300 hover:bg-stone-800 uppercase">
            24h
          </button>
          <button className="px-3 py-1.5 text-xs font-medium font-mono transition-all text-stone-500 hover:text-stone-300 hover:bg-stone-800 uppercase">
            7d
          </button>
          <div className="w-px h-4 bg-stone-700 mx-2"></div>
          <button className="px-3 py-1.5 text-xs font-medium rounded-none transition-all text-stone-500 hover:text-stone-300 hover:bg-stone-800 flex items-center gap-1">
            <HiRefresh className="text-sm" />
          </button>
        </div>
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {[
          { label: 'Active Sessions', value: '1,284', change: '+12%', color: 'primary', progress: 65 },
          { label: 'Error Rate', value: '0.08%', change: '0.4%', color: 'amber', progress: 15, warning: true },
          { label: 'CPU Load', value: '42.8%', change: 'STABLE', color: 'slate', progress: 42 },
          { label: 'Memory', value: '4.1GB', change: 'PK 6.2G', color: 'slate', progress: 58 },
        ].map((stat, idx) => (
          <div key={idx} className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-5 border-l-4 border-l-primary shadow-lg relative overflow-hidden">
            <div className="absolute inset-0 pointer-events-none opacity-40 mix-blend-overlay" style={{
              backgroundImage: 'url(data:image/svg+xml,%3Csvg width="100" height="100" xmlns="http://www.w3.org/2000/svg"%3E%3Cfilter id="noise"%3E%3CfeTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="4" /%3E%3C/filter%3E%3Crect width="100" height="100" filter="url(%23noise)" opacity="0.05" /%3E%3C/svg%3E)'
            }}></div>
            <div className="flex justify-between items-start mb-2 relative z-10">
              <span className="text-stone-500 text-xs font-display uppercase tracking-widest">{stat.label}</span>
              <span className={`text-xs font-medium flex items-center ${stat.warning ? 'bg-amber-500/10 px-1 py-0.5 border border-amber-500/20 text-amber-500' : 'text-primary bg-primary/10 px-1 py-0.5 border border-primary/20'}`}>
                {stat.warning ? <HiExclamation className="text-xs mr-1" /> : <HiTrendingUp className="text-xs mr-1" />}
                {stat.change}
              </span>
            </div>
            <div className="text-3xl font-display font-bold text-stone-100 relative z-10" style={{ textShadow: '0 0 10px rgba(239, 68, 68, 0.5)' }}>
              {stat.value}
            </div>
            <div className="mt-3 h-1.5 w-full bg-stone-900 border border-stone-800 overflow-hidden relative z-10">
              <div className={`h-full ${stat.warning ? 'bg-amber-500 shadow-[0_0_10px_#d97706]' : 'bg-primary shadow-[0_0_10px_#ef4444]'}`} style={{ width: `${stat.progress}%` }}></div>
            </div>
          </div>
        ))}
      </div>

      {/* Charts section */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Request Latency Chart */}
        <div className="lg:col-span-2 bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-6 shadow-lg relative overflow-hidden">
          <div className="absolute inset-0 pointer-events-none opacity-40 mix-blend-overlay" style={{
            backgroundImage: 'url(data:image/svg+xml,%3Csvg width="100" height="100" xmlns="http://www.w3.org/2000/svg"%3E%3Cfilter id="noise"%3E%3CfeTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="4" /%3E%3C/filter%3E%3Crect width="100" height="100" filter="url(%23noise)" opacity="0.05" /%3E%3C/svg%3E)'
          }}></div>
          <div className="flex justify-between items-center mb-8 relative z-10">
            <h3 className="text-lg font-display font-semibold flex items-center gap-2 text-stone-200">
              <BiLineChart className="text-primary text-sm" />
              Request Latency
              <span className="text-[10px] px-2 py-0.5 border border-primary/30 text-primary uppercase bg-primary/5">ms</span>
            </h3>
            <span className="text-stone-500 text-xs font-mono border border-stone-800 px-2 py-1 bg-black/20">P99: 142ms</span>
          </div>
          <div className="h-64 flex items-end justify-between gap-1 relative z-10">
            {/* Placeholder for chart */}
            <div className="flex-1 h-full flex items-end justify-center text-slate-600 text-sm">
              Chart visualization would go here
            </div>
          </div>
        </div>

        {/* Error Distribution */}
        <div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-6 shadow-lg relative overflow-hidden">
          <div className="absolute inset-0 pointer-events-none opacity-40 mix-blend-overlay" style={{
            backgroundImage: 'url(data:image/svg+xml,%3Csvg width="100" height="100" xmlns="http://www.w3.org/2000/svg"%3E%3Cfilter id="noise"%3E%3CfeTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="4" /%3E%3C/filter%3E%3Crect width="100" height="100" filter="url(%23noise)" opacity="0.05" /%3E%3C/svg%3E)'
          }}></div>
          <div className="flex justify-between items-center mb-8 relative z-10">
            <h3 className="text-lg font-display font-semibold flex items-center gap-2 text-stone-200">
              Error Distribution
            </h3>
          </div>
          <div className="space-y-6 relative z-10">
            {[
              { label: '404 Not Found', count: 423, percent: 54, color: 'from-primary to-primary-glow' },
              { label: '500 Server Error', count: 188, percent: 24, color: 'from-amber-500 to-amber-900' },
              { label: '401 Unauthorized', count: 92, percent: 12, color: 'from-stone-600 to-stone-700' },
              { label: '403 Forbidden', count: 78, percent: 10, color: 'from-stone-700 to-stone-800' },
            ].map((error, idx) => (
              <div key={idx} className="space-y-2">
                <div className="flex justify-between text-xs font-mono">
                  <span className="text-stone-400 uppercase tracking-tight">{error.label}</span>
                  <span className={`font-bold ${idx === 0 ? 'text-primary' : idx === 1 ? 'text-amber-500' : 'text-stone-500'}`}>
                    {error.count} ({error.percent}%)
                  </span>
                </div>
                <div className="h-2 w-full bg-stone-900 border border-stone-800 overflow-hidden">
                  <div className={`h-full bg-gradient-to-r ${error.color} ${idx === 0 ? 'shadow-[0_0_8px_rgba(239,68,68,0.5)]' : idx === 1 ? 'shadow-[0_0_8px_rgba(217,119,6,0.4)]' : ''}`} style={{ width: `${error.percent}%` }}></div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* System Events Log */}
      <div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 overflow-hidden shadow-lg">
        <div className="px-6 py-4 border-b border-stone-800 flex justify-between items-center relative z-10 bg-[#151210]">
          <h3 className="font-display font-semibold flex items-center gap-2 text-stone-200">
            <span className="w-2 h-2 bg-primary animate-pulse shadow-[0_0_8px_#ef4444]"></span>
            System Events Log
          </h3>
          <span className="text-xs font-mono text-stone-500 uppercase flex items-center gap-2">
            <span className="w-1.5 h-1.5 rounded-full bg-green-900"></span>
            Live Feed
          </span>
        </div>
        <div className="p-0 relative z-10">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-stone-900/80 text-stone-500 uppercase tracking-wider border-b border-stone-800">
              <tr>
                <th className="px-6 py-3 font-normal">Timestamp</th>
                <th className="px-6 py-3 font-normal">Event Descriptor</th>
                <th className="px-6 py-3 font-normal">Origin</th>
                <th className="px-6 py-3 font-normal">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-stone-800/50">
              {[
                { time: '15:02:44.201', event: 'Worker node "cluster-a-1" scaling up', origin: 'K8s Controller', status: 'Info', alert: false },
                { time: '15:01:12.873', event: 'Database connection pool exhausted', origin: 'Postgres Connector', status: 'Alert', alert: true },
                { time: '14:58:30.005', event: 'Backup sequence initiated for "prod-main"', origin: 'Storage Agent', status: 'Success', alert: false },
              ].map((log, idx) => (
                <tr key={idx} className={`hover:bg-stone-800/30 transition-colors group ${log.alert ? 'hover:bg-primary/5 border-l-2 border-l-primary bg-primary/5' : ''}`}>
                  <td className={`px-6 py-4 ${log.alert ? 'text-primary group-hover:text-red-400' : 'text-stone-400 group-hover:text-stone-300'}`}>
                    {log.time}
                  </td>
                  <td className={`px-6 py-4 ${log.alert ? 'text-stone-200 group-hover:text-white font-bold' : 'text-stone-300 group-hover:text-white'}`}>
                    {log.event}
                  </td>
                  <td className="px-6 py-4 text-stone-500">{log.origin}</td>
                  <td className="px-6 py-4">
                    <span className={`px-2 py-0.5 border uppercase text-[10px] tracking-wide ${
                      log.alert
                        ? 'border-primary bg-primary text-black font-bold shadow-[0_0_10px_rgba(239,68,68,0.4)] animate-pulse'
                        : log.status === 'Success'
                        ? 'border-green-900 bg-green-900/20 text-green-700'
                        : 'border-stone-600 bg-stone-800 text-stone-400'
                    }`}>
                      {log.status}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};
