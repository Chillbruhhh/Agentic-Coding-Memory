import React, { useState, useMemo, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useCodebases } from '../hooks/useCodebases';
import { ForceGraph3DComponent } from './ForceGraph3DComponent';
import { GraphControls } from './GraphControls';
import { GraphLegend } from './GraphLegend';
import { transformAmpToGraph, GraphNode, GraphLink } from '../utils/graphDataAdapter';

export const KnowledgeGraph: React.FC = () => {
  const { codebases, objects, relationships, loading, error } = useCodebases();
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [visibleTypes, setVisibleTypes] = useState<string[]>(['function', 'class', 'method', 'variable', 'interface', 'file', 'project', 'directory']);
  const [fileLogMarkdown, setFileLogMarkdown] = useState<string | null>(null);
  const [fileLogNotes, setFileLogNotes] = useState<string | null>(null);
  const [fileLogLoading, setFileLogLoading] = useState(false);
  const [fileLogError, setFileLogError] = useState<string | null>(null);
  const [layoutKey, setLayoutKey] = useState(0);

  // Transform codebase data to graph format
  const graphData = useMemo(() => {
    if (!objects || objects.length === 0) {
      return { nodes: [], links: [] };
    }

    // Use the actual AMP objects directly
    return transformAmpToGraph(objects, relationships);
  }, [objects, relationships]);

  // Filter nodes based on search and type filters
  const filteredData = useMemo(() => {
    if (!graphData || !graphData.nodes || !graphData.links) {
      return { nodes: [], links: [] };
    }
    
    let filteredNodes = graphData.nodes.filter(node => 
      visibleTypes.includes(node.kind) &&
      (searchQuery === '' || node.name.toLowerCase().includes(searchQuery.toLowerCase()))
    );

    // Filter links to only include those between visible nodes
    const visibleNodeIds = new Set(filteredNodes.map(n => n.id));
    const filteredLinks = graphData.links.filter(link => 
      visibleNodeIds.has(link.source) && visibleNodeIds.has(link.target)
    );

    return { nodes: filteredNodes, links: filteredLinks };
  }, [graphData, searchQuery, visibleTypes]);

  const handleNodeClick = (node: GraphNode) => {
    setSelectedNode(node);
    console.log('Node clicked:', node);
  };

  const handleNodeHover = (node: GraphNode | null) => {
    setHoveredNode(node);
  };

  const handleLinkHover = (link: GraphLink | null) => {
    // Handle link hover if needed
  };

  const handleSearch = (query: string) => {
    setSearchQuery(query);
  };

  const handleFilterChange = (filters: string[]) => {
    setVisibleTypes(filters);
  };

  const handleResetCamera = () => {
    // This would be handled by the ForceGraph3D component
    console.log('Reset camera');
  };

  useEffect(() => {
    setLayoutKey(prev => prev + 1);
  }, [searchQuery, visibleTypes.join('|')]);

  useEffect(() => {
    if (!selectedNode || !['file', 'directory'].includes(selectedNode.kind)) {
      setFileLogMarkdown(null);
      setFileLogNotes(null);
      setFileLogError(null);
      setFileLogLoading(false);
      return;
    }

    let isMounted = true;
    const fetchFileLog = async () => {
      setFileLogLoading(true);
      setFileLogError(null);
      try {
        const path = selectedNode.path || selectedNode.name;
        const response = await fetch(`http://localhost:8105/v1/codebase/file-log-objects/${encodeURIComponent(path)}`);
        if (!response.ok) {
          throw new Error(`Failed to load file log (${response.status})`);
        }
        const data = await response.json();
        const summary = data?.file_log?.summary_markdown || data?.file_log?.summary || '';
        const notes = data?.file_log?.notes || null;
        if (isMounted) {
          setFileLogMarkdown(stripNotesSection(summary || ''));
          setFileLogNotes(notes);
        }
      } catch (err) {
        if (isMounted) {
          setFileLogError(err instanceof Error ? err.message : 'Failed to load file log');
          setFileLogMarkdown(null);
          setFileLogNotes(null);
        }
      } finally {
        if (isMounted) {
          setFileLogLoading(false);
        }
      }
    };

    fetchFileLog();
    return () => {
      isMounted = false;
    };
  }, [selectedNode]);

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center bg-background-dark">
        <div className="text-primary text-lg font-mono">Loading knowledge graph...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-1 flex items-center justify-center bg-background-dark">
        <div className="text-red-500 text-lg font-mono">{error}</div>
      </div>
    );
  }

  if (!objects || objects.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center bg-background-dark">
        <div className="text-slate-400 text-lg font-mono">No objects found. Run CLI indexing first.</div>
      </div>
    );
  }

  return (
    <div className="flex-1 relative overflow-hidden bg-background-dark">
      {/* Controls + Legend Panel */}
      <div className="absolute top-4 left-4 z-20 w-64 space-y-3">
        <GraphControls
          onSearch={handleSearch}
          onFilterChange={handleFilterChange}
          onResetCamera={handleResetCamera}
        />
        <GraphLegend nodes={filteredData.nodes} />
      </div>

      {/* Stats Panel */}
      <div className="absolute bottom-4 left-4 z-20 bg-panel-dark border border-border-dark rounded p-3">
        <div className="text-xs text-slate-400 space-y-1">
          <div>Nodes: <span className="text-primary font-mono">{filteredData.nodes.length}</span></div>
          <div>Links: <span className="text-primary font-mono">{filteredData.links.length}</span></div>
          {searchQuery && (
            <div>Search: <span className="text-yellow-400">"{searchQuery}"</span></div>
          )}
        </div>
      </div>

      {/* Node Detail Panel */}
      {selectedNode && (
        <div className="absolute top-4 right-4 bottom-4 z-20 bg-panel-dark border border-border-dark rounded p-5 w-[26rem] flex flex-col shadow-[0_0_25px_rgba(0,0,0,0.4)]">
          <div className="flex items-center justify-between border-b border-border-dark pb-3">
            <div>
              <div className="text-xs uppercase tracking-[0.2em] text-primary">Selected Node</div>
              <div className="text-lg font-semibold text-slate-100 mt-1">{selectedNode.name}</div>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-[10px] bg-primary/10 text-primary border border-primary/30 px-2 py-0.5 rounded-full uppercase tracking-wider">
                {selectedNode.kind}
              </span>
              <button
                onClick={() => setSelectedNode(null)}
                className="text-slate-500 hover:text-primary transition-colors p-1"
                title="Close"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>

          <div className="mt-3 text-[11px] text-slate-400">
            <div className="uppercase tracking-wider text-[10px] text-slate-500">Path</div>
            <div className="font-mono text-xs text-slate-300 break-all mt-1">{selectedNode.path}</div>
          </div>

          {selectedNode.language && (
            <div className="mt-3 text-[11px] text-slate-400">
              <div className="uppercase tracking-wider text-[10px] text-slate-500">Language</div>
              <div className="text-xs text-slate-300 mt-1">{selectedNode.language}</div>
            </div>
          )}

          <div className="mt-4 pt-4 border-t border-border-dark flex-1 min-h-0 flex flex-col">
            {['file', 'directory'].includes(selectedNode.kind) ? (
              <>
                <div className="text-xs uppercase tracking-[0.2em] text-primary mb-3">File Log</div>
                {fileLogNotes && (
                  <div className="mb-4 border border-primary/20 bg-black/30 p-3 text-[11px] text-slate-200 font-mono">
                    <div className="text-[10px] uppercase tracking-[0.2em] text-primary/80 mb-2">Notes</div>
                    <div className="text-slate-300 leading-5">{fileLogNotes}</div>
                  </div>
                )}
                {fileLogLoading && (
                  <div className="text-xs text-slate-500">Loading file log...</div>
                )}
                {fileLogError && (
                  <div className="text-xs text-red-400">{fileLogError}</div>
                )}
                {!fileLogLoading && !fileLogError && (
                  <div className="flex-1 min-h-0 overflow-auto pr-2">
                    {fileLogMarkdown ? (
                      <ReactMarkdown
                        remarkPlugins={[remarkGfm]}
                        className="text-[11px] leading-5 text-slate-200 font-mono space-y-3"
                        components={{
                          h1: ({ children }) => <h1 className="text-sm font-semibold text-primary mb-2">{children}</h1>,
                          h2: ({ children }) => <h2 className="text-xs font-semibold text-primary/80 mt-4 mb-2">{children}</h2>,
                          ul: ({ children }) => <ul className="list-disc list-inside space-y-1">{children}</ul>,
                          ol: ({ children }) => <ol className="list-decimal list-inside space-y-1">{children}</ol>,
                          li: ({ children }) => <li className="text-slate-300">{children}</li>,
                          code: ({ children }) => <code className="bg-black/60 border border-primary/20 px-1 py-0.5 rounded text-primary">{children}</code>,
                          pre: ({ children }) => <pre className="bg-black/60 border border-primary/20 p-3 rounded overflow-auto">{children}</pre>
                        }}
                      >
                        {fileLogMarkdown}
                      </ReactMarkdown>
                    ) : (
                      <div className="text-xs text-slate-500">No file log available.</div>
                    )}
                  </div>
                )}
              </>
            ) : (
              <div className="text-xs text-slate-400">
                Select a file or directory node to view its file log.
              </div>
            )}
          </div>
        </div>
      )}

      {/* 3D Force Graph */}
      <ForceGraph3DComponent
        data={filteredData}
        onNodeClick={handleNodeClick}
        onNodeHover={handleNodeHover}
        onLinkHover={handleLinkHover}
        layoutKey={layoutKey}
      />
    </div>
  );
};

const stripNotesSection = (markdown: string) => {
  const sections = markdown.split(/\n## /);
  if (sections.length === 1) return markdown;
  const [first, ...rest] = sections;
  const filtered = rest.filter(section => !section.toLowerCase().startsWith('notes / decisions linked'));
  return [first, ...filtered.map(section => `## ${section}`)].join('\n');
};
