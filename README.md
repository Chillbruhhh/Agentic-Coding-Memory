<p align="center">
  <img src="amp/public/assets/AMP-banner.png" alt="AMP Banner" width="800"/>
</p>

# Agentic Memory Protocol (AMP)

A vendor-neutral protocol for durable, unified memory in agentic software development. AMP provides persistent, shared knowledge for AI coding agents, enabling coordination, avoiding duplication, and maintaining audit trails across sessions.

## Status

**Production Ready** - Complete working system with server, CLI, desktop UI, and MCP integration

- Server: Rust + Axum + SurrealDB with hybrid retrieval
- CLI: Terminal interface with directory indexing and TUI
- UI: Professional React/Tauri desktop application
- MCP Server: Model Context Protocol integration for AI agents

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Node.js 16+ (for UI)
- SurrealDB 2.4+ (optional, embedded by default)

### Installation

```bash
# Clone repository
git clone <repo-url>
cd ACM

# Install AMP CLI
./scripts/install.ps1  # Windows
./scripts/install.sh   # Linux/macOS

# Start AMP server
cd amp/server
cargo run

# Index a codebase
amp index /path/to/your/project

# Launch desktop UI
cd amp/ui
npm install
npm run dev
```

### Using with AI Agents

AMP integrates with AI agents via Model Context Protocol (MCP):

```bash
# Build MCP server
cd amp/mcp-server
cargo build --release

# Configure Claude Desktop
# Add to ~/.config/Claude/claude_desktop_config.json:
{
  "mcpServers": {
    "amp": {
      "command": "/path/to/amp-mcp-server",
      "env": {
        "AMP_SERVER_URL": "http://localhost:8105"
      }
    }
  }
}
```

See [amp/mcp-server/INTEGRATION.md](amp/mcp-server/INTEGRATION.md) for complete agent integration guide.

## Architecture

### System Overview

```
┌─────────────────────────────────────┐
│      AI Agents (Claude, Cursor)    │
├─────────────────────────────────────┤
│         MCP Server (Rust)           │
├─────────────────────────────────────┤
│    CLI Tool          Desktop UI     │
├─────────────────────────────────────┤
│      AMP Server (Rust + Axum)      │
├─────────────────────────────────────┤
│   SurrealDB (Vector + Graph + Doc)  │
└─────────────────────────────────────┘
```

### Technology Stack

**Backend**
- Rust 1.70+ (systems programming, type safety)
- Axum 0.7 (async web framework)
- Tokio (async runtime)
- SurrealDB 2.4+ (multi-model database)

**Frontend**
- React 18 (UI framework)
- Tauri (desktop application)
- Three.js (3D visualization)
- TailwindCSS (styling)

**Integration**
- MCP (Model Context Protocol)
- OpenAI API (embeddings)
- Ollama (local embeddings)
- Tree-sitter (code parsing - 10 languages)

## Core Features

### Memory Object Types

**Symbol** - Code structure representation
- Functions, classes, interfaces, variables
- Language-agnostic with content hashing
- Automatic dependency tracking

**Decision** - Architectural decision records
- Problem statement and context
- Options considered with pros/cons
- Rationale and outcome tracking

**ChangeSet** - Code modification records
- File changes with diffs
- Test results and validation
- Links to decisions and runs

**Run** - Agent execution tracking
- Input/output logging
- Error tracking and confidence scoring
- Duration and performance metrics

### Hybrid Retrieval System

**Text Search**
- Multi-field keyword matching
- Relevance scoring across name, description, documentation
- Project and tenant filtering

**Vector Search**
- Semantic similarity using embeddings
- OpenAI (text-embedding-3-small, 1536 dimensions)
- Ollama (local models, configurable dimensions)
- Automatic embedding generation on create/update

**Graph Traversal**
- Multi-hop relationship following
- Seven relationship types: depends_on, defined_in, calls, justified_by, modifies, implements, produced
- Three algorithms: Collect (BFS), Path (enumeration), Shortest (Dijkstra)
- Bidirectional traversal with cycle detection

**Hybrid Queries**
- Parallel execution of all search methods
- Weighted scoring: Vector (40%), Text (30%), Graph (30%)
- Result deduplication and explanation generation
- Graceful degradation on partial failures

### Multi-Agent Coordination

**Lease System**
- Resource-based locking (files, modules, services)
- Automatic expiration with configurable TTL
- Acquire, release, and renew operations
- Conflict prevention for concurrent agents

### AI-Powered Code Understanding

**Codebase Indexing**
- Tree-sitter parsing for 10 programming languages:
  - **Python** (`.py`) - functions, classes, imports
  - **TypeScript** (`.ts`, `.tsx`) - functions, classes, interfaces, imports/exports
  - **JavaScript** (`.js`, `.jsx`) - functions, classes, ES6 + CommonJS imports
  - **Rust** (`.rs`) - functions, structs, enums, traits, impl blocks, use statements
  - **Go** (`.go`) - functions, structs, interfaces, type declarations, imports
  - **C#** (`.cs`) - classes, interfaces, methods, properties, enums, namespaces
  - **Java** (`.java`) - classes, interfaces, methods, constructors, fields, enums
  - **C** (`.c`, `.h`) - functions, structs, enums, typedefs, #include
  - **C++** (`.cpp`, `.cc`, `.hpp`) - classes, structs, namespaces, templates
  - **Ruby** (`.rb`, `.rake`) - classes, modules, methods, require/require_relative
- Symbol extraction with signatures and documentation
- Dependency analysis (imports/exports)
- Content hashing for change detection

**AI File Logs**
- LLM-generated file summaries (GPT-4, Claude, Llama)
- Structured markdown with purpose, symbols, dependencies
- Configurable providers: OpenAI, OpenRouter, Ollama
- Parallel processing with worker pools

**File Intelligence**
- File content retrieval from indexed chunks
- Change history tracking
- Links to decisions and changesets
- Notes and architectural context

## MCP Tools for AI Agents

### Context & Retrieval
- **amp_context** - High-signal memory bundle for tasks
- **amp_query** - Hybrid search across all memory
- **amp_trace** - Object provenance and relationships

### Memory Writes
- **amp_write_decision** - Create architectural decision records
- **amp_write_changeset** - Document completed work
- **amp_run_start** - Begin execution tracking
- **amp_run_end** - Complete execution with outputs

### File Intelligence
- **amp_filelog_get** - Retrieve file logs with symbols
- **amp_filelog_update** - Update file after changes
- **amp_file_content_get** - Fetch stored file content

### Coordination
- **amp_lease_acquire** - Acquire resource locks
- **amp_lease_release** - Release resource locks

### Discovery
- **amp_status** - Server health and analytics
- **amp_list** - Browse objects by type

## Configuration

### Environment Variables

```bash
# Server Settings
PORT=8105
BIND_ADDRESS=127.0.0.1

# Database
DATABASE_URL=ws://localhost:7505/rpc  # or file://amp.db or memory
DB_USER=root
DB_PASS=root

# Embedding Provider
EMBEDDING_PROVIDER=openai  # openai, openrouter, ollama, or none
OPENAI_API_KEY=sk-...
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_DIMENSION=1536

# OpenRouter (alternative)
OPENROUTER_API_KEY=sk-...
OPENROUTER_EMBEDDING_MODEL=text-embedding-3-small
OPENROUTER_EMBEDDING_DIMENSION=1536

# Ollama (local)
OLLAMA_URL=http://localhost:11434
EMBEDDING_MODEL=nomic-embed-text
EMBEDDING_DIMENSION=768

# AI Index Model
INDEX_PROVIDER=openai  # openai, openrouter, ollama, or none
INDEX_OPENAI_MODEL=gpt-4o-mini
INDEX_OPENROUTER_MODEL=openai/gpt-4o-mini
INDEX_OLLAMA_MODEL=llama3.1
INDEX_WORKERS=4
```

### Database Options

**In-Memory** (development)
```bash
DATABASE_URL=memory
```

**File-Based** (persistent, single-user)
```bash
DATABASE_URL=file://amp.db
```

**External SurrealDB** (production, multi-user)
```bash
DATABASE_URL=ws://localhost:7505/rpc
DB_USER=root
DB_PASS=root
```

## API Reference

### Core Endpoints

```http
# Object Management
POST   /v1/objects           # Create single object
POST   /v1/objects/batch     # Batch create
GET    /v1/objects/{id}      # Retrieve by ID
PUT    /v1/objects/{id}      # Update object
DELETE /v1/objects/{id}      # Delete object

# Memory Retrieval
POST   /v1/query             # Hybrid search

# Relationships
POST   /v1/relationships     # Create relationship
GET    /v1/relationships     # Query relationships
DELETE /v1/relationships/{type}/{id}  # Delete relationship

# Coordination
POST   /v1/leases:acquire    # Acquire lease
POST   /v1/leases:release    # Release lease
POST   /v1/leases:renew      # Renew lease

# Codebase Intelligence
POST   /v1/codebase/parse    # Parse entire codebase
POST   /v1/codebase/parse-file  # Parse single file
GET    /v1/codebase/file-logs  # Get file logs
GET    /v1/codebase/file-log-objects/{path}  # Get file log by path
POST   /v1/codebase/ai-file-log  # Generate AI file log
GET    /v1/codebase/file-contents/{path}  # Get file content
POST   /v1/codebase/update-file-log  # Update file log

# Analytics & Settings
GET    /v1/analytics         # System analytics
GET    /v1/settings          # Get settings
PUT    /v1/settings          # Update settings

# System
GET    /health               # Health check
```

### Example: Create Symbol

```bash
curl -X POST http://localhost:8105/v1/objects \
  -H "Content-Type: application/json" \
  -d '{
    "type": "symbol",
    "tenant_id": "my-org",
    "project_id": "web-app",
    "provenance": {
      "agent": "amp-cli",
      "summary": "Indexed from codebase"
    },
    "name": "authenticate_user",
    "kind": "function",
    "path": "src/auth.py",
    "language": "python",
    "signature": "def authenticate_user(email: str, password: str) -> bool",
    "documentation": "Validates user credentials against database"
  }'
```

### Example: Hybrid Query

```bash
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "authentication security",
    "hybrid": true,
    "limit": 10,
    "filters": {
      "object_types": ["symbol", "decision"],
      "project_id": "web-app"
    }
  }'
```

## CLI Commands

```bash
# Index a codebase
amp index /path/to/project

# Query memory
amp query "authentication patterns"

# Start agent session
amp start "my-agent"

# Check status
amp status

# View history
amp history

# Launch TUI
amp tui

# Clear database
amp clear
```

## Desktop UI Features

### File Explorer
- Hierarchical project browser
- File preview with syntax highlighting
- Symbol navigation
- Language statistics

### Knowledge Graph
- Interactive 3D force-directed graph
- Node filtering by type
- Expand/collapse hierarchies
- File log panel with markdown rendering
- Real-time layout simulation

### Analytics Dashboard
- Object and relationship counts
- System metrics (CPU, memory)
- Request latency charts
- Object type distribution
- System events log

### Settings
- Server configuration
- Database settings
- Embedding provider selection
- AI index model configuration
- Worker pool settings

## Development

### Project Structure

```
ACM/
├── amp/
│   ├── server/          # Rust server (Axum + SurrealDB)
│   ├── cli/             # Terminal CLI interface
│   ├── ui/              # React/Tauri desktop UI
│   ├── mcp-server/      # MCP integration for AI agents
│   ├── spec/            # OpenAPI + JSON schemas
│   ├── scripts/         # Build and test scripts
│   └── examples/        # Usage examples
├── scripts/             # Installation scripts
├── docs/                # Documentation
├── .kiro/               # Kiro CLI configuration
└── .agents/             # Code reviews and analysis
```

### Building from Source

```bash
# Build server
cd amp/server
cargo build --release

# Build CLI
cd amp/cli
cargo build --release
cargo install --path .

# Build MCP server
cd amp/mcp-server
cargo build --release

# Build UI
cd amp/ui
npm install
npm run build
```

### Running Tests

```bash
# Server tests
cd amp/server
cargo test

# Integration tests
./amp/scripts/test-crud.ps1
./amp/scripts/test-query.ps1
./amp/scripts/test-embeddings.ps1
./amp/scripts/test-relationships.ps1
```

### SurrealDB Delete commands
```sql
DELETE FROM objects;
DELETE FROM relationships;
DELETE FROM defined_in WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);
DELETE FROM depends_on WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);  
DELETE FROM calls WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);
```

## Performance

### Benchmarks

- Object creation: <50ms average
- Hybrid queries: <500ms average (text + vector + graph)
- Vector search: <200ms for 1000 objects
- Graph traversal: <100ms for depth 3
- Batch operations: 100 objects in <1 second

### Scalability

- Tested with 10,000+ objects
- Concurrent agent support via lease system
- Configurable worker pools for indexing
- Connection pooling for database access

### Optimization

- 5-second timeouts on all operations
- Parallel query execution (tokio::join!)
- Result caching for frequent queries
- Efficient vector indexing (MTREE)

## Security

### Authentication

- External SurrealDB: Username/password authentication
- File-based DB: No authentication required
- API: Localhost-only binding by default (127.0.0.1)

### Production Deployment

- Set BIND_ADDRESS=0.0.0.0 for external access
- Use external SurrealDB with authentication
- Configure TLS termination at reverse proxy
- Implement API key authentication (future)

### Data Protection

- Input validation and sanitization
- SQL injection prevention
- Timeout protection against DoS
- Error message sanitization

## Documentation

- [White Paper](Unified_Agentic_Memory_White_Paper.pdf) - Original concept
- [Development Log](amp/DEVLOG.md) - Implementation timeline
- [MCP Integration](amp/mcp-server/INTEGRATION.md) - Agent setup guide
- [UI Design System](amp/ui/DESIGN.md) - Interface guidelines
- [OpenAPI Spec](amp/spec/openapi.yaml) - Complete API reference

## Use Cases

### AI Agent Development
- Persistent memory across sessions
- Shared knowledge between different agents
- Coordination to prevent conflicts
- Audit trail of agent actions

### Development Teams
- Architectural decision tracking
- Code change documentation
- Multi-tool integration (Cursor, Claude, custom agents)
- Knowledge base building over time

### Open Source Maintenance
- Project history preservation
- Contributor onboarding
- Pattern documentation
- Automated code understanding

## Roadmap

### Completed
- Core CRUD operations
- Hybrid retrieval system
- Multi-agent coordination
- AI-powered indexing
- Desktop UI with 3D visualization
- MCP server integration
- CLI tool with TUI
- Multi-language code parsing (10 languages)

### In Progress
- Performance optimization
- Advanced caching strategies
- Real-time subscriptions

### Planned
- Python/TypeScript SDK generation
- Web-based UI
- Advanced multi-tenancy
- Distributed deployment
- Plugin system for custom memory types

## Contributing

This project was built for the AWS Hackathon using Kiro CLI. For questions or contributions, please open an issue.

### Development Stats

- Total Development Time: 30+ hours
- Kiro CLI Usage: Extensive (file operations, code generation, debugging)
- Lines of Code: 15,000+ across Rust, TypeScript, and configuration
- Test Coverage: Comprehensive PowerShell and Bash test suites

## License

[Add license information]

## Acknowledgments

Built with:
- Kiro CLI for development workflow
- SurrealDB for multi-model database
- Axum for async web framework
- React and Tauri for desktop UI
- Tree-sitter for multi-language code parsing (Python, TypeScript, JavaScript, Rust, Go, C#, Java, C, C++, Ruby)
- OpenAI and Ollama for embeddings

## Support

For issues, questions, or feature requests:
- Check [DEVLOG.md](amp/DEVLOG.md) for implementation details
- Review [TASKS.md](amp/TASKS.md) for roadmap
- See [mcp-server/README.md](amp/mcp-server/README.md) for MCP integration
- Open an issue on GitHub