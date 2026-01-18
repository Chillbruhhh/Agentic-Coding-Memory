import React, { useState } from 'react';
import { HiSearch, HiAdjustments, HiEye, HiEyeOff } from 'react-icons/hi';
import { controlPanelTheme } from '../utils/graphTheme';

interface GraphControlsProps {
  onSearch: (query: string) => void;
  onFilterChange: (filters: string[]) => void;
  onResetCamera: () => void;
}

const symbolTypes = ['function', 'class', 'method', 'variable', 'interface'];

export const GraphControls: React.FC<GraphControlsProps> = ({
  onSearch,
  onFilterChange,
  onResetCamera
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [visibleTypes, setVisibleTypes] = useState<string[]>(symbolTypes);
  const [isExpanded, setIsExpanded] = useState(false);

  const handleSearch = (query: string) => {
    setSearchQuery(query);
    onSearch(query);
  };

  const toggleSymbolType = (type: string) => {
    const newVisibleTypes = visibleTypes.includes(type)
      ? visibleTypes.filter(t => t !== type)
      : [...visibleTypes, type];
    
    setVisibleTypes(newVisibleTypes);
    onFilterChange(newVisibleTypes);
  };

  const toggleAll = () => {
    const newVisibleTypes = visibleTypes.length === symbolTypes.length ? [] : symbolTypes;
    setVisibleTypes(newVisibleTypes);
    onFilterChange(newVisibleTypes);
  };

  return (
    <div className={`${controlPanelTheme.background} border ${controlPanelTheme.border} rounded p-4 space-y-4`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-bold text-primary uppercase tracking-wider">
          Graph Controls
        </h3>
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className={`p-1 rounded ${controlPanelTheme.button} transition-colors`}
        >
          <HiAdjustments size={14} />
        </button>
      </div>

      {/* Search */}
      <div className="relative">
        <HiSearch className="absolute left-2 top-1.5 text-slate-500" size={14} />
        <input
          type="text"
          placeholder="Search symbols..."
          value={searchQuery}
          onChange={(e) => handleSearch(e.target.value)}
          className={`w-full pl-8 pr-3 py-1.5 rounded text-xs ${controlPanelTheme.input} transition-colors`}
        />
      </div>

      {/* Expanded controls */}
      {isExpanded && (
        <>
          {/* Symbol type filters */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs text-slate-400 uppercase tracking-wider">
                Symbol Types
              </span>
              <button
                onClick={toggleAll}
                className={`px-2 py-1 rounded text-xs ${controlPanelTheme.button} transition-colors`}
              >
                {visibleTypes.length === symbolTypes.length ? 'Hide All' : 'Show All'}
              </button>
            </div>
            
            <div className="space-y-1">
              {symbolTypes.map(type => (
                <label key={type} className="flex items-center gap-2 text-xs cursor-pointer">
                  <button
                    onClick={() => toggleSymbolType(type)}
                    className="text-slate-400 hover:text-slate-200 transition-colors"
                  >
                    {visibleTypes.includes(type) ? (
                      <HiEye size={14} />
                    ) : (
                      <HiEyeOff size={14} />
                    )}
                  </button>
                  <span className={`capitalize ${visibleTypes.includes(type) ? 'text-slate-300' : 'text-slate-500'}`}>
                    {type}
                  </span>
                </label>
              ))}
            </div>
          </div>

          {/* Camera controls */}
          <div>
            <span className="text-xs text-slate-400 uppercase tracking-wider mb-2 block">
              Camera
            </span>
            <button
              onClick={onResetCamera}
              className={`w-full px-3 py-1.5 rounded text-xs ${controlPanelTheme.button} transition-colors`}
            >
              Reset View
            </button>
          </div>
        </>
      )}
    </div>
  );
};
