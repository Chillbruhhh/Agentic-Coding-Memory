import React, { useRef, useEffect, useState } from 'react';
import ForceGraph3D from 'react-force-graph-3d';
import { GraphData, GraphNode, GraphLink } from '../utils/graphDataAdapter';
import { graphTheme } from '../utils/graphTheme';

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
  const fgRef = useRef<any>();
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);
  const [hoveredLink, setHoveredLink] = useState<GraphLink | null>(null);

  // Debug log the data
  console.log('ForceGraph3D received data:', { nodes: data.nodes.length, links: data.links.length });
  console.log('Sample links in component:', data.links.slice(0, 3));

  // Initialize camera position
  useEffect(() => {
    if (fgRef.current) {
      // Set initial camera distance
      fgRef.current.cameraPosition({ z: graphTheme.cameraDistance });
    }
  }, []);

  const handleNodeHover = (node: GraphNode | null) => {
    setHoveredNode(node);
    onNodeHover?.(node);
  };

  const handleLinkHover = (link: GraphLink | null) => {
    setHoveredLink(link);
    onLinkHover?.(link);
  };

  const handleNodeClick = (node: GraphNode) => {
    // Focus camera on clicked node
    if (fgRef.current) {
      const distance = 40;
      const distRatio = 1 + distance / Math.hypot(node.x || 0, node.y || 0, node.z || 0);
      
      fgRef.current.cameraPosition(
        {
          x: (node.x || 0) * distRatio,
          y: (node.y || 0) * distRatio,
          z: (node.z || 0) * distRatio
        },
        node, // lookAt
        3000  // ms
      );
    }
    
    onNodeClick?.(node);
  };

  return (
    <div className="w-full h-full relative">
      <ForceGraph3D
        ref={fgRef}
        graphData={data}
        
        // Styling
        backgroundColor={graphTheme.backgroundColor}
        
        // Node configuration - better sizing
        nodeAutoColorBy="kind"
        nodeColor={(node: any) => node.color}
        nodeVal={(node: any) => Math.max(node.val * 0.5, 2)} // Smaller nodes
        nodeRelSize={4} // Relative size scaling
        nodeLabel={(node: any) => `
          <div class="bg-panel-dark border border-border-dark rounded p-2 text-slate-200 text-xs">
            <div class="font-bold text-primary">${node.name}</div>
            <div class="text-slate-400">${node.kind}</div>
            <div class="text-slate-500 text-[10px] mt-1">${node.path}</div>
          </div>
        `}
        
        // Link configuration - cleaner styling
        linkColor={() => 'rgba(255,255,255,0.2)'}
        linkOpacity={0.6}
        linkWidth={0.5}
        linkDirectionalParticles={0}
        linkDirectionalParticleSpeed={0.01}
        linkDirectionalParticleColor={() => '#ffffff'}
        
        // Event handlers
        onNodeClick={handleNodeClick}
        onNodeHover={handleNodeHover}
        onLinkHover={handleLinkHover}
        
        // Performance settings
        enableNodeDrag={graphTheme.enableNodeDrag}
        enablePointerInteraction={graphTheme.enablePointerInteraction}
        showNavInfo={graphTheme.showNavInfo}
        
        // Force simulation - better spread
        numDimensions={3}
        d3AlphaDecay={0.0228}
        d3VelocityDecay={0.4}
        d3Force={{
          charge: { strength: -120 },
          link: { distance: 30 }
        }}
        
        // Dimensions
        width={undefined}
        height={undefined}
      />
      
      {/* Hover tooltip */}
      {hoveredNode && (
        <div className="absolute top-4 left-4 bg-panel-dark border border-border-dark rounded p-3 text-slate-200 text-sm pointer-events-none z-10">
          <div className="font-bold text-primary">{hoveredNode.name}</div>
          <div className="text-slate-400 capitalize">{hoveredNode.kind}</div>
          <div className="text-slate-500 text-xs mt-1">{hoveredNode.path}</div>
          <div className="text-slate-500 text-xs">{hoveredNode.language}</div>
        </div>
      )}
      
      {/* Link tooltip */}
      {hoveredLink && (
        <div className="absolute top-4 right-4 bg-panel-dark border border-border-dark rounded p-3 text-slate-200 text-sm pointer-events-none z-10">
          <div className="font-bold text-primary">Relationship</div>
          <div className="text-slate-400 capitalize">{hoveredLink.type}</div>
        </div>
      )}
    </div>
  );
};
