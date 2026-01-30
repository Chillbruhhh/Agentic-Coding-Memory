# AMP Development Guide

## Quick Start

```bash
# 1. Build and run server
cd server
cargo run

# 2. Test the API
curl http://localhost:8080/health

# 3. Run demo
./scripts/demo.sh
```

## Architecture Overview

AMP follows a layered architecture:

```
┌─────────────────────────────────────┐
│           Client SDKs               │
│     (Python, TypeScript)            │
├─────────────────────────────────────┤
│           HTTP API                  │
│        (OpenAPI v1)                 │
├─────────────────────────────────────┤
│         AMP Server                  │
│    (Rust + Axum + Tokio)           │
├─────────────────────────────────────┤
│        Storage Layer                │
│      (SurrealDB + Vector)           │
└─────────────────────────────────────┘
```

## Core Components

### 1. Object Types
- **Symbol**: Code structure (functions, classes, modules)
- **Decision**: Architecture and technical decisions
- **ChangeSet**: Code changes and modifications
- **Run**: Agent execution records

### 2. Memory Operations
- **Create**: Store new objects with provenance
- **Query**: Hybrid search (vector + graph + temporal)
- **Trace**: Deterministic query explanation
- **Coordinate**: Lease-based agent coordination

### 3. Retrieval Methods
- **Vector**: Semantic similarity search
- **Graph**: Relationship traversal
- **Temporal**: Time-based filtering
- **Hybrid**: Combined approach with ranking

## Development Workflow

### Server Development
```bash
cd server
cargo build    # Build
cargo test      # Test
cargo run       # Run server
```

### Schema Changes
1. Update `spec/schemas/*.json`
2. Update `spec/openapi.yaml`
3. Update `spec/schema.surql`
4. Regenerate SDKs: `./scripts/generate-sdks.sh`

### Adding New Object Types
1. Define JSON schema in `spec/schemas/`
2. Add to OpenAPI specification
3. Create SurrealDB table definition
4. Implement Rust model in `server/src/models/`
5. Add handlers in `server/src/handlers/`

## API Usage Examples

### Create a Symbol
```bash
curl -X POST http://localhost:8080/v1/objects \
  -H "Content-Type: application/json" \
  -d '{
    "type": "symbol",
    "tenant_id": "default",
    "project_id": "my_project",
    "provenance": {
      "agent": "indexer",
      "summary": "Discovered during code scan"
    },
    "name": "main",
    "kind": "function",
    "path": "src/main.rs",
    "language": "rust"
  }'
```

### Query Objects
```bash
curl -X POST http://localhost:8080/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "rust functions",
    "filters": {
      "type": ["symbol"],
      "project_id": "my_project"
    },
    "limit": 10
  }'
```

## Testing

### Unit Tests
```bash
cd server
cargo test
```

### Integration Tests
```bash
# Start server
cargo run &

# Run API tests
./scripts/test-api.sh

# Stop server
pkill amp-server
```

### Load Testing
```bash
# Install wrk
# brew install wrk  # macOS
# apt install wrk   # Ubuntu

# Test health endpoint
wrk -t12 -c400 -d30s http://localhost:8080/health
```

## Deployment

### Local Development
```bash
cargo run
```

### Docker
```bash
# Build image
docker build -t amp-server .

# Run container
docker run -p 8080:8080 amp-server
```

### Production
- Use `cargo build --release`
- Configure external SurrealDB instance
- Set up reverse proxy (nginx/traefik)
- Enable TLS/SSL
- Configure monitoring and logging

## Configuration

Environment variables:
- `DATABASE_URL`: SurrealDB connection string (default: "memory")
- `EMBEDDING_SERVICE_URL`: External embedding service
- `MAX_EMBEDDING_DIMENSION`: Vector dimension (default: 1536)
- `RUST_LOG`: Logging level (default: "info")

## Troubleshooting

### Server Won't Start
- Check if port 8080 is available
- Verify Rust installation: `cargo --version`
- Check logs for database connection issues

### API Errors
- Verify JSON schema compliance
- Check request headers (Content-Type)
- Review server logs for detailed errors

### Performance Issues
- Monitor database query performance
- Check vector index configuration
- Profile memory usage with `cargo flamegraph`

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature/new-feature`
3. Make changes and add tests
4. Run full test suite: `cargo test`
5. Update documentation
6. Submit pull request

## Roadmap

### Phase 1 (Hackathon) ✅
- [x] Core object schemas
- [x] OpenAPI specification
- [x] Server skeleton
- [x] Database schema
- [x] Basic API endpoints

### Phase 2 (Post-Hackathon)
- [ ] Complete storage implementation
- [ ] Vector embedding generation
- [ ] Hybrid query engine
- [ ] Generated SDKs
- [ ] Comprehensive tests

### Phase 3 (Production)
- [ ] Distributed deployment
- [ ] Advanced coordination
- [ ] UI/Dashboard
- [ ] Enterprise features
- [ ] Performance optimization
