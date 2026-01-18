import React, { useRef, useState, useEffect, useMemo } from 'react';
import { HiX, HiChevronDown, HiChevronRight } from 'react-icons/hi';
import { MdPanTool, MdZoomIn, MdTouchApp } from 'react-icons/md';
import { CodebaseProject } from '../hooks/useCodebases';

interface GraphNode {
  id: string;
  name: string;
  type: string;
  path: string;
  language?: string;
  signature?: string;
  x: number;
  y: number;
  vx: number;
  vy: number;
  children?: GraphNode[];
  parent?: string;
  collapsed: boolean;
  depth: number;
}

interface KnowledgeGraphModalProps {
  codebase: CodebaseProject;
  onClose: () => void;
}

export const KnowledgeGraphModal: React.FC<KnowledgeGraphModalProps> = ({ codebase, onClose }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const bgCanvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [transform, setTransform] = useState({ x: 0, y: 0, scale: 1.5 }); // Start zoomed in
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const animationRef = useRef<number>();
  const bgAnimationRef = useRef<number>();
  const [isInitialized, setIsInitialized] = useState(false);

  // Convert codebase to hierarchical graph data
  const graphData = useMemo(() => {
    const allNodes: GraphNode[] = [];
    let nodeId = 0;

    // Create root node for the repository
    const rootNode: GraphNode = {
      id: 'root',
      name: codebase.name,
      type: 'folder',
      path: codebase.path || '/',
      x: 600,
      y: 400,
      vx: 0,
      vy: 0,
      children: [],
      collapsed: true, // Start collapsed
      depth: 0
    };
    allNodes.push(rootNode);

    const processNode = (fileNode: any, depth: number = 1, parentId?: string): GraphNode | null => {
      const node: GraphNode = {
        id: `node-${nodeId++}`,
        name: fileNode.name,
        type: fileNode.type === 'folder' ? 'folder' : 'file',
        path: fileNode.path,
        language: fileNode.language,
        x: Math.random() * 800 + 200,
        y: Math.random() * 600 + 200,
        vx: 0,
        vy: 0,
        children: [],
        parent: parentId,
        collapsed: true, // Start with everything collapsed
        depth
      };

      allNodes.push(node);

      if (fileNode.type === 'folder' && fileNode.children) {
        fileNode.children.forEach((child: any) => {
          const childNode = processNode(child, depth + 1, node.id);
          if (childNode) node.children!.push(childNode);
        });
      }

      if (fileNode.type === 'file' && fileNode.symbols) {
        fileNode.symbols.forEach((symbol: any) => {
          const symbolNode: GraphNode = {
            id: `node-${nodeId++}`,
            name: symbol.name,
            type: symbol.type,
            path: fileNode.path,
            signature: symbol.signature,
            x: node.x + (Math.random() - 0.5) * 100,
            y: node.y + (Math.random() - 0.5) * 100,
            vx: 0,
            vy: 0,
            parent: node.id,
            collapsed: false,
            depth: depth + 1
          };
          allNodes.push(symbolNode);
          node.children!.push(symbolNode);
        });
      }

      return node;
    };

    codebase.file_tree.forEach(rootFileNode => {
      const childNode = processNode(rootFileNode, 1, 'root');
      if (childNode) rootNode.children!.push(childNode);
    });
    return allNodes;
  }, [codebase]);

  useEffect(() => {
    setNodes(graphData);
    setIsInitialized(false); // Reset initialization when data changes
  }, [graphData]);

  // Center view on root node after nodes are loaded
  useEffect(() => {
    if (nodes.length > 0 && !isInitialized && canvasRef.current) {
      const canvas = canvasRef.current;
      const rect = canvas.getBoundingClientRect();
      const rootNode = nodes.find(n => n.id === 'root');
      
      if (rootNode) {
        // Center the root node in the viewport
        const centerX = rect.width / 2;
        const centerY = rect.height / 2;
        
        setTransform({
          x: centerX - rootNode.x * 1.5,
          y: centerY - rootNode.y * 1.5,
          scale: 1.5
        });
        setIsInitialized(true);
      }
    }
  }, [nodes, isInitialized]);

  const getVisibleNodes = () => {
    const visible: GraphNode[] = [];
    const isVisible = (node: GraphNode): boolean => {
      if (!node.parent) return true;
      const parent = nodes.find(n => n.id === node.parent);
      if (!parent) return true;
      if (parent.collapsed) return false;
      return isVisible(parent);
    };

    nodes.forEach(node => {
      if (isVisible(node)) visible.push(node);
    });

    return visible;
  };

  const getVisibleEdges = () => {
    const visibleNodes = getVisibleNodes();
    const visibleIds = new Set(visibleNodes.map(n => n.id));
    const edges: Array<{ from: GraphNode; to: GraphNode }> = [];

    visibleNodes.forEach(node => {
      if (node.parent) {
        const parent = nodes.find(n => n.id === node.parent);
        if (parent && visibleIds.has(parent.id)) {
          edges.push({ from: parent, to: node });
        }
      }
    });

    return edges;
  };

  // Animated background grid
  useEffect(() => {
    const bgCanvas = bgCanvasRef.current;
    if (!bgCanvas) return;

    const ctx = bgCanvas.getContext('2d');
    if (!ctx) return;

    let offset = 0;

    const renderBg = () => {
      ctx.clearRect(0, 0, bgCanvas.width, bgCanvas.height);

      const gridSize = 50;
      offset = (offset + 0.2) % gridSize;

      ctx.strokeStyle = 'rgba(239, 68, 68, 0.08)';
      ctx.lineWidth = 1;

      // Vertical lines
      for (let x = -gridSize + offset; x < bgCanvas.width + gridSize; x += gridSize) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, bgCanvas.height);
        ctx.stroke();
      }

      // Horizontal lines
      for (let y = -gridSize + offset; y < bgCanvas.height + gridSize; y += gridSize) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(bgCanvas.width, y);
        ctx.stroke();
      }

      bgAnimationRef.current = requestAnimationFrame(renderBg);
    };

    renderBg();
    return () => {
      if (bgAnimationRef.current) cancelAnimationFrame(bgAnimationRef.current);
    };
  }, []);

  // Force simulation
  useEffect(() => {
    const simulate = () => {
      const visibleNodes = getVisibleNodes();
      const edges = getVisibleEdges();

      visibleNodes.forEach(node => {
        const centerX = 600;
        const centerY = 400;
        node.vx += (centerX - node.x) * 0.0005; // Reduced from 0.001
        node.vy += (centerY - node.y) * 0.0005; // Reduced from 0.001

        visibleNodes.forEach(other => {
          if (node.id !== other.id) {
            const dx = other.x - node.x;
            const dy = other.y - node.y;
            const dist = Math.sqrt(dx * dx + dy * dy);
            const minDist = 120; // Increased from 100

            if (dist < minDist && dist > 0) {
              const force = (minDist - dist) / dist * 0.3; // Reduced from 0.5
              node.vx -= dx * force;
              node.vy -= dy * force;
            }
          }
        });
      });

      edges.forEach(({ from, to }) => {
        const dx = to.x - from.x;
        const dy = to.y - from.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const targetDist = 150;

        if (dist > 0) {
          const force = (dist - targetDist) / dist * 0.05; // Reduced from 0.1
          const fx = dx * force;
          const fy = dy * force;

          from.vx += fx;
          from.vy += fy;
          to.vx -= fx;
          to.vy -= fy;
        }
      });

      visibleNodes.forEach(node => {
        node.x += node.vx;
        node.y += node.vy;
        node.vx *= 0.9; // Increased damping from 0.85
        node.vy *= 0.9; // Increased damping from 0.85
      });

      setNodes([...nodes]);
    };

    const interval = setInterval(simulate, 16);
    return () => clearInterval(interval);
  }, [nodes]);

  // Render graph
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d', { 
      alpha: true,
      desynchronized: true,
      willReadFrequently: false
    });
    if (!ctx) return;

    // Enable high-quality rendering
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';

    const getNodeColor = (node: GraphNode, isSelected: boolean, isHovered: boolean) => {
      if (isSelected) return { fill: 'rgba(255, 255, 255, 0.15)', stroke: '#ffffff' };
      if (isHovered) return { fill: 'rgba(255, 107, 107, 0.15)', stroke: '#ff6b6b' };
      
      switch (node.type) {
        case 'folder': return { fill: 'rgba(139, 92, 246, 0.12)', stroke: '#8b5cf6' };
        case 'file': return { fill: 'rgba(16, 185, 129, 0.12)', stroke: '#10b981' };
        case 'function': return { fill: 'rgba(239, 68, 68, 0.12)', stroke: '#ef4444' };
        case 'component': return { fill: 'rgba(251, 191, 36, 0.12)', stroke: '#fbbf24' };
        case 'class': return { fill: 'rgba(249, 115, 22, 0.12)', stroke: '#f97316' };
        default: return { fill: 'rgba(107, 114, 128, 0.12)', stroke: '#6b7280' };
      }
    };

    const render = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      ctx.save();
      ctx.translate(transform.x, transform.y);
      ctx.scale(transform.scale, transform.scale);

      const visibleNodes = getVisibleNodes();
      const edges = getVisibleEdges();

      // Draw edges
      edges.forEach(({ from, to }) => {
        const isActive = selectedNode?.id === from.id || selectedNode?.id === to.id ||
                        hoveredNode?.id === from.id || hoveredNode?.id === to.id;

        ctx.beginPath();
        ctx.moveTo(from.x, from.y);
        ctx.lineTo(to.x, to.y);
        ctx.strokeStyle = isActive ? 'rgba(239, 68, 68, 0.6)' : 'rgba(239, 68, 68, 0.15)';
        ctx.lineWidth = isActive ? 2 : 1;
        ctx.stroke();
      });

      // Draw nodes
      visibleNodes.forEach(node => {
        const isSelected = selectedNode?.id === node.id;
        const isHovered = hoveredNode?.id === node.id;
        const hasChildren = node.children && node.children.length > 0;
        const size = 30; // Increased from 25 to 30

        // Hover ring to show clickable area
        if (isHovered) {
          ctx.beginPath();
          ctx.arc(node.x, node.y, 38, 0, Math.PI * 2);
          ctx.strokeStyle = 'rgba(255, 107, 107, 0.4)';
          ctx.lineWidth = 2;
          ctx.setLineDash([5, 5]);
          ctx.stroke();
          ctx.setLineDash([]);
        }

        // Outer glow for selected/hovered
        if (isSelected || isHovered) {
          ctx.beginPath();
          ctx.arc(node.x, node.y, size + 12, 0, Math.PI * 2);
          ctx.fillStyle = isSelected ? 'rgba(239, 68, 68, 0.15)' : 'rgba(239, 68, 68, 0.08)';
          ctx.fill();
        }

        const colors = getNodeColor(node, isSelected, isHovered);

        // Draw circle
        ctx.beginPath();
        ctx.arc(node.x, node.y, size, 0, Math.PI * 2);
        ctx.fillStyle = colors.fill;
        ctx.fill();
        ctx.strokeStyle = colors.stroke;
        ctx.lineWidth = 2;
        ctx.stroke();

        // Collapse indicator
        if (hasChildren) {
          const indicatorSize = 10;
          const indicatorX = node.x + size - 8;
          const indicatorY = node.y - size + 8;

          ctx.beginPath();
          ctx.arc(indicatorX, indicatorY, indicatorSize, 0, Math.PI * 2);
          ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
          ctx.fill();
          ctx.strokeStyle = colors.stroke;
          ctx.lineWidth = 1.5;
          ctx.stroke();

          ctx.fillStyle = colors.stroke;
          ctx.font = 'bold 12px monospace';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(node.collapsed ? '+' : '−', indicatorX, indicatorY);
        }

        // Label
        ctx.fillStyle = isSelected ? '#ffffff' : isHovered ? '#ff6b6b' : colors.stroke;
        ctx.font = '13px "JetBrains Mono", monospace';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'top';
        const label = node.name.length > 18 ? node.name.substring(0, 18) + '...' : node.name;
        
        const textWidth = ctx.measureText(label).width;
        ctx.fillStyle = 'rgba(0, 0, 0, 0.85)';
        ctx.fillRect(node.x - textWidth / 2 - 6, node.y + size + 8, textWidth + 12, 20);
        
        ctx.fillStyle = isSelected ? '#ffffff' : isHovered ? '#ff6b6b' : colors.stroke;
        ctx.fillText(label, node.x, node.y + size + 12);
      });

      ctx.restore();
      animationRef.current = requestAnimationFrame(render);
    };

    render();
    return () => {
      if (animationRef.current) cancelAnimationFrame(animationRef.current);
    };
  }, [nodes, transform, selectedNode, hoveredNode]);

  // Mouse interactions
  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    const canvas = canvasRef.current;
    if (!rect || !canvas) return;

    // Account for canvas internal size vs display size
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    
    const canvasX = (e.clientX - rect.left) * scaleX;
    const canvasY = (e.clientY - rect.top) * scaleY;
    
    const x = (canvasX - transform.x) / transform.scale;
    const y = (canvasY - transform.y) / transform.scale;

    const visibleNodes = getVisibleNodes();
    const clickedNode = visibleNodes.find(node => {
      const dx = x - node.x;
      const dy = y - node.y;
      return Math.sqrt(dx * dx + dy * dy) < 40;
    });

    if (clickedNode) {
      setSelectedNode(clickedNode);
      
      // Left-click expands if collapsed
      if (clickedNode.children && clickedNode.children.length > 0 && clickedNode.collapsed) {
        setNodes(nodes.map(n => 
          n.id === clickedNode.id ? { ...n, collapsed: false } : n
        ));
      }
    } else {
      setIsDragging(true);
      setDragStart({ x: e.clientX - transform.x, y: e.clientY - transform.y });
    }
  };

  const handleContextMenu = (e: React.MouseEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const rect = canvasRef.current?.getBoundingClientRect();
    const canvas = canvasRef.current;
    if (!rect || !canvas) return;

    // Account for canvas internal size vs display size
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    
    const canvasX = (e.clientX - rect.left) * scaleX;
    const canvasY = (e.clientY - rect.top) * scaleY;
    
    const x = (canvasX - transform.x) / transform.scale;
    const y = (canvasY - transform.y) / transform.scale;

    const visibleNodes = getVisibleNodes();
    const clickedNode = visibleNodes.find(node => {
      const dx = x - node.x;
      const dy = y - node.y;
      return Math.sqrt(dx * dx + dy * dy) < 40;
    });

    // Right-click collapses if expanded
    if (clickedNode && clickedNode.children && clickedNode.children.length > 0 && !clickedNode.collapsed) {
      setNodes(nodes.map(n => 
        n.id === clickedNode.id ? { ...n, collapsed: true } : n
      ));
    }
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;

    if (isDragging) {
      setTransform({
        ...transform,
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y
      });
    } else {
      // Account for canvas internal size vs display size
      const canvas = canvasRef.current;
      if (!canvas) return;
      
      const scaleX = canvas.width / rect.width;
      const scaleY = canvas.height / rect.height;
      
      const canvasX = (e.clientX - rect.left) * scaleX;
      const canvasY = (e.clientY - rect.top) * scaleY;
      
      const x = (canvasX - transform.x) / transform.scale;
      const y = (canvasY - transform.y) / transform.scale;

      const visibleNodes = getVisibleNodes();
      const hovered = visibleNodes.find(node => {
        const dx = x - node.x;
        const dy = y - node.y;
        return Math.sqrt(dx * dx + dy * dy) < 40;
      });

      setHoveredNode(hovered || null);
      
      // Change cursor based on hover state
      if (canvasRef.current) {
        canvasRef.current.style.cursor = hovered ? 'pointer' : 'move';
      }
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  const handleWheel = (e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.3, Math.min(3, transform.scale * delta));
    
    setTransform({
      ...transform,
      scale: newScale
    });
  };

  const visibleNodes = getVisibleNodes();
  const visibleEdges = getVisibleEdges();

  return (
    <div className="fixed inset-0 bg-[#050505] z-50">
      {/* Animated background canvas */}
      <canvas
        ref={bgCanvasRef}
        width={1920}
        height={1080}
        className="absolute inset-0 w-full h-full pointer-events-none opacity-40"
      />

      {/* Scanlines effect */}
      <div className="absolute inset-0 pointer-events-none opacity-10" style={{
        background: 'linear-gradient(to bottom, rgba(255,255,255,0), rgba(255,255,255,0) 50%, rgba(0,0,0,0.1) 50%, rgba(0,0,0,0.1))',
        backgroundSize: '100% 4px'
      }} />

      <div ref={containerRef} className="w-full h-full relative">
        {/* Header */}
        <div className="absolute top-0 left-0 right-0 z-20 p-6 bg-gradient-to-b from-black/90 to-transparent">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-2xl font-bold text-primary uppercase tracking-[0.2em] mb-1">
                {codebase.name}
              </h2>
              <p className="text-sm text-slate-400 font-mono">
                {visibleNodes.length} visible • {nodes.length} total • {visibleEdges.length} connections
              </p>
            </div>
            <button
              onClick={onClose}
              className="p-3 hover:bg-white/10 rounded border border-transparent hover:border-primary/50 text-slate-400 hover:text-white transition-all"
            >
              <HiX size={24} />
            </button>
          </div>
        </div>

        {/* Controls Panel */}
        <div className="absolute top-24 left-6 z-20 bg-black/60 backdrop-blur-md border-l-4 border-primary p-4 shadow-lg w-64">
          <h3 className="text-xs font-bold text-primary tracking-[0.2em] mb-4 border-b border-primary/20 pb-2">
            GRAPH CONTROLS
          </h3>
          <ul className="space-y-3">
            <li className="flex items-center text-xs text-gray-400 group cursor-pointer hover:text-primary transition-colors">
              <MdPanTool className="text-sm mr-3 text-primary/70 group-hover:text-primary" />
              Drag: Pan View
            </li>
            <li className="flex items-center text-xs text-gray-400 group cursor-pointer hover:text-primary transition-colors">
              <MdZoomIn className="text-sm mr-3 text-primary/70 group-hover:text-primary" />
              Scroll: Zoom In/Out
            </li>
            <li className="flex items-center text-xs text-gray-400 group cursor-pointer hover:text-primary transition-colors">
              <MdTouchApp className="text-sm mr-3 text-primary/70 group-hover:text-primary" />
              Click: Select/Expand Node
            </li>
            <li className="flex items-center text-xs text-gray-400 group cursor-pointer hover:text-primary transition-colors">
              <MdTouchApp className="text-sm mr-3 text-primary/70 group-hover:text-primary" />
              Right-Click: Collapse Node
            </li>
          </ul>
        </div>

        {/* Stats Panel */}
        <div className="absolute top-24 right-6 z-20 bg-black/60 backdrop-blur-md border border-primary/30 p-1 flex items-center shadow-[0_0_15px_rgba(239,68,68,0.1)]">
          <div className="flex space-x-0 divide-x divide-primary/30">
            <div className="px-6 py-3 text-center">
              <div className="text-[10px] text-gray-400 uppercase tracking-widest mb-1">Visible</div>
              <div className="text-2xl font-bold text-primary tabular-nums">{visibleNodes.length}</div>
            </div>
            <div className="px-6 py-3 text-center">
              <div className="text-[10px] text-gray-400 uppercase tracking-widest mb-1">Total</div>
              <div className="text-2xl font-bold text-primary tabular-nums">{nodes.length}</div>
            </div>
            <div className="px-6 py-3 text-center">
              <div className="text-[10px] text-gray-400 uppercase tracking-widest mb-1">Zoom</div>
              <div className="text-2xl font-bold text-primary tabular-nums">{Math.round(transform.scale * 100)}%</div>
            </div>
          </div>
        </div>

        {/* Canvas */}
        <canvas
          ref={canvasRef}
          width={1920}
          height={1080}
          className="w-full h-full cursor-move"
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          onWheel={handleWheel}
          onContextMenu={handleContextMenu}
        />

        {/* Node Info Panel */}
        {selectedNode && (
          <div className="absolute bottom-6 right-6 z-20 bg-surface-dark/90 backdrop-blur-md border border-primary w-96 shadow-2xl relative overflow-hidden animate-[slideIn_0.3s_ease-out]">
            <div className="absolute top-0 left-0 w-2 h-2 border-t border-l border-primary" />
            <div className="absolute top-0 right-0 w-2 h-2 border-t border-r border-primary" />
            <div className="absolute bottom-0 left-0 w-2 h-2 border-b border-l border-primary" />
            <div className="absolute bottom-0 right-0 w-2 h-2 border-b border-r border-primary" />
            
            <div className="p-5">
              <div className="flex justify-between items-center mb-4 border-b border-primary/20 pb-2">
                <h2 className="text-lg font-bold text-primary">{selectedNode.name}</h2>
                <span className="text-[10px] bg-primary/10 text-primary border border-primary/30 px-2 py-0.5 rounded-full uppercase tracking-wider">
                  {selectedNode.type}
                </span>
              </div>
              
              <div className="space-y-4">
                <div>
                  <div className="text-[10px] uppercase text-gray-400 tracking-wider mb-1">Path</div>
                  <div className="font-mono text-xs text-gray-300 break-all hover:text-primary transition-colors cursor-text">
                    {selectedNode.path}
                  </div>
                </div>

                {selectedNode.language && (
                  <div>
                    <div className="text-[10px] uppercase text-gray-400 tracking-wider mb-1">Language</div>
                    <div className="text-xs text-gray-300">{selectedNode.language}</div>
                  </div>
                )}
                
                {selectedNode.signature && (
                  <div>
                    <div className="text-[10px] uppercase text-gray-400 tracking-wider mb-2">Signature</div>
                    <div className="relative group/code">
                      <div className="absolute -inset-1 bg-primary/5 blur opacity-0 group-hover/code:opacity-100 transition-opacity" />
                      <code className="relative block bg-black border border-primary/30 p-3 text-xs text-primary font-mono rounded shadow-inner">
                        {selectedNode.signature}
                      </code>
                    </div>
                  </div>
                )}

                <div className="grid grid-cols-2 gap-2 mt-2 pt-2 border-t border-primary/10">
                  <div className="flex justify-between items-center">
                    <span className="text-[10px] text-gray-500">Children</span>
                    <span className="text-xs font-bold text-primary">{selectedNode.children?.length || 0}</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-[10px] text-gray-500">Depth</span>
                    <span className="text-xs font-bold text-green-500">{selectedNode.depth}</span>
                  </div>
                </div>

                {selectedNode.children && selectedNode.children.length > 0 && (
                  <button
                    onClick={() => {
                      setNodes(nodes.map(n => 
                        n.id === selectedNode.id ? { ...n, collapsed: !n.collapsed } : n
                      ));
                    }}
                    className="w-full px-4 py-2 bg-primary/10 hover:bg-primary/20 border border-primary/30 text-primary text-sm font-medium rounded transition-colors flex items-center justify-center gap-2"
                  >
                    {selectedNode.collapsed ? (
                      <>
                        <HiChevronRight /> Expand Children
                      </>
                    ) : (
                      <>
                        <HiChevronDown /> Collapse Children
                      </>
                    )}
                  </button>
                )}
              </div>
            </div>
            
            <div className="h-0.5 w-full bg-primary/20">
              <div className="h-full bg-primary w-1/3 animate-[scan_2s_ease-in-out_infinite]" />
            </div>
          </div>
        )}

        {/* System info */}
        <div className="absolute bottom-6 left-6 pointer-events-none opacity-50 z-30">
          <div className="text-[10px] text-primary/60 font-mono space-y-1">
            <p>SYSTEM: ONLINE</p>
            <p>RENDER: CANVAS-2D</p>
            <p>NODES: {visibleNodes.length}/{nodes.length}</p>
            <p>FPS: 60</p>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes slideIn {
          from {
            opacity: 0;
            transform: translateX(20px);
          }
          to {
            opacity: 1;
            transform: translateX(0);
          }
        }
        @keyframes scan {
          0% {
            transform: translateX(0);
          }
          100% {
            transform: translateX(200%);
          }
        }
      `}</style>
    </div>
  );
};
