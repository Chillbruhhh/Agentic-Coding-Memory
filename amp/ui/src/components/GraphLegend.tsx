import React from 'react';
import { getSymbolStats } from '../utils/graphDataAdapter';
import { GraphNode } from '../utils/graphDataAdapter';

interface GraphLegendProps {
  nodes: GraphNode[];
}

export const GraphLegend: React.FC<GraphLegendProps> = ({ nodes }) => {
  const stats = getSymbolStats(nodes);

  return (
    <div className="bg-panel-dark border border-border-dark rounded p-4 text-slate-200">
      <h3 className="text-sm font-bold text-primary mb-3 uppercase tracking-wider">
        Symbol Types
      </h3>
      
      <div className="space-y-2">
        {stats.map(({ kind, count, color }) => (
          <div key={kind} className="flex items-center justify-between text-xs">
            <div className="flex items-center gap-2">
              <div 
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: color }}
              />
              <span className="capitalize text-slate-300">{kind}</span>
            </div>
            <span className="text-slate-400 font-mono">{count}</span>
          </div>
        ))}
      </div>
      
      <div className="mt-4 pt-3 border-t border-border-dark">
        <div className="flex items-center justify-between text-xs">
          <span className="text-slate-400">Total Symbols</span>
          <span className="text-primary font-mono font-bold">
            {nodes.length}
          </span>
        </div>
      </div>
    </div>
  );
};
