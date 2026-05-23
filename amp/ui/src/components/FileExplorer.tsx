import React, { useState } from 'react';
import { HiFolder, HiFolderOpen, HiChevronRight, HiChevronDown, HiSearch, HiDocumentText, HiCode, HiX } from 'react-icons/hi';
import { BiFile, BiGitBranch } from 'react-icons/bi';
import { SiGraphql } from 'react-icons/si';
import { GiTrashCan } from 'react-icons/gi';
import { useCodebases, CodebaseProject, FileNode } from '../hooks/useCodebases';
import { KnowledgeGraphModal } from './KnowledgeGraphModal';
import { FileContentViewer } from './FileContentViewer';

const basenameOf = (p: string): string => {
  if (!p) return p;
  const cleaned = p.replace(/[\\/]+$/, '');
  const idx = Math.max(cleaned.lastIndexOf('/'), cleaned.lastIndexOf('\\'));
  return idx === -1 ? cleaned : cleaned.slice(idx + 1);
};

const displayName = (node: FileNode): string => {
  const raw = (node.name || '').trim();
  // If name field happens to be the full path (Windows absolute paths from the
  // indexer), strip down to the basename so the tree reads like an IDE.
  if (raw.includes('/') || raw.includes('\\') || raw.length > 80) {
    return basenameOf(node.path) || basenameOf(raw) || raw;
  }
  return raw;
};

interface FileTreeModalProps {
  codebase: CodebaseProject;
  onClose: () => void;
}

const FileTreeModal: React.FC<FileTreeModalProps> = ({ codebase, onClose }) => {
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set([codebase.path]));
  const [openTabs, setOpenTabs] = useState<FileNode[]>([]);
  const [activeTabPath, setActiveTabPath] = useState<string | null>(null);

  const activeFile = openTabs.find(t => t.path === activeTabPath) || null;

  const openFile = (node: FileNode) => {
    if (node.type !== 'file') return;
    setOpenTabs(prev => (prev.some(t => t.path === node.path) ? prev : [...prev, node]));
    setActiveTabPath(node.path);
  };

  const closeTab = (path: string) => {
    setOpenTabs(prev => {
      const idx = prev.findIndex(t => t.path === path);
      if (idx === -1) return prev;
      const next = prev.filter(t => t.path !== path);
      if (activeTabPath === path) {
        const fallback = next[idx] || next[idx - 1] || null;
        setActiveTabPath(fallback?.path || null);
      }
      return next;
    });
  };

  const toggleFolder = (path: string) => {
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(path)) {
      newExpanded.delete(path);
    } else {
      newExpanded.add(path);
    }
    setExpandedFolders(newExpanded);
  };

  const getFileIcon = (node: FileNode) => {
    if (node.type === 'folder') {
      return expandedFolders.has(node.path) ? <HiFolderOpen /> : <HiFolder />;
    }
    
    const ext = node.name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'rs': return <HiCode className="text-orange-400" />;
      case 'tsx':
      case 'ts': return <HiCode className="text-blue-400" />;
      case 'js':
      case 'jsx': return <HiCode className="text-yellow-400" />;
      case 'md': return <HiDocumentText className="text-green-400" />;
      case 'json': return <BiFile className="text-yellow-400" />;
      default: return <BiFile />;
    }
  };

  const renderFileNode = (node: FileNode, depth: number = 0) => {
    const isExpanded = expandedFolders.has(node.path);
    const isSelected = activeTabPath === node.path;

    return (
      <div key={node.path}>
        <div
          className={`flex items-center gap-1.5 py-0.5 px-2 cursor-pointer hover:bg-panel-dark transition-colors min-w-0 ${
            isSelected ? 'bg-primary/10 border-l-2 border-primary' : ''
          }`}
          style={{ paddingLeft: `${depth * 12 + 8}px` }}
          onClick={() => {
            if (node.type === 'folder') {
              toggleFolder(node.path);
            } else {
              openFile(node);
            }
          }}
        >
          {node.type === 'folder' ? (
            <span className="text-slate-400 shrink-0">
              {isExpanded ? <HiChevronDown size={14} /> : <HiChevronRight size={14} />}
            </span>
          ) : (
            <span className="w-[14px] shrink-0" />
          )}

          <div className="text-slate-400 shrink-0">
            {getFileIcon(node)}
          </div>

          <span
            className={`text-[13px] truncate flex-1 min-w-0 ${isSelected ? 'text-primary font-medium' : 'text-slate-300'}`}
            title={node.path}
          >
            {displayName(node)}
          </span>

          {node.type === 'file' && node.symbols && node.symbols.length > 0 && (
            <span className="text-[10px] text-slate-500 shrink-0" title={`${node.symbols.length} symbols`}>
              {node.symbols.length}
            </span>
          )}
        </div>
        
        {node.type === 'folder' && isExpanded && node.children && (
          <div>
            {node.children.map(child => renderFileNode(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };



  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-2 sm:p-4">
      <div className="bg-panel-dark border border-border-dark rounded-lg w-[98vw] h-[94vh] flex flex-col overflow-hidden">
        {/* Modal Header */}
        <div className="px-4 py-3 border-b border-border-dark bg-black/20 flex items-center justify-between">
          <div className="min-w-0">
            <h2 className="text-lg font-bold text-slate-200 truncate">{codebase.name}</h2>
            <p className="text-xs text-slate-400 truncate">{codebase.description}</p>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-white/5 rounded text-slate-400 hover:text-slate-200 transition-colors"
          >
            <HiX size={20} />
          </button>
        </div>

        {/* Modal Content - IDE-style layout */}
        <div className="flex-1 flex min-h-0 overflow-hidden">
          {/* File Tree Sidebar */}
          <div className="w-72 shrink-0 border-r border-border-dark flex flex-col bg-black/20">
            <div className="px-3 py-2 border-b border-border-dark bg-black/30 flex items-center justify-between">
              <span className="text-xs font-bold uppercase tracking-wider text-slate-300">Explorer</span>
              <span className="text-[11px] text-slate-500">{codebase.total_files} files</span>
            </div>
            <div className="flex-1 overflow-y-auto py-1 text-sm">
              {codebase.file_tree.map(node => renderFileNode(node))}
            </div>
          </div>

          {/* Editor area: tabs + content */}
          <div className="flex-1 flex flex-col min-w-0">
            {/* Tab bar */}
            <div className="flex items-end border-b border-border-dark bg-black/40 min-h-[36px] overflow-x-auto">
              {openTabs.length === 0 ? (
                <div className="px-4 py-2 text-xs text-slate-500">No open files</div>
              ) : (
                openTabs.map(tab => {
                  const active = tab.path === activeTabPath;
                  return (
                    <div
                      key={tab.path}
                      onClick={() => setActiveTabPath(tab.path)}
                      className={`group flex items-center gap-2 px-3 py-2 border-r border-border-dark cursor-pointer select-none text-xs transition-colors shrink-0 ${
                        active
                          ? 'bg-background-dark text-slate-100 border-b-2 border-b-primary -mb-px'
                          : 'bg-black/20 text-slate-400 hover:text-slate-200 hover:bg-black/30'
                      }`}
                      title={tab.path}
                    >
                      <span className="font-mono truncate max-w-[180px]">{displayName(tab)}</span>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          closeTab(tab.path);
                        }}
                        className="text-slate-500 hover:text-slate-200 opacity-60 group-hover:opacity-100 transition-opacity"
                        aria-label={`Close ${tab.name}`}
                      >
                        <HiX size={14} />
                      </button>
                    </div>
                  );
                })
              )}
            </div>

            {/* Editor content */}
            <div className="flex-1 flex min-h-0">
              {activeFile ? (
                <FileContentViewer file={activeFile} />
              ) : (
                <div className="flex-1 flex items-center justify-center text-slate-400">
                  <div className="text-center">
                    <HiDocumentText size={48} className="mx-auto mb-4 opacity-50" />
                    <p>Select a file to view its content</p>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

interface FileExplorerProps {
  onNavigateToGraph?: (projectId?: string) => void;
}

export const FileExplorer: React.FC<FileExplorerProps> = ({ onNavigateToGraph }) => {
  const { codebases, loading, error, refetch } = useCodebases();
  const [selectedCodebase, setSelectedCodebase] = useState<CodebaseProject | null>(null);
  const [knowledgeGraphCodebase, setKnowledgeGraphCodebase] = useState<CodebaseProject | null>(null);
  const [deleteConfirmCodebase, setDeleteConfirmCodebase] = useState<CodebaseProject | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const handleDeleteCodebase = async (codebase: CodebaseProject) => {
    setIsDeleting(true);
    try {
      const response = await fetch('http://localhost:8105/v1/codebase/delete', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          codebase_id: codebase.id,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to delete codebase');
      }

      const result = await response.json();
      console.log('Deleted codebase:', result);
      
      // Refresh the codebases list
      await refetch();
      setDeleteConfirmCodebase(null);
    } catch (err) {
      console.error('Error deleting codebase:', err);
      alert('Failed to delete codebase. Please try again.');
    } finally {
      setIsDeleting(false);
    }
  };

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-slate-400">Loading codebases...</div>
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

  const formatTimeAgo = (isoString: string) => {
    const date = new Date(isoString);
    if (isNaN(date.getTime())) return 'Unknown';

    // Format as "Jan 20, 2026"
    const options: Intl.DateTimeFormatOptions = {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    };
    return date.toLocaleString('en-US', options);
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-border-dark bg-panel-dark flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-bold text-slate-200 uppercase tracking-wider">
            Parsed Codebases
          </h2>
        </div>
        <div className="flex items-center gap-2">
          <div className="relative group">
            <HiSearch className="absolute left-2 top-1.5 text-slate-500 text-lg group-focus-within:text-primary transition-colors" />
            <input
              className="pl-9 pr-4 py-1.5 bg-black/40 border border-border-dark rounded text-sm focus:ring-1 focus:ring-primary focus:border-primary w-64 text-slate-200 placeholder-slate-600 transition-all"
              placeholder="Search codebases..."
              type="text"
            />
          </div>
        </div>
      </div>

      {/* Codebase Cards */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {codebases.map((codebase) => (
            <div
              key={codebase.id}
              className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-5 border-l-4 border-l-primary shadow-lg hover:shadow-xl transition-all cursor-pointer group hover:border-primary/50"
              onClick={() => setSelectedCodebase(codebase)}
            >
              <div className="flex items-start justify-between mb-3">
                <div className="flex-1">
                  <h3 className="text-lg font-bold text-slate-200 group-hover:text-white transition-colors">
                    {codebase.name}
                  </h3>
                  <p className="text-sm text-slate-400 mt-1 line-clamp-2">
                    {codebase.description}
                  </p>
                </div>
                <div className="flex items-center gap-1 ml-2">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onNavigateToGraph?.(codebase.id);
                    }}
                    className="p-2 hover:bg-primary/20 rounded text-slate-400 hover:text-primary transition-colors"
                    title="View in Knowledge Graph"
                  >
                    <SiGraphql size={20} />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setDeleteConfirmCodebase(codebase);
                    }}
                    className="p-2 hover:bg-red-500/20 rounded text-slate-400 hover:text-red-400 transition-colors"
                    title="Delete Codebase"
                  >
                    <GiTrashCan size={20} />
                  </button>
                </div>
              </div>

              <div className="space-y-3">
                {/* Language Stats */}
                <div>
                  <div className="flex items-center gap-2 mb-2">
                    <BiGitBranch className="text-slate-500" size={14} />
                    <span className="text-xs text-slate-500 uppercase tracking-wider">Languages</span>
                  </div>
                  <div className="flex flex-wrap gap-1">
                    {Object.entries(codebase.language_stats).map(([lang, percent]) => (
                      <span
                        key={lang}
                        className="px-2 py-0.5 bg-black/40 border border-slate-700 rounded text-xs text-slate-300"
                      >
                        {lang} {percent}%
                      </span>
                    ))}
                  </div>
                </div>

                {/* Stats */}
                <div className="grid grid-cols-2 gap-4 pt-2 border-t border-slate-800">
                  <div>
                    <div className="text-xl font-bold text-slate-100">{codebase.total_files}</div>
                    <div className="text-xs text-slate-500 uppercase">Files</div>
                  </div>
                  <div>
                    <div className="text-xl font-bold text-slate-100">{codebase.total_symbols}</div>
                    <div className="text-xs text-slate-500 uppercase">Symbols</div>
                  </div>
                </div>

                {/* Last Indexed */}
                <div className="text-xs text-slate-500">
                  Indexed {formatTimeAgo(codebase.last_indexed)}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* File Tree Modal */}
      {selectedCodebase && (
        <FileTreeModal
          codebase={selectedCodebase}
          onClose={() => setSelectedCodebase(null)}
        />
      )}

      {/* Knowledge Graph Modal */}
      {knowledgeGraphCodebase && (
        <KnowledgeGraphModal
          codebase={knowledgeGraphCodebase}
          onClose={() => setKnowledgeGraphCodebase(null)}
        />
      )}

      {/* Delete Confirmation Modal */}
      {deleteConfirmCodebase && (
        <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-panel-dark border border-red-500/50 rounded-lg w-full max-w-md p-6">
            <div className="flex items-start gap-4 mb-4">
              <div className="p-3 bg-red-500/20 rounded-lg">
                <GiTrashCan className="text-red-400" size={24} />
              </div>
              <div className="flex-1">
                <h3 className="text-lg font-bold text-slate-200 mb-2">Delete Codebase?</h3>
                <p className="text-sm text-slate-400 mb-1">
                  Are you sure you want to delete <span className="text-slate-200 font-medium">{deleteConfirmCodebase.name}</span>?
                </p>
                <p className="text-sm text-red-400">
                  This will permanently delete all files, symbols, relationships, and embeddings. This action cannot be undone.
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3 justify-end">
              <button
                onClick={() => setDeleteConfirmCodebase(null)}
                disabled={isDeleting}
                className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Cancel
              </button>
              <button
                onClick={() => handleDeleteCodebase(deleteConfirmCodebase)}
                disabled={isDeleting}
                className="px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
              >
                {isDeleting ? (
                  <>
                    <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                    Deleting...
                  </>
                ) : (
                  <>
                    <GiTrashCan size={16} />
                    Delete Codebase
                  </>
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
