# AMP Library Mode Roadmap (Post-Hackathon)

## Purpose

Define a scalable path to offer AMP as an **open-source, embeddable protocol framework** that can be integrated directly into any AI agent or agentic workflow. While the hackathon demo focused on coding agents, AMP is designed as a **general-purpose memory core** - an "oracle of truth" for any agent needing persistent, shared knowledge.

This document covers architecture, trade-offs, and phased delivery. No code changes are implied here.

## Vision

AMP aims to be the memory layer that gives any AI agent instant knowledge of:
- **What it's working on** - Current project, files, symbols, dependencies
- **What environment it's in** - OS, runtime, available tools, constraints
- **Where we are in the process** - Task status, blockers, next steps
- **What other agents have done** - Prior runs, decisions, artifacts
- **When and why** - Full provenance and audit trail

Think of AMP as giving an agent the same contextual awareness a team member developer would have.

## Goals

- Provide a stable, in-process AMP core that can be embedded in agent runtimes.
- Keep AMP server and MCP as the canonical external interfaces.
- Preserve feature parity for hybrid retrieval (text + vector + graph), file logs, and indexing.
- Minimize drift across SDKs and server by sharing schemas and tests.
- **NEW**: Support general knowledge ingestion (not just code).
- **NEW**: Offer executable API mode for efficient agent code execution.

## Non-Goals (for this phase)

- Replacing the AMP server as the default deployment.
- Rewriting existing UI or MCP tooling.
- Designing a new storage engine.

## Current State (Hackathon Demo)

- AMP runs as a standalone service with HTTP APIs and MCP tools.
- Indexing produces symbols, chunks, and file logs for **code** (10 languages).
- Hybrid retrieval merges text, vector, and graph results using **RRF (Reciprocal Rank Fusion)**.
- 13 MCP tools for cache, file sync, query, artifacts, and focus tracking.
- Focused on **coding agent unification** as the demonstration use case.

## Why a Library Mode

- Reduce latency in agent runtimes that can embed AMP in-process.
- Simplify local/offline setups where a separate service is undesirable.
- Enable direct hooks for lifecycle events (file edits, agent actions).
- Allow agents to **import AMP as a library** rather than calling an external service.

---

## Deployment Modes (Target)

| Mode | Description | Use Case |
|------|-------------|----------|
| **Service Mode** | External AMP server + HTTP clients | Multi-agent deployments, team setups |
| **Library Mode** | Embed `amp-core` in-process | Single-agent, low-latency, offline |
| **MCP Mode** | MCP tools for closed-source agents | Claude, Cursor, Windsurf integration |
| **Executable API Mode** | Generated code modules in sandbox | Efficient agent code execution (see below) |

---

## Architecture Direction

### Core Package (amp-core)

- Pure Rust core with feature flags for storage, embeddings, and graph.
- No HTTP or MCP dependencies in core.
- Provides a stable API for indexing, querying, and file log generation.

### Adapters

- **Storage adapter**: SurrealDB (primary), plus interface for others.
- **Embeddings adapter**: OpenAI, OpenRouter, Ollama, local.
- **Graph adapter**: Relation traversal and RRF scoring.
- **Ingestion adapter**: Docling, markdown parsers, structured data handlers.

### Server Package (amp-server)

- HTTP and MCP surface over amp-core.
- Minimal logic beyond routing, auth, and transport concerns.

### SDKs (Python and TS)

- Thin clients in service mode.
- Optional bindings to embedded core in library mode (phase 2+).

---

## Knowledge Ingestion Pipeline

Beyond code, AMP should ingest **any knowledge** and make it queryable.

### Document Processing

| Format | Approach |
|--------|----------|
| PDF, DOCX, HTML | Docling or similar → clean Markdown |
| Markdown | Direct ingestion with frontmatter extraction |
| JSON, YAML, CSV | Structured parsing with schema detection |
| URLs | Fetch + convert to Markdown |

### Ingestion API (Library Mode)

```rust
// Documents
ingest_document(path, format, options) -> DocumentId
ingest_url(url, options) -> DocumentId
ingest_directory(path, patterns, options) -> Vec<DocumentId>

// Chunking strategies
ChunkStrategy::Semantic      // LLM-based boundaries
ChunkStrategy::SlidingWindow // Fixed overlap
ChunkStrategy::Hierarchical  // Heading-based nesting

// Metadata extraction
extract_metadata(doc) -> {title, author, date, source, tags}
```

### Multi-Language Code Parsing

| Current (10) | Target (15+) |
|--------------|--------------|
| Python, TypeScript, JavaScript, Rust, Go, C#, Java, C, C++, Ruby | + Kotlin, Swift, Scala, PHP, Lua, Elixir |

### Prose/Documentation Parsing

- Markdown heading hierarchy → graph structure
- Code blocks → linked to language-specific symbols
- Frontmatter → metadata fields
- Links → relationship edges

---

## Executable API Mode

Based on Anthropic's [Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp) pattern.

### The Problem

Traditional MCP loads all tool definitions into context (token overhead) and passes intermediate results through the model (context bloat).

### The Solution

Present AMP tools as **code modules** that agents can import and execute directly:

```
amp/
├── memory/
│   ├── cache_read.ts
│   ├── cache_write.ts
│   ├── cache_compact.ts
│   └── index.ts
├── query/
│   ├── hybrid.ts
│   ├── vector.ts
│   ├── graph.ts
│   └── index.ts
├── files/
│   ├── sync.ts
│   ├── log_get.ts
│   ├── content_get.ts
│   └── index.ts
├── artifacts/
│   ├── write.ts
│   ├── list.ts
│   └── index.ts
└── index.ts
```

### How It Works

1. **Agent explores filesystem** to discover available tools
2. **Agent writes code** that imports and calls AMP modules
3. **Code executes in sandbox** - intermediate results stay local
4. **Only final results** return to model context

### Example: Before vs After

**Before (MCP direct calls):**
```
TOOL CALL: amp_query(query: "auth implementation")
→ returns 50KB of results into context
TOOL CALL: amp_filelog_get(path: "src/auth.py")
→ returns 10KB more into context
```

**After (Executable API):**
```typescript
import * as amp from './amp';

const results = await amp.query.hybrid({ query: "auth implementation" });
const authFiles = results.filter(r => r.path.includes('auth'));
const logs = await Promise.all(authFiles.map(f => amp.files.log_get({ path: f.path })));
const summary = logs.map(l => `${l.path}: ${l.symbols.length} symbols`);
console.log(summary); // Only this hits context
```

### Benefits

| Benefit | Description |
|---------|-------------|
| **Progressive Disclosure** | Load only the tools needed for current task |
| **Context Efficiency** | Filter/transform before returning to model |
| **Control Flow** | Loops, conditionals, error handling in code |
| **Privacy** | Tokenize sensitive data before model sees it |
| **State Persistence** | Write to `./workspace/` for resumable work |
| **Skills** | Agents save reusable functions with `SKILL.md` |

### Agent-Authored Skills

Agents can write reusable functions to `./skills/`:

```typescript
// ./skills/find-auth-code.ts
import * as amp from '../amp';

/**
 * Find all authentication-related code in the project.
 * @skill
 */
export async function findAuthCode(): Promise<AuthCodeResult> {
  const symbols = await amp.query.hybrid({ 
    query: "authentication login logout session token",
    limit: 50
  });
  return symbols.filter(s => s.type === 'function' || s.type === 'class');
}
```

AMP indexes these skills so future agents can discover and use them.

### Sandbox Integration

Reference configurations for secure execution:

- **Docker**: Isolated containers with resource limits
- **Firecracker**: Lightweight microVMs
- **gVisor**: User-space kernel for sandboxing
- **Network policies**: Allow/deny external connections
- **File access**: Whitelist for readable/writable paths

---

## Environment Awareness

Give agents the "team member" context they need.

### Agent Context API

```rust
// What environment am I in?
get_environment() -> Environment {
    os: "linux",
    runtime: "python3.11",
    working_directory: "/project",
    available_tools: ["git", "npm", "docker"],
    constraints: ["no network access", "read-only /etc"]
}

// What has happened on this project?
get_project_state(project_id) -> ProjectState {
    status: "in_progress",
    current_focus: "Authentication refactor",
    blockers: ["Waiting for API key"],
    recent_decisions: [...],
    active_agents: [...]
}

// What has this agent done before?
get_agent_history(agent_id) -> AgentHistory {
    runs: [...],
    artifacts_created: [...],
    files_modified: [...],
    decisions_made: [...]
}

// Who else is working on this?
get_active_agents(project_id) -> Vec<AgentInfo> {
    agent_id: "claude-code-123",
    focus: "API endpoints",
    files_locked: ["src/api/routes.py"],
    started_at: "2026-01-27T10:00:00Z"
}
```

### Team Knowledge Graph

```
                    ┌─────────────┐
                    │   Project   │
                    └──────┬──────┘
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
      ┌─────────┐    ┌─────────┐    ┌─────────┐
      │  Agent  │    │  Agent  │    │  Agent  │
      │   A     │    │   B     │    │   C     │
      └────┬────┘    └────┬────┘    └────┬────┘
           │              │              │
     ┌─────┴─────┐  ┌─────┴─────┐  ┌─────┴─────┐
     ▼           ▼  ▼           ▼  ▼           ▼
  ┌─────┐   ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐
  │ Run │   │ Run │ │ Run │ │ Run │ │ Run │ │ Run │
  └──┬──┘   └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘ └──┬──┘
     │         │       │       │       │       │
     ▼         ▼       ▼       ▼       ▼       ▼
  Artifacts, Decisions, Files, Focus Records...
```

Queries like:
- "Who last modified `auth.py`?"
- "What decisions led to using JWT?"
- "Is anyone else working on the API module?"

---

## API Surface (Library Mode)

### Indexing

```rust
index_project(path, options)
index_file(path, options)
update_file_log(path, delta)
```

### Ingestion

```rust
ingest_document(path, format, options)
ingest_url(url, options)
ingest_directory(path, patterns, options)
```

### Retrieval

```rust
query(text, vector, graph, options)
query_hybrid(text, options)  // Uses RRF fusion
query_graph(start_nodes, options)
```

### File Logs

```rust
get_file_log(path)
get_file_content(path, max_chars)
```

### Environment

```rust
get_environment()
get_project_state(project_id)
get_agent_history(agent_id)
get_active_agents(project_id)
```

### Observability

```rust
on_event(callback)  // For indexing and query telemetry
```

---

## Schema and Compatibility

- Define a shared schema for:
  - Objects (file, symbol, chunk, file log, document)
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
- Sandbox configs for executable API mode.

## Performance Targets

- Hybrid query latency parity with service mode.
- Indexing throughput configurable by worker count.
- Memory footprint acceptable for local agent runtimes.
- **Executable API**: 98%+ token reduction vs direct MCP calls (per Anthropic findings).

---

## Phased Delivery

### Phase 0: Design and Contracts
- Freeze core data model and query schemas.
- Write contract tests for server API.
- Document RRF algorithm and hybrid retrieval behavior.

### Phase 1: Core Extraction
- Split amp-core from amp-server.
- Move indexing, hybrid retrieval, graph traversal into amp-core.

### Phase 2: Library API (Rust)
- Provide direct Rust API for embedding.
- Integrate storage and embedding adapters.

### Phase 2.5: Knowledge Ingestion Pipeline
- Integrate Docling for document conversion.
- Add chunking strategies (semantic, sliding, hierarchical).
- Extend parsers to 15+ languages.
- Add prose/markdown parsing.

### Phase 3: Python SDK (Client + Embedded)
- Client: HTTP bindings to amp-server.
- Embedded: optional native bindings (pyo3) to amp-core.

### Phase 3.5: Executable API Mode
- Generate TypeScript/Python code modules from MCP tools.
- Filesystem-as-API structure for progressive disclosure.
- Reference sandbox configurations.
- Agent skill persistence and indexing.

### Phase 4: TypeScript SDK (Client + Embedded)
- Client: fetch/axios bindings.
- Embedded: optional node-native addon (if justified).

### Phase 5: Environment Awareness
- Implement `get_environment()`, `get_project_state()`, `get_agent_history()`.
- Team knowledge graph queries.
- Multi-agent coordination enhancements.

---

## Risks

- Duplicate logic between server and library if core extraction is incomplete.
- Dependency conflicts when embedded into agent runtimes.
- Feature drift between SDKs and server.
- **NEW**: Document ingestion quality varies by format.
- **NEW**: Sandbox security requires ongoing maintenance.

## Open Questions

- Do we require an embedded database in library mode, or allow ephemeral memory-only stores?
- Which embedding providers are mandatory for library mode v1?
- Should graph traversal be optional to reduce dependencies?
- **NEW**: Which document formats are priority for v1 ingestion?
- **NEW**: Do we generate executable APIs at build time or runtime?
- **NEW**: How do we handle agent identity across different runtimes?

## Success Criteria

- Hybrid query results match server mode for the same dataset.
- SDKs pass schema contract tests.
- Library mode can index and query without a running server.
- **NEW**: Document ingestion produces queryable chunks with metadata.
- **NEW**: Executable API mode achieves 90%+ token reduction in benchmarks.
- **NEW**: Environment APIs provide accurate context for agent decision-making.

---

## Future CLI Commands

```bash
amp query "search"           # Search memory (hybrid)
amp status                   # Check server status
amp tui                      # Launch terminal UI
amp clear                    # Clear database
amp ingest ./docs            # Ingest documents directory
amp ingest --url https://... # Ingest from URL
amp generate-api ./amp       # Generate executable API modules
amp env                      # Show environment context
amp agents                   # List active agents
```

---

## References

- [Anthropic: Code execution with MCP](https://www.anthropic.com/engineering/code-execution-with-mcp)
- [Anthropic: Equipping agents with skills](https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills)
- [Docling: Document conversion](https://github.com/DS4SD/docling)
- [RRF: Reciprocal Rank Fusion](docs/concepts/hybrid-retrieval.md)
