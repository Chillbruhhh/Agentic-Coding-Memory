# AMP MCP Server Integration Guide

Complete guide for integrating AMP MCP Server with AI agents.

## Quick Start

### 1. Build the MCP Server

**Linux/macOS:**
```bash
./scripts/build-mcp-server.sh
```

**Windows:**
```powershell
.\scripts\build-mcp-server.ps1
```

### 2. Start AMP Server

```bash
cd amp/server
cargo run --release
```

The server will start on `http://localhost:8105`

### 3. Configure Your Agent

See agent-specific instructions below.

## Agent Integration

### Claude Desktop

**Location:** 
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

**Configuration:**
```json
{
  "mcpServers": {
    "amp": {
      "command": "/absolute/path/to/amp-mcp-server",
      "args": [],
      "env": {
        "AMP_SERVER_URL": "http://localhost:8105",
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Restart Claude Desktop** after configuration.

### Cursor

**Location:** Cursor Settings → MCP Servers

**Configuration:**
```json
{
  "amp": {
    "command": "/absolute/path/to/amp-mcp-server",
    "args": [],
    "env": {
      "AMP_SERVER_URL": "http://localhost:8105"
    }
  }
}
```

### Windsurf

Similar to Cursor configuration.

### Testing with MCP Inspector

```bash
npx @modelcontextprotocol/inspector /path/to/amp-mcp-server
```

This opens a web UI to test all tools interactively.

## Docker Deployment

### Full Stack with Docker Compose

```bash
cd amp
docker-compose up -d
```

This starts:
- AMP Server (port 8105)
- AMP UI (port 3000)
- AMP MCP Server (stdio)

### MCP Server Only

```bash
cd amp/mcp-server
docker build -t amp-mcp-server .
docker run -e AMP_SERVER_URL=http://host.docker.internal:8105 amp-mcp-server
```

## Workflow Examples

### Example 1: Start a Development Session

```
Agent: Use amp_run_start
Input: {
  "goal": "Implement user authentication",
  "repo_id": "my-app",
  "agent_name": "claude"
}
Output: { "id": "run_abc123" }
```

### Example 2: Get Context for Task

```
Agent: Use amp_context
Input: {
  "goal": "authentication patterns and security",
  "scope": "repo",
  "include_recent": true,
  "include_decisions": true
}
Output: {
  "results": [
    { "id": "dec_001", "title": "Use JWT for auth", "score": 0.95 },
    { "id": "sym_042", "name": "authenticateUser", "score": 0.89 }
  ]
}
```

### Example 3: Coordinate File Access

```
Agent: Use amp_lease_acquire
Input: {
  "resource": "file:src/auth/jwt.ts",
  "duration": 300,
  "agent_id": "claude"
}
Output: { "lease_id": "lease_xyz789" }

[Agent makes changes]

Agent: Use amp_lease_release
Input: { "lease_id": "lease_xyz789" }
Output: { "success": true }
```

### Example 4: Document Changes

```
Agent: Use amp_filelog_update
Input: {
  "path": "src/auth/jwt.ts",
  "summary": "Added refresh token rotation and verifier cache",
  "linked_run": "run_abc123",
  "linked_changeset": "cs_def456"
}
Output: { "success": true, "file_log": {...} }
```

### Example 5: Record Decision

```
Agent: Use amp_write_decision
Input: {
  "title": "Use JWT with refresh token rotation",
  "context": "Need secure authentication with session management",
  "decision": "Implement JWT access tokens (15min) + refresh tokens (7 days) with rotation",
  "consequences": "Improved security, requires token storage management",
  "alternatives": ["Session cookies", "OAuth only", "API keys"]
}
Output: { "id": "dec_002" }
```

### Example 6: Complete Session

```
Agent: Use amp_run_end
Input: {
  "run_id": "run_abc123",
  "status": "success",
  "outputs": ["dec_002", "cs_def456"],
  "summary": "Implemented JWT authentication with refresh token rotation"
}
Output: { "success": true }
```

## Multi-Agent Coordination

### Scenario: Two Agents Working Simultaneously

**Agent A:**
```
1. amp_lease_acquire(resource="file:src/auth.ts", ...)
2. [Makes changes to auth.ts]
3. amp_lease_release(...)
```

**Agent B (concurrent):**
```
1. amp_lease_acquire(resource="file:src/auth.ts", ...)
   → Error: "Resource locked by Agent A"
2. [Works on different file or waits]
```

### Resource Naming Conventions

- Files: `file:path/to/file.ext`
- Modules: `module:auth`
- Features: `feature:user-login`
- Database: `database:users_table`

## Troubleshooting

### MCP Server Won't Start

**Check AMP Server:**
```bash
curl http://localhost:8105/health
```

**Check Logs:**
```bash
RUST_LOG=debug /path/to/amp-mcp-server
```

### Tools Not Appearing in Agent

1. Verify MCP server path is absolute
2. Check agent logs for connection errors
3. Restart agent after configuration changes
4. Test with MCP Inspector first

### Connection Timeouts

Increase timeout in configuration:
```json
{
  "env": {
    "AMP_SERVER_URL": "http://localhost:8105",
    "AMP_SERVER_TIMEOUT": "60"
  }
}
```

### Permission Errors

Ensure MCP server binary is executable:
```bash
chmod +x /path/to/amp-mcp-server
```

## Advanced Configuration

### Custom AMP Server URL

For remote AMP servers:
```json
{
  "env": {
    "AMP_SERVER_URL": "https://amp.example.com",
    "AMP_SERVER_TIMEOUT": "30"
  }
}
```

### Debug Logging

Enable detailed logging:
```json
{
  "env": {
    "RUST_LOG": "amp_mcp_server=debug,rmcp=debug"
  }
}
```

### Multiple AMP Instances

Configure different MCP servers for different projects:
```json
{
  "mcpServers": {
    "amp-project-a": {
      "command": "/path/to/amp-mcp-server",
      "env": { "AMP_SERVER_URL": "http://localhost:8105" }
    },
    "amp-project-b": {
      "command": "/path/to/amp-mcp-server",
      "env": { "AMP_SERVER_URL": "http://localhost:8106" }
    }
  }
}
```

## Best Practices

### 1. Always Start with amp_run_start

Track all agent sessions for audit trails.

### 2. Use Leases for File Modifications

Prevent conflicts in multi-agent scenarios.

### 3. Document Decisions

Use `amp_write_decision` for architectural choices.

### 4. Update File Logs

Keep file logs current with `amp_filelog_update`.

### 5. Complete Runs

Always call `amp_run_end` to link outputs.

### 6. Query Before Creating

Use `amp_query` to check for existing decisions/patterns.

## Performance Tips

- Use `amp_context` for broad searches
- Use `amp_query` with specific filters for targeted searches
- Set appropriate `limit` values to avoid large responses
- Use leases only when necessary (they add overhead)

## Security Considerations

- MCP server runs with user permissions
- AMP server should use authentication in production
- Use HTTPS for remote AMP servers
- Audit agent actions via Run objects
- Review lease usage for suspicious patterns

## Support

- GitHub Issues: [AMP Repository]
- Documentation: [AMP Docs]
- MCP Specification: https://modelcontextprotocol.io
