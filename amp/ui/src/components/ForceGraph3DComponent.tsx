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

const kindRank = (kind?: string) => {
  switch ((kind || '').toLowerCase()) {
    case 'project': return 0;
    case 'directory': return 1;
    case 'file': return 2;
    case 'function':
    case 'class':
    case 'method':
    case 'variable':
    case 'interface':
      return 3;
    default:
      return 4;
  }
};

const applyHierarchyLayout = (data: GraphData): GraphData => {
  if (!data?.nodes?.length) return data;

  const nodes = data.nodes.map(node => ({ ...node }));
  const nodeById = new Map<string, GraphNode>();
  nodes.forEach(node => nodeById.set(node.id, node));

  const parentCandidates = new Map<string, Set<string>>();
  data.links.forEach(link => {
    if ((link.type || '').toLowerCase() !== 'defined_in') return;
    const sourceId = typeof link.source === 'object' ? link.source.id : link.source;
    const targetId = typeof link.target === 'object' ? link.target.id : link.target;
    const source = nodeById.get(sourceId);
    const target = nodeById.get(targetId);
    if (!source || !target) return;
    const sourceRank = kindRank(source.kind);
    const targetRank = kindRank(target.kind);
    if (sourceRank === targetRank) return;
    const parent = sourceRank < targetRank ? source : target;
    const child = sourceRank < targetRank ? target : source;
    if (!parentCandidates.has(child.id)) parentCandidates.set(child.id, new Set());
    parentCandidates.get(child.id)!.add(parent.id);
  });

  const chooseParent = (child: GraphNode, candidates: string[]) => {
    if (!candidates.length) return null;
    const childKind = (child.kind || '').toLowerCase();
    if (childKind === 'file') {
      const dirParent = candidates.find(id => (nodeById.get(id)?.kind || '').toLowerCase() === 'directory');
      if (dirParent) return dirParent;
    }
    if (childKind === 'directory') {
      const dirParent = candidates.find(id => (nodeById.get(id)?.kind || '').toLowerCase() === 'directory');
      if (dirParent) return dirParent;
    }
    if (childKind && !['project', 'directory', 'file'].includes(childKind)) {
      const fileParent = candidates.find(id => (nodeById.get(id)?.kind || '').toLowerCase() === 'file');
      if (fileParent) return fileParent;
    }
    return candidates
      .map(id => ({ id, rank: kindRank(nodeById.get(id)?.kind) }))
      .sort((a, b) => a.rank - b.rank)[0]?.id || null;
  };

  const parentMap = new Map<string, string>();
  parentCandidates.forEach((parents, childId) => {
    const child = nodeById.get(childId);
    if (!child) return;
    const chosen = chooseParent(child, Array.from(parents));
    if (chosen) parentMap.set(childId, chosen);
  });

  const childrenByParent = new Map<string, GraphNode[]>();
  parentMap.forEach((parentId, childId) => {
    const parent = nodeById.get(parentId);
    const child = nodeById.get(childId);
    if (!parent || !child) return;
    if (!childrenByParent.has(parentId)) childrenByParent.set(parentId, []);
    childrenByParent.get(parentId)!.push(child);
  });

  const roots = nodes.filter(node => (node.kind || '').toLowerCase() === 'project');
  const rootSpacing = 250;
  roots.forEach((root, index) => {
    root.x = index * rootSpacing;
    root.y = 0;
    root.z = 0;
  });

  const visited = new Set<string>();
  const layoutChildren = (parent: GraphNode, depth: number) => {
    const children = childrenByParent.get(parent.id) || [];
    if (!children.length) return;
    const parentKind = (parent.kind || '').toLowerCase();
    const maxDepth = 3;
    if (depth > maxDepth) return;
    const radiusBase = 90;
    const radius = Math.max(radiusBase - depth * 8, 45);
    const angleStep = (Math.PI * 2) / children.length;
    const yOffset = depth * 60;

    children.forEach((child, idx) => {
      const angle = idx * angleStep;
      const targetX = (parent.x || 0) + radius * Math.cos(angle);
      const targetY = (parent.y || 0) - yOffset;
      const targetZ = (parent.z || 0) + radius * Math.sin(angle);
      child.x = targetX;
      child.y = targetY;
      child.z = targetZ;
    });

    children.forEach(child => {
      if (visited.has(child.id)) return;
      visited.add(child.id);
      const childKind = (child.kind || '').toLowerCase();
      if (parentKind === 'project' && childKind !== 'directory') return;
      if (parentKind === 'directory' && childKind !== 'file') return;
      layoutChildren(child, depth + 1);
    });
  };

  roots.forEach(root => {
    visited.add(root.id);
    layoutChildren(root, 1);
  });

  return { ...data, nodes };
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
      setGraphData(applyHierarchyLayout(safeData));
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
