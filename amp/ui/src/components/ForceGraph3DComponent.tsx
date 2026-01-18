import React, { useRef, useEffect } from 'react';
import ForceGraph3D from '3d-force-graph';
import * as THREE from 'three';
import { GraphData, GraphNode, GraphLink } from '../utils/graphDataAdapter';
import { graphTheme } from '../utils/graphTheme';

console.log('Three.js version:', THREE.REVISION);

interface ForceGraph3DComponentProps {
  data: GraphData;
  onNodeClick?: (node: GraphNode) => void;
  onNodeHover?: (node: GraphNode | null) => void;
  onLinkHover?: (link: GraphLink | null) => void;
}

export const ForceGraph3DComponent: React.FC<ForceGraph3DComponentProps> = ({
  data,
  onNodeClick,
  onNodeHover,
  onLinkHover
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<any>(null);
  const didInitialZoomRef = useRef(false);

  useEffect(() => {
    if (!containerRef.current) {
      console.log('No container ref');
      return;
    }
    
    return () => {
      if (graphRef.current) {
        graphRef.current._destructor();
        graphRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    if (!containerRef.current) {
      return;
    }

    if (!graphRef.current) {
      const width = containerRef.current.clientWidth;
      const height = containerRef.current.clientHeight;

      graphRef.current = new ForceGraph3D(containerRef.current)
        .width(width)
        .height(height)
        .backgroundColor(graphTheme.backgroundColor)
        .nodeAutoColorBy('kind')
        .nodeVal((node: any) => node.val || 5)
        .linkColor(() => 'rgba(255,255,255,0.2)');

      const controls = graphRef.current.controls();
      if (controls) {
        controls.enableDamping = false;
        controls.rotateSpeed = 1.1;
        controls.zoomSpeed = 1.2;
        controls.panSpeed = 0.9;
        controls.minDistance = 5;
        controls.maxDistance = 5000;
        controls.enablePan = true;
      }

      const chargeForce = graphRef.current.d3Force('charge');
      if (chargeForce?.strength) {
        chargeForce.strength(-120);
      }

      const linkForce = graphRef.current.d3Force('link');
      if (linkForce?.distance) {
        linkForce.distance(40);
      }

      // Let the layout drift instead of constantly re-centering on (0,0,0).
      graphRef.current.d3Force('center', null);
    }

    graphRef.current
      .onNodeClick((node: any) => onNodeClick?.(node))
      .onNodeHover((node: any) => onNodeHover?.(node))
      .onLinkHover((link: any) => onLinkHover?.(link));
  }, [onNodeClick, onNodeHover, onLinkHover]);

  useEffect(() => {
    if (!graphRef.current) {
      return;
    }

    // Ensure data has valid arrays with defaults
    const safeData = {
      nodes: data?.nodes || [],
      links: data?.links || []
    };

    if (safeData.nodes.length === 0) {
      console.log('No nodes to render');
      return;
    }

    graphRef.current.graphData(safeData);

    if (!didInitialZoomRef.current) {
      didInitialZoomRef.current = true;
    }
  }, [data]);

  return <div ref={containerRef} className="w-full h-full" />;
};
