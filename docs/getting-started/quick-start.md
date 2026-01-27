# Quick Start Guide

Get AMP running in 5 minutes.

## Prerequisites

- Docker (recommended) OR
- Rust 1.70+ and Node.js 16+

## Option 1: Docker (Recommended)

The fastest way to get started:

```bash
# Clone the repository
git clone <repo-url>
cd amp

# Start all services
docker compose up
```

This starts:
- **AMP Server**: http://localhost:8105
- **MCP Server**: http://localhost:8106
- **Desktop UI**: http://localhost:8109
- **SurrealDB**: localhost:7505

## Option 2: Manual Setup

### Install CLI Tool

**Windows:**
```powershell
.\scripts\install.ps1
```

**Linux/macOS:**
```bash
./scripts/install.sh
```

### Start the Server

```bash
cd amp/server
cargo run --release
```

Server starts on http://localhost:8105

### Index Your First Codebase

```bash
# Index a project
amp index /path/to/your/project

# Query the indexed code
amp query "authentication functions"
```

## Verify Installation

Check that everything is working:

```bash
# Health check
curl http://localhost:8105/health

# Should return: {"status":"healthy"}
```

## Next Steps

1. **Configure an AI Agent**: See [MCP Integration](../guides/agents/mcp-integration.md)
2. **Explore the UI**: Open http://localhost:8109 in your browser
3. **Learn Core Concepts**: Read [What is AMP?](../concepts/what-is-amp.md)
4. **Try Examples**: Check out [Basic Usage](../examples/basic-usage.md)

## Quick Test

Create a test symbol:

```bash
curl -X POST http://localhost:8105/v1/objects \
  -H "Content-Type: application/json" \
  -d '{
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "demo",
    "provenance": {
      "agent": "manual",
      "summary": "Quick start test"
    },
    "name": "hello_world",
    "kind": "function",
    "path": "test.py",
    "language": "python",
    "signature": "def hello_world():",
    "documentation": "A test function"
  }'
```

Query it back:

```bash
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "hello",
    "limit": 5
  }'
```

## Troubleshooting

**Port already in use:**
```bash
# Change the port in .env
PORT=8106
```

**Docker issues:**
```bash
# Rebuild containers
docker compose down
docker compose up --build
```

**Can't connect to server:**
- Check firewall settings
- Verify server is running: `curl http://localhost:8105/health`
- Check logs: `docker compose logs amp-server`

## Getting Help

- [Common Issues](../troubleshooting/common-issues.md)
- [Configuration Guide](configuration.md)
- [Full Installation Guide](installation.md)

## NUCLEAR DELETE QUERY

```sql
DELETE FROM objects;
DELETE FROM relationships;
DELETE FROM defined_in WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);
DELETE FROM depends_on WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);  
DELETE FROM calls WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);
```