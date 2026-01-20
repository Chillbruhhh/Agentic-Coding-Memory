import React, { useRef, useEffect, useCallback, useState, useMemo } from 'react';
import ForceGraph3D from 'react-force-graph-3d';
import { GraphData, GraphNode, GraphLink, getNodeColor } from '../utils/graphDataAdapter';
import { graphTheme } from '../utils/graphTheme';

interface ForceGraph3DComponentProps {
  data: GraphData;
  highlightedNodeIds?: Set<string>;
  onNodeClick?: (node: GraphNode) => void;
  onNodeHover?: (node: GraphNode | null) => void;
  onLinkHover?: (link: GraphLink | null) => void;
  layoutKey?: number;
}

// Dim color helper
const dimColor = (color: string, opacity: number = 0.15): string => {
  // Convert hex to rgba with low opacity
  if (color.startsWith('#')) {
    const r = parseInt(color.slice(1, 3), 16);
    const g = parseInt(color.slice(3, 5), 16);
    const b = parseInt(color.slice(5, 7), 16);
    return `rgba(${r},${g},${b},${opacity})`;
  }
  return color;
};

export const ForceGraph3DComponent: React.FC<ForceGraph3DComponentProps> = ({
  data,
  highlightedNodeIds,
  onNodeClick,
  onNodeHover,
  onLinkHover,
  layoutKey
}) => {
  const fgRef = useRef<any>(null);
  const didInitialZoomRef = useRef(false);
  const [mounted, setMounted] = useState(false);
  const [graphData, setGraphData] = useState<GraphData>({ nodes: [], links: [] });

  // Check if we have an active filter (not all nodes highlighted)
  const hasActiveFilter = useMemo(() => {
    if (!highlightedNodeIds || highlightedNodeIds.size === 0) return false;
    if (!data?.nodes || data.nodes.length === 0) return false;
    return highlightedNodeIds.size < data.nodes.length;
  }, [highlightedNodeIds, data?.nodes]);

  // Mark as mounted after first render
  useEffect(() => {
    const timer = setTimeout(() => setMounted(true), 100);
    return () => clearTimeout(timer);
  }, []);

  // Update graph data only after mounted
  useEffect(() => {
    if (!mounted) return;

    const safeData = {
      nodes: data?.nodes || [],
      links: data?.links || []
    };

    // Delay data update to ensure layout is initialized
    const timer = setTimeout(() => {
      setGraphData(safeData);
    }, 50);

    return () => clearTimeout(timer);
  }, [data, mounted]);

  // Zoom to fit on initial load
  useEffect(() => {
    if (fgRef.current && graphData?.nodes?.length && !didInitialZoomRef.current) {
      didInitialZoomRef.current = true;
      setTimeout(() => {
        fgRef.current?.zoomToFit?.(400);
      }, 500);
    }
  }, [graphData]);

  // Reheat simulation when layout key changes
  useEffect(() => {
    if (fgRef.current && graphData?.nodes?.length && layoutKey !== undefined) {
      fgRef.current?.d3ReheatSimulation?.();
    }
  }, [layoutKey]);

  const handleNodeClick = useCallback((node: any) => {
    onNodeClick?.(node);
  }, [onNodeClick]);

  const handleNodeHover = useCallback((node: any) => {
    onNodeHover?.(node);
  }, [onNodeHover]);

  const handleLinkHover = useCallback((link: any) => {
    onLinkHover?.(link);
  }, [onLinkHover]);

  // Node color based on highlight state
  const getNodeColorWithHighlight = useCallback((node: any) => {
    const baseColor = node.color || getNodeColor(node.kind);
    if (!hasActiveFilter) {
      return baseColor;
    }
    const isHighlighted = highlightedNodeIds?.has(node.id);
    return isHighlighted ? baseColor : dimColor(baseColor, 0.15);
  }, [hasActiveFilter, highlightedNodeIds]);

  // Node size based on highlight state
  const getNodeVal = useCallback((node: any) => {
    const baseVal = node.val || 5;
    if (!hasActiveFilter) {
      return baseVal;
    }
    const isHighlighted = highlightedNodeIds?.has(node.id);
    return isHighlighted ? baseVal : baseVal * 0.4;
  }, [hasActiveFilter, highlightedNodeIds]);

  // Link color based on highlight state
  const getLinkColor = useCallback((link: any) => {
    if (!hasActiveFilter) {
      return 'rgba(255,255,255,0.2)';
    }
    const sourceId = typeof link.source === 'object' ? link.source.id : link.source;
    const targetId = typeof link.target === 'object' ? link.target.id : link.target;
    const bothHighlighted = highlightedNodeIds?.has(sourceId) && highlightedNodeIds?.has(targetId);
    return bothHighlighted ? 'rgba(255,255,255,0.5)' : 'rgba(255,255,255,0.02)';
  }, [hasActiveFilter, highlightedNodeIds]);

  if (!mounted) {
    return <div className="w-full h-full flex items-center justify-center text-slate-500">Initializing graph...</div>;
  }

  if (graphData.nodes.length === 0 && data?.nodes?.length === 0) {
    return <div className="w-full h-full flex items-center justify-center text-slate-500">No nodes to display</div>;
  }

  return (
    <ForceGraph3D
      ref={fgRef}
      graphData={graphData}
      backgroundColor={graphTheme.backgroundColor}
      nodeColor={getNodeColorWithHighlight}
      nodeVal={getNodeVal}
      linkColor={getLinkColor}
      onNodeClick={handleNodeClick}
      onNodeHover={handleNodeHover}
      onLinkHover={handleLinkHover}
      cooldownTicks={200}
      warmupTicks={100}
      d3AlphaDecay={0.02}
      d3VelocityDecay={0.3}
    />
  );
};
