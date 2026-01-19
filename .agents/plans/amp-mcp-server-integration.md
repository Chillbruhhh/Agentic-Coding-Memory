# AMP MCP Server Integration Plan

**Created**: 2026-01-18  
**Status**: Planning  
**Estimated Time**: 8-12 hours  

## Feature Overview

Build an MCP (Model Context Protocol) server using the official Rust SDK (rmcp v0.13.0) to expose AMP's memory protocol capabilities as tools for AI agents. This creates a bridge between closed-source coding agents (Claude Desktop, Cursor, etc.) and AMP's hybrid retrieval system.

## Architecture Design

### Container Structure
```
amp-ecosystem/
├── amp-server/          # Existing AMP protocol server (Port 8105)
├── amp-ui/              # Existing desktop UI (Port 3000)
└── amp-mcp-server/      # NEW: MCP tool exposure layer (stdio)
```

### MCP Server Role
- **Translation Layer**: Converts MCP tool calls to AMP HTTP API requests
- **Agent Interface**: Provides semantic, intent-based tools for AI agents
- **Coordination Hub**: Manages multi-agent resource coordination via leases

### Communication Flow
```
AI Agent (Claude/Cursor)
    ↓ (MCP stdio protocol)
MCP Server (amp-mcp-server)
    ↓ (HTTP/JSON)
AMP Server (amp-server)
    ↓ (SurrealDB queries)
Database (SurrealDB)
```

## Technical Implementation

### Dependencies & Setup
```toml
[package]
name = "amp-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Workspace dependencies
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }

# MCP SDK
rmcp = { version = "0.13.0", features = ["server"] }

# HTTP client for AMP API
reqwest = { version = "0.11", features = ["json"] }

# Utilities
tracing = "0.1"
tracing-subscriber = "0.3"
dotenvy = "0.15"
```

### Core MCP Tools (10 Total)

#### 1. Context & Retrieval Tools (3 tools)

**amp_context**
- **Purpose**: High-signal memory bundle for task
- **Input**: 
  - `goal` (string): Task description
  - `scope` (string): "repo" | "path" | "module"
  - `include_recent` (bool): Include recent changes
  - `include_decisions` (bool): Include architectural decisions
- **Output**: Compact bundle with IDs + summaries + key paths
- **AMP Endpoint**: POST /v1/query (hybrid mode)

**amp_query**
- **Purpose**: Hybrid search (text+vector+graph)
- **Input**:
  - `query` (string): Search query
  - `mode` (string): "hybrid" | "text" | "vector" | "graph"
  - `filters` (object): Optional filters
  - `graph_options` (object): Depth, algorithm
- **Output**: Ranked results with explanations
- **AMP Endpoint**: POST /v1/query

**amp_trace**
- **Purpose**: Provenance/lineage tracking
- **Input**:
  - `object_id` (string): Object to trace
  - `depth` (int): Traversal depth
- **Output**: Relationship graph and history
- **AMP Endpoint**: GET /v1/relationships + graph traversal

#### 2. Memory Write Tools (4 tools)

**amp_write_decision**
- **Purpose**: Create ADR-style architectural decisions
- **Input**:
  - `title` (string): Decision title
  - `context` (string): Background context
  - `decision` (string): What was decided
  - `consequences` (string): Expected outcomes
  - `alternatives` (array): Alternatives considered
- **Output**: Created Decision object ID
- **AMP Endpoint**: POST /v1/objects

**amp_write_changeset**
- **Purpose**: Document completed work units
- **Input**:
  - `description` (string): Change description
  - `files_changed` (array): List of file paths
  - `diff_summary` (string): Summary of changes
  - `linked_decisions` (array): Related decision IDs
- **Output**: Created ChangeSet object ID
- **AMP Endpoint**: POST /v1/objects

**amp_run_start**
- **Purpose**: Begin execution tracking
- **Input**:
  - `goal` (string): Execution goal
  - `repo_id` (string): Repository identifier
  - `agent_name` (string): Agent identifier
- **Output**: Run object ID
- **AMP Endpoint**: POST /v1/objects

**amp_run_end**
- **Purpose**: Complete execution with outputs
- **Input**:
  - `run_id` (string): Run to complete
  - `status` (string): "success" | "failure" | "partial"
  - `outputs` (array): Created object IDs
  - `summary` (string): Execution summary
- **Output**: Updated Run object
- **AMP Endpoint**: PUT /v1/objects/{id}

#### 3. File Intelligence Tools (2 tools)

**amp_filelog_get**
- **Purpose**: Retrieve file log by path
- **Input**:
  - `path` (string): File path
- **Output**: File log with symbols, dependencies, changes
- **AMP Endpoint**: GET /v1/codebase/file-logs/{path}

**amp_filelog_update**
- **Purpose**: Update file after changes
- **Input**:
  - `path` (string): File path
  - `summary` (string): Change summary
  - `linked_run` (string): Run ID
  - `linked_changeset` (string): ChangeSet ID
- **Output**: Updated file log
- **AMP Endpoint**: POST /v1/codebase/update-file-log

#### 4. Coordination Tools (2 tools)

**amp_lease_acquire**
- **Purpose**: Resource coordination
- **Input**:
  - `resource` (string): Resource identifier (e.g., "file:src/auth.ts")
  - `duration` (int): Lease duration in seconds
  - `agent_id` (string): Agent identifier
- **Output**: Lease ID or error if locked
- **AMP Endpoint**: POST /v1/leases/acquire

**amp_lease_release**
- **Purpose**: Release coordination
- **Input**:
  - `lease_id` (string): Lease to release
- **Output**: Success confirmation
- **AMP Endpoint**: POST /v1/leases/release

#### 5. Discovery Tools (2 tools)

**amp_status**
- **Purpose**: Health + analytics combined
- **Input**: None
- **Output**: Server health, object counts, recent activity
- **AMP Endpoints**: GET /health + GET /v1/analytics

**amp_list**
- **Purpose**: Browse objects by type/recent
- **Input**:
  - `type` (string): "symbol" | "decision" | "changeset" | "run"
  - `limit` (int): Max results
  - `sort` (string): "recent" | "name"
- **Output**: List of objects with summaries
- **AMP Endpoint**: POST /v1/query with filters

### Project Structure
```
amp/mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # MCP server entry point with stdio transport
│   ├── amp_client.rs        # HTTP client wrapper for AMP API
│   ├── config.rs            # Configuration management
│   ├── schema.rs            # MCP tool schemas and validation
│   └── tools/               # Individual MCP tool implementations
│       ├── mod.rs           # Tool registry
│       ├── context.rs       # amp_context implementation
│       ├── query.rs         # amp_query, amp_trace
│       ├── memory.rs        # write_decision, write_changeset, run_start/end
│       ├── files.rs         # filelog_get, filelog_update
│       ├── coordination.rs  # lease_acquire, lease_release
│       └── discovery.rs     # amp_status, amp_list
└── README.md                # Usage documentation
```

### Agent Workflow Integration
```
1. amp_run_start(goal, repo_id) 
   → Creates Run object in AMP

2. amp_context(goal, scope) 
   → Uses /v1/query endpoint for high-signal bundle

3. amp_query(...) 
   → Leverages hybrid retrieval for specific searches

4. [Agent work with lease coordination]
   → amp_lease_acquire before modifying files
   → Make changes
   → amp_lease_release after completion

5. amp_filelog_update(...) 
   → Documents changes to files

6. amp_write_decision/changeset 
   → Creates memory objects for decisions and changes

7. amp_run_end(run_id, outputs) 
   → Links all outputs via relationships
```

## Implementation Plan

### Phase 1: Core Infrastructure (2-3 hours)

**1.1 Project Setup**
- [ ] Add `amp/mcp-server/` to workspace in root `Cargo.toml`
- [ ] Create `amp/mcp-server/Cargo.toml` with dependencies
- [ ] Create basic directory structure
- [ ] Add `.env.example` for configuration

**1.2 AMP HTTP Client**
- [ ] Implement `amp_client.rs` with reqwest
- [ ] Add connection pooling and timeout handling
- [ ] Create typed request/response structures
- [ ] Add error handling and retry logic

**1.3 Basic MCP Server**
- [ ] Implement `main.rs` with rmcp stdio transport
- [ ] Add server info and capabilities
- [ ] Implement health check
- [ ] Add tracing and logging setup

### Phase 2: Essential Tools (3-4 hours)

**2.1 Query Tools**
- [ ] Implement `amp_context` tool
  - Schema definition with schemars
  - HTTP call to /v1/query
  - Response formatting
- [ ] Implement `amp_query` tool
  - Support all query modes
  - Filter and graph options
  - Result ranking
- [ ] Implement `amp_list` tool
  - Object type filtering
  - Sorting and pagination

**2.2 Write Tools**
- [ ] Implement `amp_run_start` tool
  - Run object creation
  - Timestamp and metadata
- [ ] Implement `amp_run_end` tool
  - Run completion
  - Output linking
- [ ] Implement `amp_write_decision` tool
  - Decision object creation
  - ADR format validation

### Phase 3: Advanced Features (2-3 hours)

**3.1 File Intelligence**
- [ ] Implement `amp_filelog_get` tool
  - Path-based retrieval
  - Symbol and dependency parsing
- [ ] Implement `amp_filelog_update` tool
  - Change documentation
  - Relationship linking

**3.2 Coordination**
- [ ] Implement `amp_lease_acquire` tool
  - Resource locking
  - Conflict detection
- [ ] Implement `amp_lease_release` tool
  - Lease cleanup
  - Error handling

**3.3 Additional Tools**
- [ ] Implement `amp_trace` tool
  - Provenance tracking
  - Graph traversal
- [ ] Implement `amp_write_changeset` tool
  - ChangeSet creation
  - File linking
- [ ] Implement `amp_status` tool
  - Health + analytics aggregation

### Phase 4: Integration & Testing (1-2 hours)

**4.1 Docker Compose Setup**
- [ ] Create `docker-compose.yml` for full stack
- [ ] Configure service networking
- [ ] Add environment variables
- [ ] Test multi-container orchestration

**4.2 Agent Testing**
- [ ] Create Claude Desktop configuration
- [ ] Test tool discovery
- [ ] Validate complete workflow
- [ ] Document usage examples

**4.3 Documentation**
- [ ] Write comprehensive README
- [ ] Add tool usage examples
- [ ] Create troubleshooting guide
- [ ] Document agent integration steps

## Success Criteria

### Technical Validation
- ✅ MCP server connects via stdio transport
- ✅ All 10 tools properly exposed and functional
- ✅ Successful HTTP communication with AMP server
- ✅ Proper error handling and logging
- ✅ Tool schemas validate correctly

### Agent Integration
- ✅ Claude Desktop can discover and use tools
- ✅ Agent workflow (start→context→work→document→end) functional
- ✅ Multi-agent coordination via leases working
- ✅ File logs updated with agent changes
- ✅ Relationships properly created between objects

### System Integration
- ✅ Container orchestration with Docker Compose
- ✅ Independent scaling of each service
- ✅ Proper service discovery and networking
- ✅ Graceful error handling when services unavailable

## Risk Mitigation

### Technical Risks

**MCP Protocol Compatibility**
- Risk: rmcp SDK version incompatibility
- Mitigation: Use official SDK v0.13.0, test with multiple clients

**HTTP Client Reliability**
- Risk: Network failures, timeouts
- Mitigation: Implement retry logic, connection pooling, proper timeouts

**Schema Validation**
- Risk: Invalid tool inputs from agents
- Mitigation: Comprehensive schemars definitions, input validation

### Integration Risks

**Agent Compatibility**
- Risk: Different MCP client implementations
- Mitigation: Test with Claude Desktop, Cursor, and MCP Inspector

**Performance**
- Risk: Slow HTTP calls blocking agent
- Mitigation: Async HTTP client, connection pooling, caching

**Error Handling**
- Risk: AMP server unavailable
- Mitigation: Graceful degradation, clear error messages, retry logic

## Testing Strategy

### Unit Tests
- Tool schema validation
- AMP client request/response handling
- Error handling and edge cases

### Integration Tests
- Full workflow testing
- Multi-tool coordination
- Lease conflict scenarios

### Agent Tests
- Claude Desktop integration
- Complete development workflow
- Multi-agent coordination

## Future Enhancements

### Advanced Features
- **Streaming Responses**: For large query results
- **Batch Operations**: Multiple tool calls in single request
- **Caching Layer**: Reduce AMP server load
- **Metrics Collection**: Tool usage analytics
- **WebSocket Support**: Real-time updates

### Agent Ecosystem
- **Custom Prompts**: Agent-specific memory patterns
- **Workflow Templates**: Common development patterns
- **Integration Examples**: Documentation for various agents
- **Agent Profiles**: Per-agent configuration and preferences

### Performance Optimizations
- **Request Batching**: Combine multiple API calls
- **Response Caching**: Cache frequently accessed data
- **Connection Pooling**: Reuse HTTP connections
- **Lazy Loading**: Load data on demand

## Configuration

### Environment Variables
```bash
# AMP Server Configuration
AMP_SERVER_URL=http://localhost:8105
AMP_SERVER_TIMEOUT=30

# MCP Server Configuration
MCP_SERVER_NAME=amp-mcp-server
MCP_SERVER_VERSION=0.1.0

# Logging
RUST_LOG=info
```

### Claude Desktop Configuration
```json
{
  "mcpServers": {
    "amp": {
      "command": "/path/to/amp-mcp-server",
      "args": [],
      "env": {
        "AMP_SERVER_URL": "http://localhost:8105"
      }
    }
  }
}
```

## Documentation Requirements

### README.md
- Installation instructions
- Configuration guide
- Tool reference
- Usage examples
- Troubleshooting

### Tool Documentation
- Each tool's purpose
- Input/output schemas
- Example usage
- Error handling

### Integration Guide
- Claude Desktop setup
- Cursor configuration
- Docker Compose usage
- Multi-agent scenarios

This MCP server creates a powerful bridge between AI agents and AMP's memory protocol, enabling sophisticated multi-agent coordination and persistent project memory.
