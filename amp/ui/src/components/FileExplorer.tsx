import React, { useState } from 'react';
import { HiFolder, HiFolderOpen, HiChevronRight, HiChevronDown, HiSearch, HiDocumentText, HiCode, HiX } from 'react-icons/hi';
import { IoCreateOutline } from 'react-icons/io5';
import { BiFile, BiGitBranch } from 'react-icons/bi';
import { SiGraphql } from 'react-icons/si';
import { GiTrashCan } from 'react-icons/gi';
import { useCodebases, CodebaseProject, FileNode } from '../hooks/useCodebases';
import { KnowledgeGraphModal } from './KnowledgeGraphModal';
import { FileContentViewer } from './FileContentViewer';

interface FileTreeModalProps {
  codebase: CodebaseProject;
  onClose: () => void;
}

const FileTreeModal: React.FC<FileTreeModalProps> = ({ codebase, onClose }) => {
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set([codebase.path]));
  const [selectedFile, setSelectedFile] = useState<FileNode | null>(null);

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
    const isSelected = selectedFile?.path === node.path;

    return (
      <div key={node.path}>
        <div
          className={`flex items-center space-x-2 p-2 cursor-pointer hover:bg-panel-dark transition-colors ${
            isSelected ? 'bg-primary/10 border-l-2 border-primary' : ''
          }`}
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
          onClick={() => {
            if (node.type === 'folder') {
              toggleFolder(node.path);
            } else {
              setSelectedFile(node);
            }
          }}
        >
          {node.type === 'folder' && (
            <button className="text-slate-400 hover:text-primary">
              {isExpanded ? <HiChevronDown size={16} /> : <HiChevronRight size={16} />}
            </button>
          )}
          
          <div className="text-slate-400">
            {getFileIcon(node)}
          </div>
          
          <span className={`text-sm ${isSelected ? 'text-primary font-medium' : 'text-slate-300'}`}>
            {node.name}
          </span>
          
          {node.symbols && node.symbols.length > 0 && (
            <span className="text-xs text-slate-500 ml-auto">
              {node.symbols.length} symbols
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
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-panel-dark border border-border-dark rounded-lg w-full max-w-6xl h-[80vh] flex flex-col">
        {/* Modal Header */}
        <div className="p-4 border-b border-border-dark bg-black/20 flex items-center justify-between">
          <div>
            <h2 className="text-lg font-bold text-slate-200">{codebase.name}</h2>
            <p className="text-sm text-slate-400">{codebase.description}</p>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-white/5 rounded text-slate-400 hover:text-slate-200 transition-colors"
          >
            <HiX size={20} />
          </button>
        </div>

        {/* Modal Content */}
        <div className="flex-1 flex overflow-hidden">
          {/* File Tree */}
          <div className="w-1/2 border-r border-border-dark flex flex-col">
            <div className="p-3 border-b border-border-dark bg-black/10">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium text-slate-300">Files</span>
                <span className="text-xs text-slate-500">{codebase.total_files} files</span>
              </div>
            </div>
            <div className="flex-1 overflow-y-auto py-2">
              {codebase.file_tree.map(node => renderFileNode(node))}
            </div>
          </div>

          {/* File Preview */}
          <div className="w-1/2 flex flex-col">
            {selectedFile ? (
              <FileContentViewer file={selectedFile} />
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
  );
};

interface FileExplorerProps {
  onNavigateToGraph?: () => void;
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
          <button 
            onClick={refetch}
            className="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-slate-200 transition-colors"
          >
            <IoCreateOutline size={16} />
          </button>
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
                      if (onNavigateToGraph) {
                        onNavigateToGraph();
                      }
                    }}
                    className="p-2 hover:bg-primary/20 rounded text-slate-400 hover:text-primary transition-colors"
                    title="View Knowledge Graph"
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
