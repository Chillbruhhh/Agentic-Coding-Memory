<p align="center">
  <img src="public/assets/AMP-banner.png" alt="AMP Logo" width="800"/>
</p>

# Agentic Memory Protocol (AMP)

A vendor-neutral protocol for durable, unified memory in agentic software development.

**Status**: ✅ Complete Working System - Server, CLI, UI, and MCP Server

## Quick Start

```bash
# Clone and navigate
git clone <repo-url>
cd ACM/amp

# Run the server
cd server
cargo run

# Index a codebase (in another terminal)
cd ../cli
cargo run -- index /path/to/your/project

# Launch the UI (in another terminal)
cd ../ui
npm install
npm run dev

# Use with AI agents (Claude Desktop, Cursor)
cd ../mcp-server
cargo build --release
# See mcp-server/INTEGRATION.md for agent configuration
```

## Components

### AMP Server
Core protocol implementation with hybrid retrieval (text + vector + graph).

### CLI Tool
Terminal interface for indexing codebases and managing memory objects.

### Desktop UI
Professional React/Tauri application with 3D knowledge graph visualization.

### MCP Server (NEW)
Model Context Protocol server exposing AMP tools to AI agents like Claude Desktop and Cursor.

**10 MCP Tools:**
- Context & Retrieval: `amp_context`, `amp_query`, `amp_trace`
- Memory Writes: `amp_write_decision`, `amp_write_changeset`, `amp_run_start`, `amp_run_end`
- File Intelligence: `amp_filelog_get`, `amp_filelog_update`
- Coordination: `amp_lease_acquire`, `amp_lease_release`
- Discovery: `amp_status`, `amp_list`

See [mcp-server/README.md](mcp-server/README.md) for details.

## Configuration

Environment variables:

- `PORT` - Server port (default: 8105)
- `BIND_ADDRESS` - Bind address (default: 127.0.0.1)
  - ⚠️ Set to `0.0.0.0` to allow external connections
- `DATABASE_URL` - Database location (default: memory)
  - Use `memory` for in-memory database
  - Use `file://path/to/db` for persistent file-based storage
- `EMBEDDING_SERVICE_URL` - Optional embedding service endpoint
- `MAX_EMBEDDING_DIMENSION` - Max embedding dimensions (default: 1536, range: 1-10000)

### Windows + Docker Path Mapping

When running `amp-server` in Docker on Windows, the compose setup mounts `C:\Users` into the container at `/workspace` (read-only). The server maps Windows paths to `/workspace/...` automatically for parsing.

Override the defaults if you want a narrower mount:
- `AMP_WINDOWS_MOUNT_ROOT` (default: `C:\Users`)
- `AMP_WORKSPACE_MOUNT` (default: `/workspace`)

## Architecture

- **Server**: Rust + Axum + SurrealDB
- **CLI**: Rust + Ratatui TUI
- **UI**: React + Tauri + Three.js
- **MCP Server**: Rust + rmcp SDK
- **Storage**: Embedded SurrealDB with vector indexing
- **Protocol**: HTTP + JSON with OpenAPI specification

## Development

```bash
# Build everything
cargo build --workspace

# Run tests
cargo test --workspace

# Generate SDKs
./scripts/generate-sdks.sh
```
