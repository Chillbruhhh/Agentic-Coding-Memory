<p align="center">
  <img src="amp/public/assets/AMP-banner.png" alt="AMP Banner" width="800"/>
</p>

# Agentic Memory Protocol (AMP)

A vendor-neutral protocol for durable, unified memory in agentic software development. AMP provides persistent, shared knowledge for AI agents, enabling seamless coordination, eliminating redundant work, and maintaining complete audit trails across sessions.

Built for the Dynamous-Kiro Hackathon, this implementation demonstrates AMP's capabilities through a coding agent use case—but this is just the beginning. The current system showcases hybrid retrieval (vector + graph + temporal), multi-language code parsing, and real-time knowledge graph visualization, proving AMP's potential as a universal memory substrate.

Post-Hackathon Vision: AMP will evolve into the foundational memory layer for all AI agent implementations. Our roadmap includes embeddable libraries (Python, TypeScript, Rust), distributed coordination primitives, and advanced memory techniques that ensure your agents maintain persistent context throughout their entire workflow—not just within a single session, but across the lifetime of your project.

AMP is positioning to become the standard protocol for agent memory, providing the infrastructure that lets agents remember, reason, and collaborate at scale.

## Quick Start

### Docker (Recommended)

```bash
cd amp
docker compose up
```

| Service | URL |
|---------|-----|
| AMP Server | `http://localhost:8105` |
| MCP Server | `http://localhost:8106` |
| SurrealDB | `http://localhost:7505` |
| UI | `http://localhost:8109` |

### CLI Installation

```bash
# Windows
.\scripts\install.ps1

# Linux/macOS
./scripts/install.sh

# Index a codebase (run from project root)
amp index
```

---

## AI Agent Integration

Configure your AI agent to use AMP via MCP:

<details>
<summary><strong>Claude Desktop</strong></summary>

Edit `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "amp": {
      "type": "streamable-http",
      "url": "http://localhost:8106/mcp"
    }
  }
}
```
</details>

<details>
<summary><strong>Claude Code (CLI)</strong></summary>

```bash
claude mcp add amp -- cmd /c npx mcp-remote@latest http://127.0.0.1:8106/mcp --allow-http AMP_AGENT_NAME="Claude Code"
```
</details>

<details>
<summary><strong>OpenCode</strong></summary>

Edit `.opencode/mcp.json`:

```json
{
  "AMP": {
    "type": "local",
    "command": ["npx", "-y", "mcp-remote@latest", "http://127.0.0.1:8106/mcp", "--allow-http"],
    "enabled": true,
    "environment": {
      "AMP_AGENT_NAME": "OpenCode"
    }
  }
}
```
</details>

<details>
<summary><strong>Codex</strong></summary>

Edit `~/.codex/mcp.toml`:

```toml
[mcp_servers.amp]
command = "npx"
args = ["mcp-remote@latest", "http://127.0.0.1:8106/mcp", "--allow-http"]
AMP_AGENT_NAME = "Codex"
startup_timeout_sec = 30
```
</details>

<details>
<summary><strong>Kiro CLI</strong></summary>

Edit `~/.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "amp": {
      "command": "npx",
      "args": ["mcp-remote", "http://localhost:8106/mcp", "--allow-http"],
      "env": {
        "AMP_AGENT_NAME": "Kiro"
      }
    }
  }
}
```
</details>

<details>
<summary><strong>Cursor / VS Code</strong></summary>

Edit `.cursor/mcp.json` or `.vscode/mcp.json`:

```json
{
  "mcpServers": {
    "amp": {
      "type": "streamable-http",
      "url": "http://localhost:8106/mcp"
    }
  }
}
```
</details>

---

## Architecture

```
┌─────────────────────────────────────┐
│   AI Agents (Claude, Cursor, etc)   │
├─────────────────────────────────────┤
│       MCP Server (Port 8106)        │
├─────────────────────────────────────┤
│      AMP Server (Port 8105)         │
│      Rust + Axum + SurrealDB        │
├─────────────────────────────────────┤
│      CLI Tool + Desktop UI          │
└─────────────────────────────────────┘
```

## Core Features

| Feature | Description |
|---------|-------------|
| **Persistent Memory** | Symbols, decisions, changesets, notes, runs |
| **Hybrid Retrieval** | Vector similarity + graph traversal + text search |
| **Multi-Language Parser** | Python, TypeScript, JavaScript, Rust, Go, C#, Java, C, C++, Ruby |
| **Episodic Cache** | Rolling window of session context (~20 blocks) |
| **File Provenance** | Audit trails, symbols, dependencies per file |
| **Artifact System** | Long-term memory for decisions, conventions, rationale |

## MCP Tools (13 tools)

| Category | Tools |
|----------|-------|
| **Cache** | `amp_cache_read`, `amp_cache_write`, `amp_cache_compact` |
| **File Provenance** | `amp_file_sync`, `amp_filelog_get` |
| **Discovery** | `amp_status`, `amp_list`, `amp_query`, `amp_trace` |
| **Artifacts** | `amp_write_artifact` |
| **Focus** | `amp_focus` |
| **Utility** | `amp_file_content_get`, `amp_file_path_resolve` |

---

## API Reference

### Core Endpoints

```http
# Health
GET    /health

# Objects
POST   /v1/objects              # Create object
POST   /v1/objects/batch        # Batch create
GET    /v1/objects/:id          # Get by ID
PUT    /v1/objects/:id          # Update
DELETE /v1/objects/:id          # Delete

# Query & Trace
POST   /v1/query                # Hybrid search
GET    /v1/trace/:id            # Object provenance

# Relationships
POST   /v1/relationships        # Create relationship
GET    /v1/relationships        # Query relationships
DELETE /v1/relationships/:type/:id  # Delete relationship

# Leases (Multi-Agent Coordination)
POST   /v1/leases/acquire       # Acquire lease
POST   /v1/leases/release       # Release lease
POST   /v1/leases/renew         # Renew lease

# Codebase
POST   /v1/codebase/parse       # Parse entire codebase
POST   /v1/codebase/parse-file  # Parse single file
POST   /v1/codebase/delete      # Delete codebase data
POST   /v1/codebase/sync        # Sync file state (file_sync)
GET    /v1/codebase/file-logs   # List all file logs
GET    /v1/codebase/file-logs/:path  # Get file log by path
GET    /v1/codebase/file-log-objects/:path  # Get file log object
GET    /v1/codebase/file-contents/:path  # Get file content
POST   /v1/codebase/update-file-log  # Update file log
POST   /v1/codebase/ai-file-log  # Generate AI file log

# Artifacts
POST   /v1/artifacts            # Create artifact
GET    /v1/artifacts            # List artifacts
DELETE /v1/artifacts/:id        # Delete artifact

# Cache (Episodic Memory)
POST   /v1/cache/pack           # Get cache pack (legacy)
POST   /v1/cache/write          # Write items (legacy)
POST   /v1/cache/gc             # Garbage collection
POST   /v1/cache/block/write    # Write to cache block
POST   /v1/cache/block/compact  # Compact current block
POST   /v1/cache/block/search   # Search blocks
GET    /v1/cache/block/current/:scope_id  # Get current block
GET    /v1/cache/block/:id      # Get specific block

# Connections (Agent Tracking)
POST   /v1/connections/register   # Register connection
POST   /v1/connections/heartbeat  # Heartbeat
POST   /v1/connections/disconnect # Disconnect
GET    /v1/connections            # List connections
POST   /v1/connections/cleanup    # Cleanup expired

# Analytics & Settings
GET    /v1/analytics            # System analytics
GET    /v1/settings             # Get settings
PUT    /v1/settings             # Update settings
```

---

## Configuration

All settings can be configured via the **UI Settings tab** or environment variables.

### Environment Variables

```bash
# Server
PORT=8105
BIND_ADDRESS=0.0.0.0

# Embeddings
EMBEDDING_PROVIDER=openai  # openai, openrouter, ollama, none
OPENAI_API_KEY=sk-...
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_DIMENSION=1536

# Ollama (local alternative)
OLLAMA_URL=http://localhost:11434
EMBEDDING_MODEL=nomic-embed-text
```

---

## CLI Commands

```bash
amp index              # Index current directory
amp index /path        # Index specific path
```

---

## Development

```bash
# Build all
cargo build --workspace --release

# Run server
cd amp/server && cargo run --release

# Run UI
cd amp/ui && npm install && npm run dev
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [DOCS](docs/) | Agentic Memory Protocol Documentation |
| [DEV DOCS](devdocs/) | Development Documentation |
| [DEVLOG](amp/DEVLOG.md) | Development timeline |
| [SKILLS](SKILLS/) | Agent integration guide |
| [MCP Integration](amp/mcp-server/INTEGRATION.md) | MCP setup details |

---

## Tech Stack

**Backend**: Rust, Axum, Tokio, SurrealDB  
**Frontend**: React, TypeScript, Tauri, Three.js  
**CLI**: Rust, Ratatui, Tree-sitter  
**MCP**: Rust, rmcp SDK

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute.

---

## License

AMP is released under the **[AMP Community License v1.0](LICENSE)**.

- Free for research, personal projects, hackathons, and internal evaluation
- Commercial use requires a separate license
