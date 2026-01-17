# Agentic Memory Protocol (AMP)

A vendor-neutral protocol for durable, unified memory in agentic software development.

## Quick Start

```bash
# Start the AMP server
cd server
cargo run

# Use Python SDK
cd sdks/python
pip install -e .
python examples/basic_usage.py

# Use TypeScript SDK
cd sdks/typescript
npm install
npm run example
```

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

## Architecture

- **Server**: Rust + Axum + SurrealDB
- **SDKs**: Auto-generated Python and TypeScript clients
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
