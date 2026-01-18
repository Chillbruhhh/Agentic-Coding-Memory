import React, { useState, useMemo } from 'react';
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
      {/* Controls Panel */}
      <div className="absolute top-4 left-4 z-20 w-64">
        <GraphControls
          onSearch={handleSearch}
          onFilterChange={handleFilterChange}
          onResetCamera={handleResetCamera}
        />
      </div>

      {/* Legend Panel */}
      <div className="absolute top-4 right-4 z-20 w-48">
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

      {/* Node Info Panel */}
      {selectedNode && (
        <div className="absolute bottom-4 right-4 z-20 bg-panel-dark border border-border-dark rounded p-4 w-80">
          <h3 className="text-sm font-bold text-primary mb-2 uppercase tracking-wider">
            Selected Symbol
          </h3>
          <div className="space-y-2 text-xs">
            <div>
              <span className="text-slate-400">Name:</span>
              <span className="text-slate-200 ml-2 font-mono">{selectedNode.name}</span>
            </div>
            <div>
              <span className="text-slate-400">Type:</span>
              <span className="text-slate-200 ml-2 capitalize">{selectedNode.kind}</span>
            </div>
            <div>
              <span className="text-slate-400">Path:</span>
              <span className="text-slate-200 ml-2 font-mono text-[10px] break-all">{selectedNode.path}</span>
            </div>
            <div>
              <span className="text-slate-400">Language:</span>
              <span className="text-slate-200 ml-2">{selectedNode.language}</span>
            </div>
          </div>
        </div>
      )}

      {/* 3D Force Graph */}
      <ForceGraph3DComponent
        data={filteredData}
        onNodeClick={handleNodeClick}
        onNodeHover={handleNodeHover}
        onLinkHover={handleLinkHover}
      />
    </div>
  );
};
