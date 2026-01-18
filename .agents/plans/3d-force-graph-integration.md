# Feature Plan: 3D Force Graph Integration for AMP Knowledge Visualization

## Overview
Replace the current basic 3D knowledge graph with **vasturiano's 3d-force-graph** library to create a more powerful, interactive, and visually appealing knowledge graph for the AMP Console.

## Current State Analysis
- **Existing**: Basic Three.js/React Three Fiber implementation
- **Issues**: Limited interactivity, basic visualization, no force simulation
- **Data**: 901 code symbols (functions, classes, methods, variables) with relationships

## Proposed Solution: 3d-force-graph Integration

### Why 3d-force-graph?
✅ **Mature Library**: 2.8k+ stars, actively maintained by vasturiano  
✅ **React Integration**: `react-force-graph-3d` package available  
✅ **Rich Features**: Force simulation, node clustering, interactive controls  
✅ **Performance**: WebGL-based rendering, handles large datasets  
✅ **Customization**: Extensive API for styling nodes/links  

### Key Features to Implement

#### 1. **Force-Directed Layout**
```javascript
// Automatic node positioning based on relationships
const Graph = ForceGraph3D()
  .graphData({
    nodes: ampSymbols,
    links: ampRelationships
  })
  .nodeAutoColorBy('kind') // Color by symbol type
  .linkDirectionalParticles(2); // Animated particles on links
```

#### 2. **Symbol Type Visualization**
- **Functions**: Blue spheres with larger size
- **Classes**: Red cubes with medium size  
- **Methods**: Green cylinders (smaller)
- **Variables**: Yellow dots (smallest)
- **Files/Directories**: Gray containers

#### 3. **Interactive Features**
- **Node Hover**: Show symbol details (name, type, file path)
- **Node Click**: Navigate to file or show symbol definition
- **Link Hover**: Show relationship type (calls, defines, imports)
- **Camera Controls**: Zoom, pan, rotate with smooth animations

#### 4. **Data Integration**
```javascript
// Transform AMP data to 3d-force-graph format
const transformAmpData = (ampObjects) => ({
  nodes: ampObjects
    .filter(obj => obj.type === 'Symbol' && ['function', 'class', 'method', 'variable'].includes(obj.kind))
    .map(obj => ({
      id: obj.id,
      name: obj.name,
      kind: obj.kind,
      path: obj.path,
      language: obj.language,
      val: getNodeSize(obj.kind), // Size based on symbol type
      color: getNodeColor(obj.kind) // Color based on symbol type
    })),
  links: ampRelationships.map(rel => ({
    source: rel.from_id,
    target: rel.to_id,
    type: rel.relation_type,
    value: 1
  }))
});
```

## Implementation Plan

### Phase 1: Library Integration (2 hours)
1. **Install Dependencies**
   ```bash
   npm install react-force-graph-3d 3d-force-graph
   ```

2. **Create New Component**
   ```typescript
   // components/ForceGraph3D.tsx
   import ForceGraph3D from 'react-force-graph-3d';
   ```

3. **Replace Current KnowledgeGraph**
   - Keep existing data fetching logic
   - Replace Three.js rendering with ForceGraph3D

### Phase 2: Data Transformation (1 hour)
1. **Create Data Adapter**
   ```typescript
   // utils/graphDataAdapter.ts
   export const transformAmpToGraph = (objects, relationships) => {
     // Transform AMP objects to 3d-force-graph format
   };
   ```

2. **Symbol Type Mapping**
   - Define colors, sizes, shapes for each symbol type
   - Create legend component

### Phase 3: Styling & Theming (1 hour)
1. **Cyberpunk Theme Integration**
   ```javascript
   .backgroundColor('#09090b') // Match app background
   .nodeColor(node => getThemeColor(node.kind))
   .linkColor('#ef4444') // Primary red
   .linkOpacity(0.6)
   ```

2. **Custom Node Rendering**
   ```javascript
   .nodeThreeObject(node => {
     // Custom 3D objects for different symbol types
     return createSymbolMesh(node.kind);
   })
   ```

### Phase 4: Interactivity (1.5 hours)
1. **Event Handlers**
   ```javascript
   .onNodeHover(node => setHoveredNode(node))
   .onNodeClick(node => navigateToSymbol(node))
   .onLinkHover(link => setHoveredLink(link))
   ```

2. **UI Controls**
   - Search/filter nodes
   - Toggle node types
   - Adjust force simulation parameters

### Phase 5: Performance Optimization (0.5 hours)
1. **Large Dataset Handling**
   ```javascript
   .numDimensions(3)
   .enableNodeDrag(false) // Disable for performance
   .enablePointerInteraction(true)
   ```

2. **Lazy Loading**
   - Load graph data progressively
   - Implement viewport culling for large datasets

## Technical Specifications

### Component Structure
```
components/
├── ForceGraph3D.tsx          # Main 3D graph component
├── GraphControls.tsx         # UI controls (search, filters)
├── GraphLegend.tsx           # Symbol type legend
└── GraphTooltip.tsx          # Node/link hover tooltips

utils/
├── graphDataAdapter.ts       # AMP to 3d-force-graph transformer
├── graphTheme.ts            # Cyberpunk styling
└── graphUtils.ts            # Helper functions
```

### Data Flow
```
AMP Objects → Data Adapter → 3D Force Graph → User Interaction → Navigation
     ↓              ↓              ↓              ↓              ↓
  901 symbols → Graph format → WebGL render → Hover/Click → File explorer
```

## Benefits Over Current Implementation

### 1. **Better Performance**
- WebGL rendering vs Canvas/SVG
- Optimized for large datasets (1000+ nodes)
- Smooth 60fps animations

### 2. **Enhanced Interactivity**
- Force simulation creates natural clustering
- Smooth camera controls and transitions
- Rich hover/click interactions

### 3. **Professional Appearance**
- Industry-standard force-directed layout
- Customizable node/link styling
- Matches cyberpunk theme perfectly

### 4. **Extensibility**
- Plugin system for custom behaviors
- Easy to add new visualization modes
- Support for different graph algorithms

## Risk Assessment

### Low Risk
- **Library Maturity**: Well-established, actively maintained
- **React Integration**: Official React wrapper available
- **Documentation**: Comprehensive examples and API docs

### Medium Risk
- **Bundle Size**: ~200KB additional (acceptable for desktop app)
- **Learning Curve**: New API to learn (well-documented)

### Mitigation
- **Fallback**: Keep current implementation as backup
- **Progressive Enhancement**: Implement features incrementally
- **Testing**: Thorough testing with real AMP data

## Success Metrics

### Functional
- ✅ Displays all 901 symbols correctly
- ✅ Shows relationships between symbols
- ✅ Smooth interactions (hover, click, zoom)
- ✅ Matches cyberpunk theme

### Performance
- ✅ <2 second initial load time
- ✅ 60fps during interactions
- ✅ Handles 1000+ nodes smoothly

### User Experience
- ✅ Intuitive navigation
- ✅ Clear visual hierarchy
- ✅ Responsive to user input

## Timeline: 6 hours total
- **Phase 1**: Library Integration (2h)
- **Phase 2**: Data Transformation (1h)  
- **Phase 3**: Styling & Theming (1h)
- **Phase 4**: Interactivity (1.5h)
- **Phase 5**: Performance (0.5h)

## Next Steps
1. **Approval**: Confirm this approach aligns with project goals
2. **Spike**: Create minimal proof-of-concept (30 minutes)
3. **Implementation**: Follow phased approach above
4. **Testing**: Validate with real AMP data
5. **Refinement**: Polish based on user feedback

This integration will transform the AMP Console's knowledge graph from a basic visualization into a professional, interactive 3D experience that truly showcases the power of the Agentic Memory Protocol.
