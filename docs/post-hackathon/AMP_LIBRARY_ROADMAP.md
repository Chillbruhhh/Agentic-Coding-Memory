# AMP Library Mode Roadmap (Post-Hackathon)

## Purpose
Define a scalable path to offer AMP as an embeddable library (in-process), while keeping the AMP server and MCP as first-class options. This document focuses on architecture, trade-offs, and phased delivery. No code changes are implied here.

## Goals
- Provide a stable, in-process AMP core that can be embedded in agent runtimes.
- Keep AMP server and MCP as the canonical external interfaces.
- Preserve feature parity for hybrid retrieval (text + vector + graph), file logs, and indexing.
- Minimize drift across SDKs and server by sharing schemas and tests.

## Non-Goals (for this phase)
- Replacing the AMP server as the default deployment.
- Rewriting existing UI or MCP tooling.
- Designing a new storage engine.

## Current State (Reference)
- AMP runs as a standalone service with HTTP APIs and MCP tools.
- Indexing produces symbols, chunks, and file logs.
- Hybrid retrieval merges text, vector, and graph results.

## Why a Library Mode
- Reduce latency in agent runtimes that can embed AMP in-process.
- Simplify local/offline setups where a separate service is undesirable.
- Enable direct hooks for lifecycle events (file edits, agent actions).

## Deployment Modes (Target)
1. Service mode (existing): external AMP server + clients.
2. Library mode: embed AMP core in-process, expose local API surface.
3. MCP mode: MCP tools remain a bridge for closed-source or remote agents.

## Architecture Direction
### Core Package (amp-core)
- Pure Rust core with feature flags for storage, embeddings, and graph.
- No HTTP or MCP dependencies in core.
- Provides a stable API for indexing, querying, and file log generation.

### Adapters
- Storage adapter: SurrealDB (primary), plus interface for others.
- Embeddings adapter: OpenAI, OpenRouter, Ollama, local.
- Graph adapter: relation traversal and scoring.

### Server Package (amp-server)
- HTTP and MCP surface over amp-core.
- Minimal logic beyond routing, auth, and transport concerns.

### SDKs (Python and TS)
- Thin clients in service mode.
- Optional bindings to embedded core in library mode (phase 2+).

## API Surface (Library Mode)
### Indexing
- index_project(path, options)
- index_file(path, options)
- update_file_log(path, delta)

### Retrieval
- query(text, vector, graph, options)
- query_hybrid(text, options)
- query_graph(start_nodes, options)

### File Logs
- get_file_log(path)
- get_file_content(path, max_chars)

### Observability
- on_event(callback) for indexing and query telemetry.

## Schema and Compatibility
- Define a shared schema for:
  - Objects (file, symbol, chunk, file log)
  - Relationships
  - Query request/response
- Publish versioned JSON schema to keep SDKs aligned.
- Add contract tests that run against server and library mode.

## Configuration Model
- Single config surface for both modes.
- Provider settings (LLM, embeddings) and storage settings are consistent.
- Avoid hidden defaults; log effective configuration at startup.

## Lifecycle and Concurrency
- Explicit init/shutdown for library mode.
- Bounded worker pools for indexing and embedding.
- Backpressure and cancellation support for agent runtimes.

## Security
- In-process mode should not require network exposure.
- Provide a policy hook for allow/deny file paths.
- Keep API key handling centralized and auditable.

## Performance Targets
- Hybrid query latency parity with service mode.
- Indexing throughput configurable by worker count.
- Memory footprint acceptable for local agent runtimes.

## Phased Delivery
### Phase 0: Design and Contracts
- Freeze core data model and query schemas.
- Write contract tests for server API.

### Phase 1: Core Extraction
- Split amp-core from amp-server.
- Move indexing, hybrid retrieval, graph traversal into amp-core.

### Phase 2: Library API (Rust)
- Provide direct Rust API for embedding.
- Integrate storage and embedding adapters.

### Phase 3: Python SDK (Client + Embedded)
- Client: HTTP bindings to amp-server.
- Embedded: optional native bindings (pyo3) to amp-core.

### Phase 4: TypeScript SDK (Client + Embedded)
- Client: fetch/axios bindings.
- Embedded: optional node-native addon (if justified).

## Risks
- Duplicate logic between server and library if core extraction is incomplete.
- Dependency conflicts when embedded into agent runtimes.
- Feature drift between SDKs and server.

## Open Questions
- Do we require an embedded database in library mode, or allow ephemeral memory-only stores?
- Which embedding providers are mandatory for library mode v1?
- Should graph traversal be optional to reduce dependencies?

## Success Criteria
- Hybrid query results match server mode for the same dataset.
- SDKs pass schema contract tests.
- Library mode can index and query without a running server.

## Next Steps (Post-Hackathon)
- Review this roadmap and lock the core schema.
- Create a minimal amp-core crate boundary.
- Build first Rust API and validate with a small example agent.
