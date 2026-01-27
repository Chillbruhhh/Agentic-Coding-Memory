# MCP Integration Guide

Connect AI agents to AMP using the Model Context Protocol (MCP).

## What is MCP?

MCP (Model Context Protocol) is a standard protocol that allows AI agents to access external tools and data sources. AMP provides an MCP server that exposes 13 tools for AI agents.

## Quick Setup

### Prerequisites

- AMP server running (http://localhost:8105)
- MCP server running (http://localhost:8106)

### Start MCP Server

**Docker:**
```bash
docker compose up amp-mcp-server
```

**Manual:**
```bash
cd amp/mcp-server
cargo run --release
```

Verify it's running:
```bash
curl http://localhost:8106/health
```

## Agent Configuration

### Claude Code (CLI)

Add AMP to Claude Code:

```bash
claude mcp add amp -- npx mcp-remote@latest http://127.0.0.1:8106/mcp --allow-http
```

Verify:
```bash
claude mcp list
```

### Claude Desktop

Edit `claude_desktop_config.json`:

**Location:**
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

**Configuration:**
```json
{
  "mcpServers": {
    "amp": {
      "url": "http://localhost:8106/mcp"
    }
  }
}
```

Restart Claude Desktop.

### OpenCode

Edit `.opencode/mcp.json` in your project:

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

### Codex

Edit `~/.codex/mcp.toml`:

```toml
[mcp_servers.amp]
command = "npx"
args = ["mcp-remote@latest", "http://127.0.0.1:8106/mcp", "--allow-http"]
AMP_AGENT_NAME = "codex"
startup_timeout_sec = 30
```

### Kiro CLI

Edit `~/.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "amp": {
      "command": "npx",
      "args": ["mcp-remote", "http://localhost:8106/mcp"],
      "env": {
        "AMP_AGENT_NAME": "Kiro"
      }
    }
  }
}
```

### Cursor IDE

Edit Cursor settings:

1. Open Settings (Cmd/Ctrl + ,)
2. Search for "MCP"
3. Add server configuration:

```json
{
  "mcp.servers": {
    "amp": {
      "url": "http://localhost:8106/mcp"
    }
  }
}
```

## Available Tools (13 total)

AMP provides 13 MCP tools organized by category:

### Discovery & Status

**amp_status** - Server health and analytics
```typescript
amp_status()
// Returns: { status, object_counts, connected_agents, ... }
```

**amp_list** - Browse objects by type
```typescript
amp_list({
  type: "symbol",  // or "decision", "changeset", "note"
  limit: 50
})
```

### Memory & Retrieval

**amp_query** - Hybrid search across all memory
```typescript
amp_query({
  text: "authentication functions",
  hybrid: true,
  limit: 10
})
```

**amp_trace** - Object provenance and relationships
```typescript
amp_trace({
  object_id: "uuid-here",
  depth: 2
})
```

### Memory Writes

**amp_write_artifact** - Create decisions, notes, changesets
```typescript
// Create a decision
amp_write_artifact({
  type: "decision",
  title: "Use JWT for auth",
  context: "Need stateless authentication",
  decision: "Implement JWT tokens",
  consequences: "Requires token refresh logic"
})

// Create a note
amp_write_artifact({
  type: "note",
  title: "Auth implementation notes",
  content: "Remember to add rate limiting..."
})

// Create a changeset
amp_write_artifact({
  type: "changeset",
  description: "Add password hashing",
  files_changed: ["src/auth.py"],
  diff_summary: "+15 -3 lines"
})
```

### Focus Tracking

**amp_focus** - Track agent focus on files/modules
```typescript
amp_focus({
  path: "src/auth.py",
  action: "enter"  // or "exit"
})
```

### File Intelligence

**amp_filelog_get** - Retrieve file logs with symbols
```typescript
amp_filelog_get({
  path: "src/auth.py"
})
// Returns: { symbols, imports, exports, summary, ... }
```

**amp_file_sync** - Sync file after changes (creates provenance)
```typescript
amp_file_sync({
  path: "src/auth.py",
  action: "update",  // or "create", "delete"
  summary: "Added password hashing function"
})
```

**amp_file_content_get** - Fetch stored file content
```typescript
amp_file_content_get({
  path: "src/auth.py",
  max_chars: 10000
})
```

**amp_file_path_resolve** - Resolve canonical file paths
```typescript
amp_file_path_resolve({
  path: "auth.py"  // Returns: /full/path/to/src/auth.py
})
```

### Cache (Short-term Memory)

**amp_cache_write** - Write to session memory
```typescript
amp_cache_write({
  scope_id: "project:my-app",
  items: [
    { key: "current-task", value: "Implement auth" },
    { key: "files-in-progress", value: ["src/auth.py"] }
  ]
})
```

**amp_cache_read** - Read from session memory
```typescript
amp_cache_read({
  scope_id: "project:my-app"
})
// Returns current block with all items
```

**amp_cache_compact** - Compact cache (summarize old items)
```typescript
amp_cache_compact({
  scope_id: "project:my-app",
  max_tokens: 4000
})
```

## Usage Patterns

### Pattern 1: Codebase Understanding

```typescript
// 1. Query for relevant code
const results = await amp_query({
  text: "authentication logic",
  hybrid: true,
  limit: 10
});

// 2. Get file details
const fileLog = await amp_filelog_get({
  path: results[0].path
});

// 3. Trace relationships
const trace = await amp_trace({
  object_id: results[0].id,
  depth: 2
});
```

### Pattern 2: Making Changes

```typescript
// 1. Focus on the file
await amp_focus({
  path: "src/auth.py",
  action: "enter"
});

// 2. Make changes (your code here)

// 3. Record changeset
await amp_write_artifact({
  type: "changeset",
  description: "Refactored authentication",
  files_changed: ["src/auth.py", "tests/test_auth.py"],
  diff_summary: "+45 -20 lines"
});

// 4. Sync file to update provenance
await amp_file_sync({
  path: "src/auth.py",
  action: "update",
  summary: "Refactored to use JWT tokens"
});

// 5. Exit focus
await amp_focus({
  path: "src/auth.py",
  action: "exit"
});
```

### Pattern 3: Session Memory

```typescript
// At session start - read previous state
const cache = await amp_cache_read({
  scope_id: "project:my-app"
});

// During work - write progress
await amp_cache_write({
  scope_id: "project:my-app",
  items: [
    { key: "current_file", value: "src/auth.py" },
    { key: "task", value: "Add password validation" },
    { key: "progress", value: "50%" }
  ]
});

// Periodically - compact to save tokens
await amp_cache_compact({
  scope_id: "project:my-app",
  max_tokens: 4000
});
```

## Testing Your Integration

### 1. Verify Connection

```bash
# Check MCP server is accessible
curl http://localhost:8106/health
```

### 2. Test Tool Access

In your AI agent, try:

```
Query AMP for "test functions"
```

The agent should use `amp_query` tool and return results.

### 3. Test Write Operations

```
Create a decision record about using TypeScript
```

The agent should use `amp_write_artifact` tool.

### 4. Check Server Logs

```bash
# Docker
docker compose logs amp-mcp-server

# Manual
# Check terminal where MCP server is running
```

## Troubleshooting

### Agent Can't See Tools

**Check MCP server is running:**
```bash
curl http://localhost:8106/health
```

**Verify configuration:**
- Check config file location
- Ensure JSON is valid
- Restart agent after config changes

### Connection Refused

**Check ports:**
```bash
# Server should be on 8105
curl http://localhost:8105/health

# MCP should be on 8106
curl http://localhost:8106/health
```

**Check firewall:**
- Allow connections to ports 8105 and 8106
- On Windows, check Windows Defender

### Tools Return Errors

**Check server logs:**
```bash
docker compose logs amp-server
```

**Check authentication:**
- MCP server uses localhost by default
- No authentication required for local development

## Advanced Configuration

### Custom Agent Name

Set agent name in environment:

```json
{
  "env": {
    "AMP_AGENT_NAME": "my-custom-agent"
  }
}
```

### Custom Ports

If using non-default ports:

```json
{
  "url": "http://localhost:9000/mcp"
}
```

### Remote Server

For remote AMP server:

```json
{
  "url": "http://amp-server.example.com:8106/mcp"
}
```

Note: Ensure server allows remote connections (BIND_ADDRESS=0.0.0.0)
