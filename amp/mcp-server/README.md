# AMP MCP Server

Model Context Protocol (MCP) server for the Agentic Memory Protocol (AMP). Exposes AMP's memory capabilities as tools for AI agents like Claude Desktop and Cursor.

## Features

- **10 Focused Tools** for agent memory management
- **Hybrid Retrieval** combining text, vector, and graph search
- **Multi-Agent Coordination** via resource leases
- **File Intelligence** with symbol tracking and change documentation
- **Execution Tracking** for agent runs and outputs

## Installation

### Prerequisites

- Rust 1.70+
- Running AMP server (default: http://localhost:8105)

### Build

```bash
cd amp/mcp-server
cargo build --release
```

The binary will be at `target/release/amp-mcp-server`

## Configuration

Create a `.env` file or set environment variables:

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

## Usage

### Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

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

### Cursor

Add to Cursor's MCP configuration.

### Direct Usage

```bash
# Run the server (stdio transport)
./amp-mcp-server

# With custom configuration
AMP_SERVER_URL=http://localhost:8105 ./amp-mcp-server
```

## Available Tools

### Retrieval

**amp_query** - Hybrid search across memory
- Input: `query`, `mode` (hybrid/text/vector/graph), `filters`, `graph_options`
- Output: Ranked results with explanations

**amp_trace** - Trace object provenance and relationships
- Input: `object_id`, `depth`
- Output: Relationship graph

### Memory Writes

**amp_write_decision** - Create architectural decision record
- Input: `title`, `context`, `decision`, `consequences`, `alternatives`
- Output: Created Decision object ID

**amp_write_changeset** - Document completed work
- Input: `description`, `files_changed`, `diff_summary`, `linked_decisions`
- Output: Created ChangeSet object ID

**amp_run_start** - Begin execution tracking
- Input: `goal`, `repo_id`, `agent_name`
- Output: Run object ID

**amp_run_end** - Complete execution
- Input: `run_id`, `status`, `outputs`, `summary`
- Output: Updated Run object

### File Intelligence

**amp_filelog_get** - Retrieve file log
- Input: `path`
- Output: File log with symbols, dependencies, changes

**amp_filelog_update** - Update file after changes
- Input: `path`, `summary`, `linked_run`, `linked_changeset`
- Output: Updated file log

### Coordination

**amp_lease_acquire** - Acquire resource lease
- Input: `resource`, `duration`, `agent_id`
- Output: Lease ID or error if locked

**amp_lease_release** - Release resource lease
- Input: `lease_id`
- Output: Success confirmation

### Discovery

**amp_status** - Get server health and analytics
- Input: None
- Output: Health status and object counts

**amp_list** - Browse objects by type
- Input: `type`, `limit`, `sort`
- Output: List of objects

## Agent Workflow Example

```
1. amp_run_start(goal="Implement auth", repo_id="my-app", agent_name="claude")
   → Returns run_id


2. amp_lease_acquire(resource="file:src/auth.ts", duration=300, agent_id="claude")
   → Acquires exclusive access

3. [Agent makes changes to auth.ts]

4. amp_filelog_update(path="src/auth.ts", summary="Added JWT auth", linked_run=run_id)
   → Documents changes

5. amp_write_decision(title="Use JWT", context="...", decision="...", consequences="...")
   → Records architectural decision

6. amp_lease_release(lease_id=lease_id)
   → Releases file lock

7. amp_run_end(run_id=run_id, status="success", outputs=[decision_id], summary="Auth implemented")
   → Completes execution tracking
```

## Development

### Project Structure

```
src/
├── main.rs              # MCP server entry point
├── amp_client.rs        # HTTP client for AMP API
├── config.rs            # Configuration management
└── tools/               # Tool implementations
    ├── mod.rs           # Tool registry
    ├── query.rs         # amp_query, amp_trace
    ├── memory.rs        # write_decision, write_changeset, run_start/end
    ├── files.rs         # filelog_get, filelog_update
    ├── coordination.rs  # lease_acquire, lease_release
    └── discovery.rs     # amp_status, amp_list
```

### Testing

```bash
# Build
cargo build

# Run with logging
RUST_LOG=debug cargo run

# Test with MCP Inspector
npx @modelcontextprotocol/inspector cargo run
```

## Troubleshooting

### Connection Issues

- Ensure AMP server is running on configured URL
- Check `AMP_SERVER_URL` environment variable
- Verify network connectivity

### Tool Errors

- Check AMP server logs for API errors
- Verify tool input schemas match expectations
- Enable debug logging: `RUST_LOG=debug`

### Agent Integration

- Verify MCP server path in agent configuration
- Check agent logs for connection errors
- Test with MCP Inspector first

## License

Same as AMP project