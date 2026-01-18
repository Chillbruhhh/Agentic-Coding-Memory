import React from 'react';
import { FileNode } from '../hooks/useCodebases';

interface FileContentViewerProps {
  file: FileNode | null;
}

export const FileContentViewer: React.FC<FileContentViewerProps> = ({ file }) => {
  if (!file) {
    return (
      <div className="flex-1 flex items-center justify-center bg-background-dark text-slate-400">
        <div className="text-center">
          <div className="text-4xl mb-4">📄</div>
          <div>Select a file to view its contents</div>
        </div>
      </div>
    );
  }

  if (file.type === 'folder') {
    return (
      <div className="flex-1 flex items-center justify-center bg-background-dark text-slate-400">
        <div className="text-center">
          <div className="text-4xl mb-4">📁</div>
          <div>Folder selected: {file.name}</div>
          <div className="text-sm mt-2">
            {file.children?.length || 0} items
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 bg-background-dark border-l border-border-dark">
      {/* File Header */}
      <div className="border-b border-border-dark p-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-bold text-primary">{file.name}</h3>
            <p className="text-sm text-slate-400">{file.path}</p>
          </div>
          <div className="flex items-center space-x-2">
            {file.language && (
              <span className="px-2 py-1 bg-primary/10 text-primary text-xs rounded">
                {file.language}
              </span>
            )}
            {file.size && (
              <span className="text-xs text-slate-500">{file.size}</span>
            )}
          </div>
        </div>
      </div>

      {/* Symbols List */}
      <div className="p-4 overflow-y-auto" style={{ maxHeight: 'calc(100vh - 250px)' }}>
        <h4 className="text-sm font-bold text-slate-300 mb-3 uppercase tracking-wider">
          Symbols ({file.symbols?.length || 0})
        </h4>
        
        {file.symbols && file.symbols.length > 0 ? (
          <div className="space-y-2">
            {file.symbols.map((symbol, index) => (
              <div
                key={index}
                className="flex items-center space-x-3 p-2 bg-panel-dark rounded border border-border-dark hover:border-primary/50 transition-colors"
              >
                <div className="w-2 h-2 rounded-full bg-primary"></div>
                <div className="flex-1">
                  <div className="flex items-center space-x-2">
                    <span className="font-mono text-slate-200">{symbol.name}</span>
                    <span className="px-1.5 py-0.5 bg-slate-700 text-slate-300 text-xs rounded">
                      {symbol.type}
                    </span>
                  </div>
                  {symbol.signature && (
                    <div className="text-xs text-slate-400 font-mono mt-1">
                      {symbol.signature}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-slate-500 text-sm">No symbols found in this file</div>
        )}
      </div>
    </div>
  );
};
