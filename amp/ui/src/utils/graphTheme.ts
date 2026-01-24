// Cyberpunk theme configuration for 3D force graph
export const graphTheme = {
  // Background colors
  backgroundColor: '#09090b', // Match app background
  
  // Node colors (cyberpunk palette)
  nodeColors: {
    function: '#3b82f6',   // Blue
    class: '#ef4444',      // Red (primary)
    method: '#10b981',     // Green
    variable: '#f59e0b',   // Yellow
    interface: '#8b5cf6',  // Purple
    file: '#94a3b8',       // Soft slate
    directory: '#38bdf8',  // Sky blue
    project: '#ef4444',    // Bright red (repo core)
    default: '#64748b'     // Slate
  },
  
  // Link styling
  linkColor: '#ffffff',      // White
  linkOpacity: 0.6,
  linkWidth: 1,
  
  // Particle effects
  particleColor: '#ffffff',  // White particles
  particleSpeed: 0.01,
  
  // Lighting
  ambientLight: 0.4,
  directionalLight: 0.8,
  
  // Camera
  cameraDistance: 300,
  
  // Force simulation
  forceStrength: -100,
  linkDistance: 30,
  
  // Performance
  enableNodeDrag: false,
  enablePointerInteraction: true,
  showNavInfo: false
};

// Get themed node color
export const getThemedNodeColor = (kind: string): string => {
  return graphTheme.nodeColors[kind as keyof typeof graphTheme.nodeColors] || graphTheme.nodeColors.default;
};

// Graph control panel styling
export const controlPanelTheme = {
  background: 'bg-panel-dark',
  border: 'border-border-dark',
  text: 'text-slate-300',
  button: 'bg-black/40 hover:bg-primary/20 border-slate-700 hover:border-primary/50',
  input: 'bg-black/40 border-slate-700 focus:border-primary text-slate-200'
};
