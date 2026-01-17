# Agentic Memory Protocol (AMP)

## Overview

A vendor-neutral protocol for durable, unified memory in agentic software development. AMP provides persistent, shared knowledge for AI coding agents, enabling coordination, avoiding duplication, and maintaining audit trails across sessions.

**Status**: ✅ Working MVP with CRUD operations

## Quick Start

```bash
# Clone and navigate
git clone <repo-url>
cd ACM/amp

# Run the server
cd server
cargo run

# Test CRUD operations
cd ../scripts
./test-crud.ps1  # Windows
./test-crud.sh   # Linux/Mac
```

## Project Structure

```
ACM/
├── amp/                    # AMP implementation
│   ├── server/            # Rust server (Axum + SurrealDB)
│   ├── cli/               # Terminal CLI interface
│   ├── spec/              # OpenAPI + JSON schemas + DB schema
│   ├── scripts/           # Test and demo scripts
│   └── examples/          # SDK usage examples
├── scripts/               # Build and installation scripts
├── docs/                  # Documentation and specifications
├── sql/                   # Database queries and utilities
├── .kiro/                 # Kiro CLI configuration
│   ├── steering/          # Project context documents
│   └── prompts/           # Custom development prompts
└── .agents/               # Code reviews and analysis
```

## Features Implemented

✅ **CRUD Operations**
- Create single objects (POST /v1/objects)
- Batch create with detailed status (POST /v1/objects/batch)
- Retrieve by ID (GET /v1/objects/{id})

✅ **Codebase Parser** (NEW)
- Tree-sitter based parsing for Python and TypeScript
- Symbol extraction (functions, classes, interfaces, variables)
- Dependency analysis (imports/exports)
- Structured file logs optimized for embeddings
- Change tracking with links to AMP objects
- Content hash-based change detection

✅ **Memory Object Types**
- Symbol (code structure)
- Decision (architecture choices)
- ChangeSet (code modifications)
- Run (agent executions)

✅ **Production Ready**
- 5-second timeouts on all DB operations
- Proper error handling and logging
- Config validation
- Security (localhost-only by default)

## Architecture

- **Server**: Rust + Axum + Tokio (async runtime)
- **Database**: SurrealDB (embedded with vector indexing)
- **API**: HTTP + JSON with OpenAPI v1 specification
- **SDKs**: Auto-generated Python and TypeScript clients (planned)

## Configuration

Environment variables:

- `PORT` - Server port (default: 8105)
- `BIND_ADDRESS` - Bind address (default: 127.0.0.1)
  - ⚠️ Set to `0.0.0.0` to allow external connections
- `DATABASE_URL` - Database location (default: memory)
  - Use `memory` for in-memory database
  - Use `file://path/to/db` for persistent storage
- `EMBEDDING_SERVICE_URL` - Optional embedding service endpoint
- `MAX_EMBEDDING_DIMENSION` - Max embedding dimensions (default: 1536, range: 1-10000)

## Documentation

- [White Paper](Unified_Agentic_Memory_White_Paper.pdf) - Original concept and design
- [Detailed Spec](Unified_Agentic_Memory_White_Paper-Detail.md) - Technical details
- [Development Log](amp/DEVLOG.md) - Implementation timeline and decisions
- [Task Roadmap](amp/TASKS.md) - Remaining features and priorities
- [Code Reviews](.agents/code-reviews/) - Quality analysis and fixes

## Development

Built with Kiro CLI for the AWS Hackathon. See [DEVLOG.md](amp/DEVLOG.md) for development process and time tracking.

**Total Development Time**: 7 hours  
**Kiro CLI Usage**: Extensive (file operations, code generation, documentation)

## Key Design Decisions

1. **Protocol-First**: Started with OpenAPI spec and JSON schemas
2. **Rust + SurrealDB**: Performance, type safety, built-in vector support
3. **Embedded Database**: Simplified deployment for hackathon
4. **Hybrid Retrieval**: Vector + graph + temporal (planned)
5. **Coordination Primitives**: Lease-based multi-agent coordination (planned)

## Next Steps

See [TASKS.md](amp/TASKS.md) for the complete roadmap. Priority items:

1. Query endpoint with hybrid retrieval
2. Vector embedding integration
3. Graph relationship queries
4. SDK generation (Python, TypeScript)
5. Comprehensive testing

## License

[Add license information]

## Contributing

This is a hackathon project. For questions or contributions, please open an issue.