import React, { useRef, useEffect, useCallback, useState } from 'react';
import ForceGraph3D from 'react-force-graph-3d';
import { GraphData, GraphNode, GraphLink } from '../utils/graphDataAdapter';
import { graphTheme } from '../utils/graphTheme';

interface ForceGraph3DComponentProps {
  data: GraphData;
  onNodeClick?: (node: GraphNode) => void;
  onNodeHover?: (node: GraphNode | null) => void;
  onLinkHover?: (link: GraphLink | null) => void;
  layoutKey?: number;
}

export const ForceGraph3DComponent: React.FC<ForceGraph3DComponentProps> = ({
  data,
  onNodeClick,
  onNodeHover,
  onLinkHover,
  layoutKey
}) => {
  const fgRef = useRef<any>(null);
  const didInitialZoomRef = useRef(false);
  const [mounted, setMounted] = useState(false);
  const [graphData, setGraphData] = useState<GraphData>({ nodes: [], links: [] });

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
      nodeAutoColorBy="kind"
      nodeVal={(node: any) => node.val || 5}
      linkColor={() => 'rgba(255,255,255,0.2)'}
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
