import React, { useState, useEffect, useRef } from 'react';
import { BiNetworkChart } from 'react-icons/bi';
import { HiDatabase, HiTerminal, HiShieldCheck, HiCode, HiCube } from 'react-icons/hi';
import { IoAnalytics } from 'react-icons/io5';
import { MdSettings, MdZoomIn, MdPanTool, MdTouchApp } from 'react-icons/md';

interface Node {
  id: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  x: number;
  y: number;
  type: 'primary' | 'secondary' | 'tertiary';
  connections: string[];
}

export const KnowledgeGraph: React.FC = () => {
  const [selectedNode, setSelectedNode] = useState<string>('setup_routes');
  const [hoveredNode, setHoveredNode] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const [animate, setAnimate] = useState(false);
  
  useEffect(() => {
    setAnimate(true);
  }, []);

  const nodes: Node[] = [
    { id: 'setup_routes', label: 'setup_routes', icon: BiNetworkChart, x: 50, y: 50, type: 'primary', connections: ['handlers', 'database', 'config', 'middleware'] },
    { id: 'handlers', label: 'handlers.rs', icon: HiCode, x: 35, y: 40, type: 'secondary', connections: ['setup_routes', 'models', 'services'] },
    { id: 'database', label: 'database.rs', icon: HiDatabase, x: 65, y: 40, type: 'secondary', connections: ['setup_routes', 'models'] },
    { id: 'config', label: 'config.rs', icon: HiTerminal, x: 60, y: 65, type: 'secondary', connections: ['setup_routes'] },
    { id: 'middleware', label: 'middleware', icon: HiShieldCheck, x: 40, y: 70, type: 'secondary', connections: ['setup_routes'] },
    { id: 'models', label: 'models', icon: HiCube, x: 50, y: 30, type: 'tertiary', connections: ['handlers', 'database'] },
    { id: 'services', label: 'services', icon: IoAnalytics, x: 25, y: 45, type: 'tertiary', connections: ['handlers'] },
    { id: 'utils', label: 'utils', icon: MdSettings, x: 75, y: 45, type: 'tertiary', connections: ['database'] },
  ];

  const getNodeInfo = (nodeId: string) => {
    const nodeData: Record<string, any> = {
      setup_routes: {
        path: '/amp/server/src/main.rs',
        signature: 'fn setup_routes() -> Router',
        complexity: 'Low',
        refCount: 3,
        type: 'Function',
        description: 'Main routing configuration for the AMP server'
      },
      handlers: {
        path: '/amp/server/src/handlers/mod.rs',
        signature: 'pub mod objects; pub mod query;',
        complexity: 'Medium',
        refCount: 8,
        type: 'Module',
        description: 'HTTP request handlers for API endpoints'
      },
      database: {
        path: '/amp/server/src/database.rs',
        signature: 'pub struct Database { client: Surreal<Any> }',
        complexity: 'High',
        refCount: 12,
        type: 'Struct',
        description: 'SurrealDB connection and query management'
      },
      config: {
        path: '/amp/server/src/config.rs',
        signature: 'pub struct Config { port: u16, ... }',
        complexity: 'Low',
        refCount: 5,
        type: 'Struct',
        description: 'Server configuration management'
      },
      middleware: {
        path: '/amp/server/src/middleware.rs',
        signature: 'pub fn cors_layer() -> CorsLayer',
        complexity: 'Low',
        refCount: 2,
        type: 'Function',
        description: 'HTTP middleware and request processing'
      },
      models: {
        path: '/amp/server/src/models/mod.rs',
        signature: 'pub struct AmpObject { ... }',
        complexity: 'Medium',
        refCount: 15,
        type: 'Module',
        description: 'Data models and type definitions'
      },
      services: {
        path: '/amp/server/src/services/mod.rs',
        signature: 'pub trait EmbeddingService { ... }',
        complexity: 'High',
        refCount: 6,
        type: 'Trait',
        description: 'Business logic and service layer'
      },
      utils: {
        path: '/amp/server/src/utils.rs',
        signature: 'pub fn normalize_id(id: &str) -> String',
        complexity: 'Low',
        refCount: 4,
        type: 'Function',
        description: 'Utility functions and helpers'
      },
    };
    return nodeData[nodeId] || {
      path: `/amp/server/src/${nodeId}.rs`,
      signature: `pub mod ${nodeId}`,
      complexity: 'Low',
      refCount: 1,
      type: 'Module',
      description: 'Module component'
    };
  };

  const selectedNodeData = getNodeInfo(selectedNode);

  // Calculate SVG line coordinates
  const getLineCoords = (from: Node, to: Node) => {
    const containerWidth = 800;
    const containerHeight = 600;
    return {
      x1: (from.x / 100) * containerWidth,
      y1: (from.y / 100) * containerHeight,
      x2: (to.x / 100) * containerWidth,
      y2: (to.y / 100) * containerHeight,
    };
  };

  return (
    <div className="flex-1 relative overflow-hidden bg-[#050505]">
      {/* Animated grid background */}
      <div className="absolute inset-0 pointer-events-none opacity-30" style={{
        backgroundImage: 'linear-gradient(rgba(255, 51, 51, 0.05) 1px, transparent 1px), linear-gradient(90deg, rgba(255, 51, 51, 0.05) 1px, transparent 1px)',
        backgroundSize: '50px 50px'
      }} />
      
      {/* Radial gradient glow */}
      <div className="absolute inset-0 pointer-events-none bg-[radial-gradient(circle_at_center,transparent_20%,#000000_100%)]" />
      
      {/* Central glow effect */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] bg-primary/5 rounded-full blur-[100px] pointer-events-none" />

      {/* Scanlines effect */}
      <div className="absolute inset-0 pointer-events-none opacity-20" style={{
        background: 'linear-gradient(to bottom, rgba(255,255,255,0), rgba(255,255,255,0) 50%, rgba(0,0,0,0.1) 50%, rgba(0,0,0,0.1))',
        backgroundSize: '100% 4px'
      }} />

      {/* Controls panel - top left */}
      <div className="absolute top-6 left-6 z-30 bg-surface-light/90 dark:bg-black/60 backdrop-blur-md border-l-4 border-primary p-4 shadow-lg w-64">
        <h3 className="text-xs font-bold text-primary tracking-[0.2em] mb-4 border-b border-primary/20 pb-2">
          GRAPH CONTROLS
        </h3>
        <ul className="space-y-3">
          <li className="flex items-center text-xs text-gray-600 dark:text-gray-400 group cursor-pointer hover:text-primary transition-colors">
            <MdPanTool className="text-sm mr-3 text-primary/70 group-hover:text-primary" />
            Drag: Rotate View
          </li>
          <li className="flex items-center text-xs text-gray-600 dark:text-gray-400 group cursor-pointer hover:text-primary transition-colors">
            <MdZoomIn className="text-sm mr-3 text-primary/70 group-hover:text-primary" />
            Scroll: Zoom In/Out
          </li>
          <li className="flex items-center text-xs text-gray-600 dark:text-gray-400 group cursor-pointer hover:text-primary transition-colors">
            <MdTouchApp className="text-sm mr-3 text-primary/70 group-hover:text-primary" />
            Click: Select Node
          </li>
        </ul>
      </div>

      {/* Stats panel - top right */}
      <div className="absolute top-6 right-6 z-30 bg-surface-light/90 dark:bg-black/60 backdrop-blur-md border border-primary/30 p-1 flex items-center shadow-[0_0_15px_rgba(255,51,51,0.1)]">
        <div className="flex space-x-0 divide-x divide-primary/30">
          <div className="px-6 py-3 text-center">
            <div className="text-[10px] text-gray-500 dark:text-gray-400 uppercase tracking-widest mb-1">Nodes</div>
            <div className="text-2xl font-bold text-primary tabular-nums">17</div>
          </div>
          <div className="px-6 py-3 text-center">
            <div className="text-[10px] text-gray-500 dark:text-gray-400 uppercase tracking-widest mb-1">Edges</div>
            <div className="text-2xl font-bold text-primary tabular-nums">15</div>
          </div>
          <div className="px-4 py-3 flex items-center justify-center">
            <div className="w-12 h-8 flex items-end justify-between space-x-[2px]">
              {[40, 70, 100, 50, 30].map((height, i) => (
                <div 
                  key={i}
                  className={`w-1 bg-primary${height === 100 ? '' : height > 60 ? '/60' : '/40'} animate-pulse`}
                  style={{ 
                    height: `${height}%`,
                    animationDelay: `${i * 0.1}s`
                  }}
                />
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Graph visualization area */}
      <div className="absolute inset-0 flex items-center justify-center overflow-hidden">
        {/* SVG for connections */}
        <svg 
          ref={svgRef}
          className="absolute inset-0 w-full h-full pointer-events-none z-10"
          style={{ filter: 'url(#glow)' }}
        >
          <defs>
            <filter id="glow" height="140%" width="140%" x="-20%" y="-20%">
              <feGaussianBlur result="blur" stdDeviation="2" />
              <feComposite in="SourceGraphic" in2="blur" operator="over" />
            </filter>
            <linearGradient id="edge-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" style={{ stopColor: '#FF3333', stopOpacity: 0.1 }} />
              <stop offset="50%" style={{ stopColor: '#FF3333', stopOpacity: 0.8 }} />
              <stop offset="100%" style={{ stopColor: '#FF3333', stopOpacity: 0.1 }} />
            </linearGradient>
          </defs>
          
          {/* Draw connections */}
          {nodes.map(node => 
            node.connections.map(connId => {
              const targetNode = nodes.find(n => n.id === connId);
              if (!targetNode) return null;
              const coords = getLineCoords(node, targetNode);
              const isActive = selectedNode === node.id || selectedNode === connId || 
                              hoveredNode === node.id || hoveredNode === connId;
              
              return (
                <line
                  key={`${node.id}-${connId}`}
                  x1={coords.x1}
                  y1={coords.y1}
                  x2={coords.x2}
                  y2={coords.y2}
                  stroke={isActive ? '#FF3333' : 'rgba(255, 51, 51, 0.2)'}
                  strokeWidth={isActive ? 2 : 1}
                  className="transition-all duration-300"
                  style={{
                    strokeDasharray: '10',
                    animation: isActive ? 'dash 30s linear infinite' : 'none'
                  }}
                />
              );
            })
          )}
        </svg>

        {/* Nodes */}
        <div className="relative w-[800px] h-[600px] pointer-events-none">
          {nodes.map((node, idx) => {
            const Icon = node.icon;
            const isPrimary = node.type === 'primary';
            const isSecondary = node.type === 'secondary';
            const isSelected = selectedNode === node.id;
            const isHovered = hoveredNode === node.id;
            const isConnected = nodes.find(n => n.id === selectedNode)?.connections.includes(node.id);
            
            return (
              <div
                key={node.id}
                className={`absolute pointer-events-auto cursor-pointer transition-all duration-500 ${
                  animate ? 'opacity-100 scale-100' : 'opacity-0 scale-50'
                }`}
                style={{
                  left: `${node.x}%`,
                  top: `${node.y}%`,
                  transform: 'translate(-50%, -50%)',
                  transitionDelay: `${idx * 50}ms`
                }}
                onClick={() => setSelectedNode(node.id)}
                onMouseEnter={() => setHoveredNode(node.id)}
                onMouseLeave={() => setHoveredNode(null)}
              >
                {/* Node glow effect */}
                {(isSelected || isHovered) && (
                  <div className="absolute inset-0 -m-6 bg-primary/20 rounded-full blur-xl animate-pulse" />
                )}
                
                {/* Node container */}
                <div className={`relative flex flex-col items-center ${
                  isPrimary ? 'animate-[float_6s_ease-in-out_infinite]' : ''
                }`}>
                  {/* Hexagon for primary, circle for secondary, square for tertiary */}
                  <div className={`
                    ${isPrimary ? 'w-16 h-16' : isSecondary ? 'w-12 h-12' : 'w-10 h-10'}
                    bg-surface-light dark:bg-surface-dark
                    ${isPrimary ? 'border-2' : 'border'}
                    ${isSelected ? 'border-primary shadow-[0_0_20px_rgba(255,51,51,0.6)]' : 
                      isConnected ? 'border-primary/60 shadow-[0_0_10px_rgba(255,51,51,0.3)]' : 
                      'border-primary/40'}
                    flex items-center justify-center
                    ${isPrimary ? 'clip-hex' : isSecondary ? 'rounded-sm' : 'rounded-full'}
                    transition-all duration-300
                    ${isHovered ? 'scale-110 bg-primary/10' : 'scale-100'}
                    ${isSelected ? 'scale-110' : ''}
                  `}>
                    {/* Pulse ring for primary */}
                    {isPrimary && isSelected && (
                      <div className="absolute inset-0 border border-primary animate-ping opacity-20 clip-hex" />
                    )}
                    
                    <Icon className={`
                      ${isPrimary ? 'text-2xl' : isSecondary ? 'text-lg' : 'text-sm'}
                      ${isSelected ? 'text-white' : 'text-primary'}
                      transition-colors duration-300
                    `} />
                  </div>
                  
                  {/* Label */}
                  <div className={`
                    mt-2 px-2 py-0.5 
                    bg-black/80 dark:bg-black/90 
                    border border-primary/50 
                    text-primary text-[10px] font-bold uppercase tracking-wider 
                    backdrop-blur-sm
                    transition-all duration-300
                    ${isHovered || isSelected ? 'opacity-100' : isPrimary ? 'opacity-100' : 'opacity-70'}
                  `}>
                    {node.label}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Node details panel - bottom right */}
      <div className="absolute bottom-6 right-6 z-30 bg-surface-light/95 dark:bg-surface-dark/90 backdrop-blur-md border border-primary w-96 shadow-2xl relative overflow-hidden">
        {/* Corner decorations */}
        <div className="absolute top-0 left-0 w-2 h-2 border-t border-l border-primary" />
        <div className="absolute top-0 right-0 w-2 h-2 border-t border-r border-primary" />
        <div className="absolute bottom-0 left-0 w-2 h-2 border-b border-l border-primary" />
        <div className="absolute bottom-0 right-0 w-2 h-2 border-b border-r border-primary" />
        
        <div className="p-5">
          <div className="flex justify-between items-center mb-4 border-b border-primary/20 pb-2">
            <h2 className="text-lg font-bold text-primary">{selectedNode}</h2>
            <span className="text-[10px] bg-primary/10 text-primary border border-primary/30 px-2 py-0.5 rounded-full">
              {selectedNodeData.type}
            </span>
          </div>
          
          <div className="space-y-4">
            <div>
              <div className="text-[10px] uppercase text-gray-500 dark:text-gray-400 tracking-wider mb-1">Path</div>
              <div className="font-mono text-xs text-gray-700 dark:text-gray-300 break-all hover:text-primary transition-colors cursor-text">
                {selectedNodeData.path}
              </div>
            </div>
            
            <div>
              <div className="text-[10px] uppercase text-gray-500 dark:text-gray-400 tracking-wider mb-2">Signature</div>
              <div className="relative group/code">
                <div className="absolute -inset-1 bg-primary/5 blur opacity-0 group-hover/code:opacity-100 transition-opacity" />
                <code className="relative block bg-gray-100 dark:bg-black border border-primary/30 p-3 text-xs text-primary font-mono rounded shadow-inner">
                  <span className="text-orange-600 dark:text-orange-400">fn</span> {selectedNodeData.signature}
                </code>
              </div>
            </div>
            
            <div className="grid grid-cols-2 gap-2 mt-2 pt-2 border-t border-primary/10">
              <div className="flex justify-between items-center">
                <span className="text-[10px] text-gray-500">Complexity</span>
                <span className={`text-xs font-bold ${
                  selectedNodeData.complexity === 'Low' ? 'text-green-500' :
                  selectedNodeData.complexity === 'Medium' ? 'text-yellow-500' :
                  'text-red-500'
                }`}>
                  {selectedNodeData.complexity}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-[10px] text-gray-500">Ref count</span>
                <span className="text-xs font-bold text-primary">{selectedNodeData.refCount}</span>
              </div>
            </div>
          </div>
        </div>
        
        {/* Animated scan line */}
        <div className="h-0.5 w-full bg-primary/20">
          <div className="h-full bg-primary w-1/3 animate-[scan_2s_ease-in-out_infinite]" />
        </div>
      </div>

      {/* System info - bottom left */}
      <div className="absolute bottom-6 left-6 pointer-events-none opacity-50 z-30">
        <div className="text-[10px] text-primary/60 font-mono space-y-1">
          <p>SYSTEM: ONLINE</p>
          <p>RENDER: CANVAS-2D</p>
          <p>LATENCY: 12ms</p>
        </div>
      </div>

      <style>{`
        @keyframes dash {
          to {
            stroke-dashoffset: -1000;
          }
        }
        @keyframes float {
          0%, 100% {
            transform: translate(-50%, -50%) translateY(0);
          }
          50% {
            transform: translate(-50%, -50%) translateY(-10px);
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
        .clip-hex {
          clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
        }
      `}</style>
    </div>
  );
};
