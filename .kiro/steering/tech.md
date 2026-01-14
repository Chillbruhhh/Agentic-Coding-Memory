# Technical Architecture

## Technology Stack
**Core Stack**:
- **Server**: Rust + Axum + Tokio (async runtime)
- **Database**: SurrealDB (embedded with vector indexing)
- **API**: HTTP + JSON with OpenAPI v1 specification
- **SDKs**: Auto-generated Python and TypeScript clients

**Dependencies**:
- Axum 0.7 (web framework)
- SurrealDB 1.0 (database with vector support)
- Serde (JSON serialization)
- UUID + Chrono (identifiers and timestamps)
- Tower-HTTP (CORS and tracing middleware)
- Reqwest (HTTP client for external services)

## Architecture Overview
AMP follows a layered architecture with protocol-first design:

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

**Core Components**:
- **Memory Objects**: Symbol, Decision, ChangeSet, Run with base fields and relationships
- **Hybrid Retrieval**: Vector similarity + Graph traversal + Temporal filtering
- **Coordination**: Lease-based system for multi-agent coordination
- **Traceability**: Deterministic query explanation and provenance tracking

## Development Environment
**Required Tools**:
- Rust 1.70+ with Cargo
- SurrealDB (embedded, no separate installation needed)
- Python 3.8+ (for SDK examples)
- Node.js 16+ (for TypeScript SDK examples)
- OpenAPI Generator (for SDK generation)

**Setup Instructions**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <repo>
cd amp/server
cargo build

# Run server
cargo run

# Generate SDKs (optional)
./scripts/generate-sdks.sh
```

## Code Standards
**Rust Standards**:
- Use `rustfmt` for consistent formatting
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Prefer `Result<T, E>` for error handling
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` for data structures
- Document public APIs with `///` comments

**Protocol Standards**:
- All objects must include base fields (id, type, tenant_id, project_id, timestamps, provenance)
- Use UUID v4 for all identifiers
- ISO 8601 timestamps in UTC
- JSON Schema validation for all API inputs
- OpenAPI specification drives all client generation

## Testing Strategy
**Testing Approach**:
- Unit tests for all core functions using Rust's built-in test framework
- Integration tests for API endpoints using test client
- Property-based testing for schema validation
- Load testing for performance benchmarks
- Mock external services for reliable testing

**Test Organization**:
- `tests/unit/` - Unit tests alongside source code
- `tests/integration/` - API integration tests
- `tests/load/` - Performance and load tests
- `examples/` - Example usage that doubles as integration tests

## Deployment Process
[How code gets from development to production]

## Performance Requirements
[Speed, scalability, and resource constraints]

## Security Considerations
[Security practices, authentication, and data protection]
