import { useAnalytics } from '../hooks/useAnalytics';
import React from 'react';
import { HiTrendingUp, HiExclamation } from 'react-icons/hi';
import { BiLineChart } from 'react-icons/bi';

export const Analytics: React.FC = () => {
  const { analytics, loading, error, timeInterval, setTimeInterval } = useAnalytics();
  
  // Process latency data: 1 point per second, fixed 60-second window
  const processLatencyData = () => {
    const rawPoints = analytics?.requestLatency?.dataPoints ?? [];
    if (rawPoints.length === 0) return [];
    
    // Group by second and average
    const pointsBySecond = new Map<number, number[]>();
    rawPoints.forEach(point => {
      const timestamp = new Date(point.timestamp).getTime();
      const secondKey = Math.floor(timestamp / 1000);
      if (!pointsBySecond.has(secondKey)) {
        pointsBySecond.set(secondKey, []);
      }
      pointsBySecond.get(secondKey)!.push(point.latency);
    });
    
    // Convert to array with averaged latencies
    const processedPoints = Array.from(pointsBySecond.entries())
      .map(([secondKey, latencies]) => ({
        timestamp: secondKey * 1000,
        latency: latencies.reduce((a, b) => a + b, 0) / latencies.length
      }))
      .sort((a, b) => a.timestamp - b.timestamp)
      .slice(-60); // Keep last 60 seconds
    
    return processedPoints;
  };
  
  const latencyPoints = processLatencyData();
  
  // Fixed scale for stable rendering (0-200ms)
  const latencyMax = 200;
  const latencyMin = 0;
  const latencyRange = latencyMax - latencyMin;
  
  // Generate path with proper coordinates
  const generateLatencyPath = () => {
    if (latencyPoints.length === 0) return { line: '', area: '', width: 800, height: 100 };
    
    const width = 800;
    const height = 100;
    const points = latencyPoints.map((point, idx) => {
      const x = latencyPoints.length > 1 ? (idx / (latencyPoints.length - 1)) * width : width / 2;
      const clampedLatency = Math.min(point.latency, latencyMax);
      const y = height - ((clampedLatency - latencyMin) / latencyRange) * height;
      return { x, y };
    });
    
    const line = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
    const area = `${line} L ${width} ${height} L 0 ${height} Z`;
    
    return { line, area, width, height };
  };
  
  const { line: latencyPath, area: latencyAreaPath, width: svgWidth, height: svgHeight } = generateLatencyPath();
  
  const latencyTickLabels = latencyPoints.length > 0
    ? latencyPoints
        .filter((_, idx) => idx % Math.ceil(latencyPoints.length / 6) === 0)
        .slice(0, 6)
        .map(point => {
          const time = new Date(point.timestamp);
          return `${time.getHours().toString().padStart(2, '0')}:${time
            .getMinutes()
            .toString()
            .padStart(2, '0')}`;
        })
    : [];

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
      </div>

      {/* Stats cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {[
          { label: 'Total Objects', value: analytics.totalObjects.toString(), change: `${Object.keys(analytics.objectsByType).length} types`, color: 'primary', progress: null, showBar: false },
          { label: 'Relationships', value: analytics.totalRelationships.toString(), change: 'ACTIVE', color: 'slate', progress: null, showBar: false },
          { label: 'CPU Usage', value: `${analytics.systemMetrics.cpuUsage.toFixed(1)}%`, change: analytics.systemMetrics.cpuUsage > 80 ? 'HIGH' : 'NORMAL', color: 'slate', progress: analytics.systemMetrics.cpuUsage, warning: analytics.systemMetrics.cpuUsage > 80, showBar: true },
          { label: 'Memory Usage', value: `${analytics.systemMetrics.memoryUsage.toFixed(1)}%`, change: analytics.systemMetrics.uptime, color: 'slate', progress: analytics.systemMetrics.memoryUsage, warning: analytics.systemMetrics.memoryUsage > 85, showBar: true },
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
            {stat.showBar && stat.progress !== null && (
              <div className="mt-3 h-1.5 w-full bg-stone-900 border border-stone-800 overflow-hidden relative z-10">
                <div className={`h-full ${stat.warning ? 'bg-amber-500 shadow-[0_0_10px_#d97706]' : 'bg-primary shadow-[0_0_10px_#ef4444]'}`} style={{ width: `${stat.progress}%` }}></div>
              </div>
            )}
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
            <span className="text-stone-500 text-xs font-mono border border-stone-800 px-2 py-1 bg-black/20">
              P99: {analytics.requestLatency?.p99 || 142}ms
            </span>
          </div>
          <div className="h-64 relative z-10 px-2">
            {latencyPoints.length === 0 ? (
              <div className="h-full flex items-center justify-center text-stone-600 text-sm">
                No latency data yet
              </div>
            ) : (
              <>
                <svg 
                  viewBox={`0 0 ${svgWidth} ${svgHeight}`}
                  className="w-full h-full" 
                  preserveAspectRatio="none"
                >
                  <defs>
                    <linearGradient id="latencyFill" x1="0" x2="0" y1="0" y2="1">
                      <stop offset="0%" stopColor="#ef4444" stopOpacity="0.2" />
                      <stop offset="100%" stopColor="#ef4444" stopOpacity="0.01" />
                    </linearGradient>
                  </defs>
                  <path
                    d={latencyAreaPath}
                    fill="url(#latencyFill)"
                    stroke="none"
                  />
                  <path
                    d={latencyPath}
                    fill="none"
                    stroke="#ef4444"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
                <div className="mt-3 flex justify-between text-[9px] text-stone-600 font-mono px-2">
                  {latencyTickLabels.map((label, idx) => (
                    <span key={`${label}-${idx}`}>{label}</span>
                  ))}
                </div>
              </>
            )}
          </div>
        </div>

        {/* Object Types Distribution */}
        <div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-6 shadow-lg relative overflow-hidden">
          <div className="absolute inset-0 pointer-events-none opacity-40 mix-blend-overlay" style={{
            backgroundImage: 'url(data:image/svg+xml,%3Csvg width="100" height="100" xmlns="http://www.w3.org/2000/svg"%3E%3Cfilter id="noise"%3E%3CfeTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="4" /%3E%3C/filter%3E%3Crect width="100" height="100" filter="url(%23noise)" opacity="0.05" /%3E%3C/svg%3E)'
          }}></div>
          <div className="flex justify-between items-center mb-8 relative z-10">
            <h3 className="text-lg font-display font-semibold flex items-center gap-2 text-stone-200">
              Object Types
            </h3>
          </div>
          <div className="space-y-4 relative z-10 max-h-[400px] overflow-y-auto pr-2">
            {Object.entries(analytics.objectsByType)
              .sort(([, a], [, b]) => b - a)
              .map(([type, count], idx) => {
                const percent = analytics.totalObjects > 0
                  ? Math.round((count / analytics.totalObjects) * 100)
                  : 0;
                const colors = [
                  'from-primary to-red-600',
                  'from-purple-500 to-purple-900',
                  'from-green-500 to-green-900',
                  'from-amber-500 to-amber-900',
                  'from-blue-500 to-blue-900',
                  'from-pink-500 to-pink-900',
                  'from-cyan-500 to-cyan-900',
                  'from-stone-600 to-stone-700'
                ];
                const color = colors[idx % colors.length];
                
                return (
                  <div key={type} className="space-y-1.5">
                    <div className="flex justify-between text-xs font-mono">
                      <span className="text-stone-400 uppercase tracking-tight">{type}</span>
                      <span className={`font-bold ${
                        idx === 0 ? 'text-primary' : 
                        idx === 1 ? 'text-purple-500' : 
                        idx === 2 ? 'text-green-500' : 
                        idx === 3 ? 'text-amber-500' :
                        'text-stone-500'
                      }`}>
                        {count} ({percent}%)
                      </span>
                    </div>
                    <div className="h-1.5 w-full bg-stone-900 border border-stone-800 overflow-hidden">
                      <div 
                        className={`h-full bg-gradient-to-r ${color} ${idx === 0 ? 'shadow-[0_0_8px_rgba(239,68,68,0.5)]' : ''} transition-all duration-500`} 
                        style={{ width: `${percent}%` }}
                      ></div>
                    </div>
                  </div>
                );
              })}
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
          <span className="text-xs font-mono text-stone-500 uppercase">
            {analytics.systemEvents.length} EVENTS
          </span>
        </div>
        <div className="p-0 relative z-10">
          {analytics.systemEvents.length === 0 ? (
            <div className="px-6 py-8 text-center text-stone-600 text-sm">
              No system events yet
            </div>
          ) : (
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
                {analytics.systemEvents.map((log, idx) => (
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
          )}
        </div>
      </div>
    </div>
  );
};
