# Feature: Cyberpunk Tauri + React + Three.js Knowledge Graph UI

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Create a stunning cyberpunk-themed desktop application using Tauri that visualizes AMP's knowledge graph in 3D. The application will display the hierarchical codebase structure (project → directories → files → symbols) as an interactive, collapsible 3D graph with neon aesthetics, glitch effects, and smooth animations.

## User Story

As a developer using AMP
I want to visualize my codebase as a beautiful 3D hierarchical graph with cyberpunk aesthetics
So that I can intuitively explore project structure, navigate relationships, and create impressive demos of the AMP system

## Problem Statement

Current AMP visualization is limited to SurrealDB browser showing flat network graphs. Users cannot easily understand the hierarchical nature of their codebase or navigate the relationships between project components. The system needs a professional, visually appealing interface for demonstrations and daily use.

## Solution Statement

Build a Tauri desktop application with React/TypeScript frontend featuring a Three.js-powered 3D knowledge graph. The graph will display hierarchical relationships with cyberpunk styling, allowing users to collapse/expand nodes, navigate through the codebase structure, and interact with individual components.

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: High
**Primary Systems Affected**: New desktop application, AMP server API integration
**Dependencies**: Tauri 2.0, React 18, TypeScript, Three.js, React Three Fiber

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/cli/src/client.rs` (lines 1-200) - Why: Contains AmpClient pattern for API communication
- `amp/cli/src/config.rs` (lines 1-50) - Why: Configuration management pattern
- `amp/server/src/handlers/query.rs` (lines 1-100) - Why: Query endpoint structure and response format
- `amp/server/src/config.rs` (lines 1-50) - Why: Server configuration and port settings
- `amp/spec/openapi.yaml` (lines 200-300) - Why: API endpoint specifications

### New Files to Create

- `amp/ui/` - New Tauri application directory
- `amp/ui/src-tauri/` - Tauri Rust backend
- `amp/ui/src/` - React TypeScript frontend
- `amp/ui/src/components/KnowledgeGraph.tsx` - Main 3D graph component
- `amp/ui/src/hooks/useAmpData.ts` - Data fetching hook
- `amp/ui/src/styles/cyberpunk.css` - Cyberpunk theme styles

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [Tauri 2.0 Getting Started](https://tauri.app/v1/guides/getting-started/setup/)
  - Specific section: Project setup and configuration
  - Why: Required for creating Tauri application structure
- [React Three Fiber Introduction](https://docs.pmnd.rs/react-three-fiber/getting-started/introduction)
  - Specific section: Basic setup and scene creation
  - Why: Essential for 3D graph visualization
- [Three.js Graph Examples](https://threejs.org/examples/#webgl_interactive_cubes)
  - Specific section: Interactive 3D objects
  - Why: Patterns for interactive graph nodes

### Patterns to Follow

**API Client Pattern** (from `amp/cli/src/client.rs`):
```rust
pub struct AmpClient {
    client: reqwest::Client,
    base_url: String,
}

impl AmpClient {
    pub async fn query(&self, request: QueryRequest) -> Result<QueryResponse> {
        let response = self.client
            .post(&format!("{}/v1/query", self.base_url))
            .json(&request)
            .send()
            .await?;
        Ok(response.json().await?)
    }
}
```

**Error Handling Pattern**:
```rust
use anyhow::Result;
// All functions return Result<T, anyhow::Error>
```

**Configuration Pattern** (from `amp/server/src/config.rs`):
```rust
pub struct Config {
    pub port: u16,
    pub bind_address: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = env::var("PORT").unwrap_or_else(|_| "8105".to_string()).parse()?;
        // ...
    }
}
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Set up Tauri project structure and basic React + Three.js integration

**Tasks:**
- Initialize Tauri project with React TypeScript template
- Configure Tauri backend for AMP API communication
- Set up React Three Fiber and basic 3D scene
- Implement cyberpunk CSS theme foundation

### Phase 2: Core Implementation

Build the 3D knowledge graph visualization and data integration

**Tasks:**
- Implement AMP API client in Tauri backend
- Create hierarchical graph layout algorithm
- Build interactive 3D nodes with Three.js
- Add collapse/expand functionality for hierarchy

### Phase 3: Integration

Connect all components and add advanced features

**Tasks:**
- Integrate real AMP data with 3D visualization
- Add node selection and information panels
- Implement smooth animations and transitions
- Add cyberpunk visual effects (neon, glitch, particles)

### Phase 4: Testing & Validation

Ensure functionality and performance

**Tasks:**
- Test with real AMP data (931 nodes, 924 relationships)
- Validate performance with large graphs
- Test desktop application packaging
- Verify cross-platform compatibility

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### CREATE amp/ui/ directory structure

- **IMPLEMENT**: Initialize Tauri project with React TypeScript template
- **PATTERN**: Standard Tauri project structure
- **IMPORTS**: `@tauri-apps/cli`, `@tauri-apps/api`
- **GOTCHA**: Ensure Tauri 2.0 compatibility, not v1
- **VALIDATE**: `cd amp/ui && npm run tauri dev`

### UPDATE amp/ui/src-tauri/Cargo.toml

- **IMPLEMENT**: Add dependencies for HTTP client and JSON handling
- **PATTERN**: Mirror dependency pattern from `amp/server/Cargo.toml`
- **IMPORTS**: `reqwest`, `serde`, `serde_json`, `anyhow`, `tokio`
- **GOTCHA**: Use workspace dependencies if possible
- **VALIDATE**: `cd amp/ui/src-tauri && cargo check`

### CREATE amp/ui/src-tauri/src/amp_client.rs

- **IMPLEMENT**: Rust AMP API client for Tauri backend
- **PATTERN**: Mirror `amp/cli/src/client.rs` structure
- **IMPORTS**: `reqwest`, `serde_json`, `anyhow::Result`
- **GOTCHA**: Use async/await with Tauri's async runtime
- **VALIDATE**: `cd amp/ui/src-tauri && cargo test amp_client`

### CREATE amp/ui/src-tauri/src/commands.rs

- **IMPLEMENT**: Tauri commands for frontend-backend communication
- **PATTERN**: Tauri command pattern with `#[tauri::command]`
- **IMPORTS**: `tauri::command`, AMP client types
- **GOTCHA**: All commands must be async and return Result
- **VALIDATE**: `cd amp/ui/src-tauri && cargo check`

### UPDATE amp/ui/src-tauri/src/main.rs

- **IMPLEMENT**: Register Tauri commands and initialize app
- **PATTERN**: Standard Tauri main.rs with command registration
- **IMPORTS**: Commands module, Tauri builder
- **GOTCHA**: Must invoke all commands in builder
- **VALIDATE**: `cd amp/ui && npm run tauri dev`

### UPDATE amp/ui/package.json

- **IMPLEMENT**: Add React Three Fiber and styling dependencies
- **PATTERN**: Standard React TypeScript dependencies
- **IMPORTS**: `@react-three/fiber`, `@react-three/drei`, `three`, `@types/three`
- **GOTCHA**: Ensure Three.js version compatibility
- **VALIDATE**: `cd amp/ui && npm install && npm run build`

### CREATE amp/ui/src/types/amp.ts

- **IMPLEMENT**: TypeScript types for AMP data structures
- **PATTERN**: Mirror Rust structs from server code
- **IMPORTS**: Standard TypeScript type definitions
- **GOTCHA**: Ensure JSON serialization compatibility
- **VALIDATE**: `cd amp/ui && npm run type-check`

### CREATE amp/ui/src/hooks/useAmpData.ts

- **IMPLEMENT**: React hook for fetching AMP data via Tauri
- **PATTERN**: Standard React hook with async state management
- **IMPORTS**: `react`, `@tauri-apps/api/tauri`
- **GOTCHA**: Handle loading states and error cases
- **VALIDATE**: Test hook in development mode

### CREATE amp/ui/src/components/KnowledgeGraph.tsx

- **IMPLEMENT**: Main 3D graph component with React Three Fiber
- **PATTERN**: React Three Fiber Canvas with 3D objects
- **IMPORTS**: `@react-three/fiber`, `@react-three/drei`, `three`
- **GOTCHA**: Performance optimization for large graphs
- **VALIDATE**: Render test graph with mock data

### CREATE amp/ui/src/components/GraphNode.tsx

- **IMPLEMENT**: Individual 3D node component with interactions
- **PATTERN**: React Three Fiber mesh with event handlers
- **IMPORTS**: `@react-three/fiber`, Three.js geometry
- **GOTCHA**: Node positioning and hierarchy layout
- **VALIDATE**: Test node rendering and click events

### CREATE amp/ui/src/utils/graphLayout.ts

- **IMPLEMENT**: Hierarchical graph layout algorithm
- **PATTERN**: Tree layout with force-directed positioning
- **IMPORTS**: Three.js Vector3, mathematical utilities
- **GOTCHA**: Performance with large hierarchies
- **VALIDATE**: Test layout with sample hierarchy data

### CREATE amp/ui/src/styles/cyberpunk.css

- **IMPLEMENT**: Cyberpunk theme with neon colors and effects
- **PATTERN**: CSS custom properties and animations
- **IMPORTS**: CSS animations, gradients, shadows
- **GOTCHA**: Performance impact of complex animations
- **VALIDATE**: Visual inspection in browser

### CREATE amp/ui/src/components/NodeInfoPanel.tsx

- **IMPLEMENT**: Information panel for selected nodes
- **PATTERN**: React component with conditional rendering
- **IMPORTS**: React state management
- **GOTCHA**: Panel positioning and responsive design
- **VALIDATE**: Test panel with different node types

### UPDATE amp/ui/src/App.tsx

- **IMPLEMENT**: Main application component with graph integration
- **PATTERN**: React component with Three.js Canvas
- **IMPORTS**: All created components and hooks
- **GOTCHA**: Canvas sizing and responsive behavior
- **VALIDATE**: `cd amp/ui && npm run dev`

### CREATE amp/ui/src/components/Controls.tsx

- **IMPLEMENT**: Camera controls and graph manipulation UI
- **PATTERN**: React component with Three.js controls
- **IMPORTS**: `@react-three/drei` controls
- **GOTCHA**: Control conflicts and smooth interactions
- **VALIDATE**: Test camera movement and zoom

### ADD amp/ui/src/effects/

- **IMPLEMENT**: Cyberpunk visual effects (particles, glitch, neon)
- **PATTERN**: Three.js shaders and post-processing
- **IMPORTS**: Three.js effects, custom shaders
- **GOTCHA**: Performance impact on lower-end hardware
- **VALIDATE**: Visual effects testing

### UPDATE amp/ui/src-tauri/tauri.conf.json

- **IMPLEMENT**: Tauri configuration for desktop app
- **PATTERN**: Standard Tauri configuration
- **IMPORTS**: App metadata, window settings
- **GOTCHA**: Icon paths and build settings
- **VALIDATE**: `cd amp/ui && npm run tauri build`

---

## TESTING STRATEGY

### Unit Tests

**React Components**: Test component rendering and interactions using Jest + React Testing Library
**Tauri Commands**: Test Rust backend commands with mock AMP server
**Graph Layout**: Test layout algorithm with various hierarchy structures

### Integration Tests

**End-to-End**: Test complete data flow from AMP server through Tauri to React UI
**Performance**: Test with large graphs (931 nodes) for frame rate and responsiveness
**Desktop App**: Test packaged application on different platforms

### Edge Cases

- Empty or malformed AMP data
- Very large hierarchies (1000+ nodes)
- Network connectivity issues
- Window resizing and responsive behavior

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
cd amp/ui && npm run lint
cd amp/ui && npm run type-check
cd amp/ui/src-tauri && cargo fmt --check
cd amp/ui/src-tauri && cargo clippy
```

### Level 2: Unit Tests

```bash
cd amp/ui && npm test
cd amp/ui/src-tauri && cargo test
```

### Level 3: Integration Tests

```bash
cd amp/ui && npm run test:e2e
cd amp/ui && npm run tauri dev  # Manual testing
```

### Level 4: Manual Validation

```bash
# Start AMP server
cd amp/server && cargo run

# Start Tauri app in development
cd amp/ui && npm run tauri dev

# Test with real AMP data
cd amp/cli && cargo run -- index
# Then verify data appears in Tauri UI
```

### Level 5: Build Validation

```bash
cd amp/ui && npm run tauri build
# Test packaged application
```

---

## ACCEPTANCE CRITERIA

- [ ] Tauri desktop application launches successfully
- [ ] 3D knowledge graph displays AMP hierarchical data
- [ ] Nodes are interactive (click, hover, selection)
- [ ] Hierarchy is collapsible/expandable
- [ ] Cyberpunk theme is visually appealing
- [ ] Performance is smooth with 931 nodes
- [ ] Information panels show node details
- [ ] Camera controls work intuitively
- [ ] Application packages for desktop distribution
- [ ] Integration with AMP server API works correctly

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit + integration)
- [ ] No linting or type checking errors
- [ ] Manual testing confirms feature works
- [ ] Acceptance criteria all met
- [ ] Performance meets requirements
- [ ] Desktop application packages successfully

---

## NOTES

**Design Decisions:**
- Using React Three Fiber for declarative 3D programming
- Tauri backend handles AMP API communication for security
- Hierarchical layout algorithm optimized for code structure visualization
- Cyberpunk theme with neon blues, purples, and greens
- Instanced rendering for performance with large graphs

**Performance Considerations:**
- Level-of-detail (LOD) for distant nodes
- Frustum culling for off-screen objects
- Efficient update cycles for animations
- Memory management for large hierarchies

**Future Enhancements:**
- VR/AR support with WebXR
- Real-time collaboration features
- Advanced graph analysis tools
- Plugin system for custom visualizations
