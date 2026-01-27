# AMP Tool Examples

Real-world examples for each AMP tool with complete parameters and expected outputs.

---

## Cache Tools

### amp_cache_get - Load project context

```json
{
  "scope_id": "project:amp",
  "token_budget": 600
}
```

**Output**:
```
Memory Pack for scope: project:amp
--------------------------------------------------
Summary: AMP is an Agentic Memory Protocol with SurrealDB backend.

Facts:
  - Cache uses chars/4 for token estimation
  - Semantic dedup threshold is 0.92 cosine similarity
  - Default TTL is 30 minutes

Decisions:
  * Use SurrealDB for multi-model storage
  * Implement cache as Unity Layer for token efficiency

Token count: 487 / 600
Version: 12
Fresh: yes
```

### amp_cache_get - With semantic query

```json
{
  "scope_id": "project:amp",
  "token_budget": 800,
  "query": "authentication implementation"
}
```

Results prioritized by relevance to "authentication implementation".

### amp_cache_write - Store multiple items

```json
{
  "scope_id": "project:amp",
  "items": [
    {
      "kind": "fact",
      "preview": "CacheService creates embeddings for semantic dedup",
      "facts": [
        "Embeddings generated via EmbeddingService",
        "1536-dimension vectors stored in MTREE index"
      ],
      "importance": 0.7
    },
    {
      "kind": "warning",
      "preview": "Cache queries timeout after 5 seconds",
      "facts": ["Default timeout is 5s, configurable"],
      "importance": 0.8
    },
    {
      "kind": "decision",
      "preview": "Using POST for cache/pack endpoint",
      "facts": ["POST allows query embedding in body"],
      "importance": 0.6
    }
  ]
}
```

**Output**:
```
Cache write complete: 2 items written, 1 merged with existing
```
(One item had >0.92 similarity to existing, so it was merged.)

---

## Discovery Tools

### amp_status - Health check

```json
{}
```

**Output** (summarized):
```
{
  "health": {"status": "healthy", "version": "0.1.0"},
  "analytics": {
    "totalObjects": 668,
    "totalRelationships": 1311,
    "objectsByType": {
      "symbol": 546,
      "decision": 3,
      "changeset": 5,
      "filelog": 39,
      "filechunk": 40
    },
    "systemMetrics": {
      "cpuUsage": 2.4,
      "memoryUsage": 13.7,
      "uptime": "113h 10m"
    }
  }
}
```

### amp_list - Browse decisions

```json
{
  "type": "decision",
  "limit": 5
}
```

**Output**:
```
List of decision objects:

Found 3 objects:

1. Decision: Use SurrealDB for memory storage
2. Decision: Implement semantic cache Unity Layer
3. Decision: Use RRF for hybrid search ranking
```

### amp_list - Browse symbols by kind

```json
{
  "type": "symbol",
  "symbol_kind": "function",
  "limit": 10
}
```

**Output**:
```
List of symbol objects:

Found 10 objects:

1. Symbol: get_pack (function) in src/services/cache.rs
2. Symbol: write_items (function) in src/services/cache.rs
3. Symbol: handle_query (function) in src/handlers/query.rs
...
```


```json
{
  "goal": "implement rate limiting for API",
  "scope": "project:amp",
  "include_decisions": true
}
```

**Output**:
```
Context for: implement rate limiting for API
Scope: project:amp

Found 8 relevant items:

Key Symbols:
  1. middleware (module) in src/middleware/mod.rs
  2. rate_limit (function) in src/middleware/rate_limit.rs
  3. AppState (struct) in src/main.rs

Relevant Decisions:
  1. Use tower middleware for request processing (accepted)

Related Files:
  1. src/middleware/mod.rs
  2. src/config.rs
```

### amp_query - Hybrid search

```json
{
  "query": "error handling patterns",
  "mode": "hybrid",
  "filters": {"type": "symbol"},
  "limit": 5
}
```

**Output**:
```
Hybrid Query (RRF): error handling patterns

Found 5 results (ranked by Reciprocal Rank Fusion):

1. Symbol: handle_error (function) in src/handlers/error.rs
   id: 8a3b2c1d
   RRF Score: 0.4521 (text:0.312, vector:0.289)

2. Symbol: ErrorResponse (struct) in src/models/error.rs
   id: 9d4e5f6a
   RRF Score: 0.3892 (text:0.201, vector:0.412)

3. Symbol: map_err (function) in src/utils/result.rs
   id: 1e2f3a4b
   RRF Score: 0.3156 (text:0.156, vector:0.298)
...
```

### amp_trace - Follow relationships

```json
{
  "object_id": "8a3b2c1d-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "depth": 2
}
```

**Output**:
```
Trace for object: 8a3b2c1d-... (depth: 2)

Found 6 relationships:

1. 8a3b2c1d -> 9d4e5f6a (calls)
2. 8a3b2c1d -> 1e2f3a4b (depends_on)
3. 8a3b2c1d -> abc12345 (defined_in)
4. 9d4e5f6a -> def67890 (implements)
...
```

---

## Writing Tools

### amp_write_artifact - Create decision (ADR)

```json
{
  "type": "decision",
  "title": "Use Redis for session storage",
  "status": "accepted",
  "context": "Need fast session lookup. Options: in-memory, Redis, database. In-memory doesn't scale horizontally. Database adds latency.",
  "decision": "Redis chosen for sub-millisecond reads and built-in expiration",
  "consequences": "Adds Redis dependency. Need connection pooling. Sessions lost if Redis restarts without persistence.",
  "alternatives": [
    "In-memory with sticky sessions",
    "PostgreSQL with caching",
    "DynamoDB for serverless"
  ],
  "project_id": "project:myapp",
  "tenant_id": "default",
  "tags": ["architecture", "session", "redis"],
  "linked_files": ["src/config.rs", "src/middleware/session.rs"]
}
```

**Output**:
```
Artifact created: {
  "id": "587e6591-29d8-4bb9-a48b-cfe385845506",
  "type": "decision",
  "created_at": "2026-01-23T01:35:00.082Z"
}
```

### amp_write_artifact - Create changeset

```json
{
  "type": "changeset",
  "title": "Added rate limiting middleware",
  "description": "Implemented token bucket rate limiting with governor crate",
  "files_changed": [
    "src/middleware/rate_limit.rs",
    "src/middleware/mod.rs",
    "src/main.rs",
    "Cargo.toml"
  ],
  "diff_summary": "+120 lines\n\nAdded:\n- RateLimiter struct with token bucket algorithm\n- middleware function for axum integration\n- Configuration in main.rs (100 req/min per IP)\n\nDependencies:\n- governor crate added to Cargo.toml",
  "project_id": "project:myapp",
  "tenant_id": "default",
  "tags": ["middleware", "rate-limiting", "implementation"],
  "linked_decisions": ["587e6591-29d8-4bb9-a48b-cfe385845506"]
}
```

**Output**:
```
Artifact created: {
  "id": "abc12345-def6-7890-abcd-ef1234567890",
  "type": "changeset",
  "created_at": "2026-01-23T01:36:00.000Z"
}
```

### amp_write_artifact - Create note

```json
{
  "type": "note",
  "title": "Rate limiting gotchas",
  "category": "warning",
  "content": "# Rate Limiting Notes\n\n## Gotchas\n\n1. Token bucket resets at midnight UTC\n2. Whitelist internal IPs in production\n3. Return 429 with Retry-After header\n\n## Testing\n\nUse `wrk` or `ab` to verify limits work correctly.",
  "project_id": "project:myapp",
  "tenant_id": "default",
  "tags": ["middleware", "rate-limiting", "production"],
  "linked_files": ["src/middleware/rate_limit.rs"]
}
```

**Output**:
```
Artifact created: {
  "id": "note-12345678-abcd-efgh-ijkl-mnopqrstuvwx",
  "type": "note",
  "created_at": "2026-01-23T01:37:00.000Z"
}
```

### amp_write_artifact - Create filelog

```json
{
  "type": "filelog",
  "title": "Cache Service - Semantic Memory Module",
  "file_path": "src/services/cache.rs",
  "summary": "Cache service for semantic memory with TTL and dedup. Provides token-efficient memory packs for agent context injection. Handles automatic garbage collection of expired items.",
  "symbols": [
    "CacheService",
    "MemoryPack",
    "CacheItem",
    "get_pack",
    "write_items",
    "gc"
  ],
  "dependencies": [
    "database",
    "embedding",
    "serde"
  ],
  "project_id": "project:amp",
  "tenant_id": "default",
  "tags": ["cache", "semantic", "memory"]
}
```

**Output**:
```
Artifact created: {
  "id": "filelog-cache-rs-12345678",
  "type": "filelog",
  "created_at": "2026-01-23T01:38:00.000Z"
}
```

---

## File Tools

### amp_filelog_get - Read file info

```json
{
  "path": "src/services/cache.rs"
}
```

**Output**:
```json
{
  "file_log": {
    "file_path": "src/services/cache.rs",
    "summary": "Cache service for semantic memory with TTL and dedup",
    "purpose": "Token-efficient short-term memory for agents",
    "key_symbols": ["CacheService", "get_pack", "write_items", "MemoryPack"],
    "dependencies": ["database", "embedding"],
    "change_count": 3,
    "last_modified": "2026-01-23T01:30:00Z"
  }
}
```

### amp_filelog_update - Record change

```json
{
  "path": "src/services/cache.rs",
  "summary": "Added garbage collection method for expired items",
  "linked_changeset": "abc123-def456"
}
```

### amp_file_content_get - Retrieve content

```json
{
  "path": "src/services/cache.rs",
  "max_chars": 5000
}
```

---

## Run Tracking

### amp_run_start - Begin execution

```json
{
  "goal": "Implement user authentication feature",
  "repo_id": "project:myapp",
  "agent_name": "claude-code"
}
```

**Output**:
```
Run started: {
  "id": "run-12345678",
  "status": "running",
  "started_at": "2026-01-23T01:40:00Z"
}
```

### amp_run_end - Complete execution

```json
{
  "run_id": "run-12345678",
  "status": "completed",
  "outputs": [
    "Created auth.rs with JWT validation",
    "Added login/logout endpoints",
    "Updated middleware chain"
  ],
  "summary": "Authentication implemented with JWT tokens. Users can login via /api/auth/login and logout via /api/auth/logout. Protected routes require Bearer token."
}
```

---

## Coordination

### amp_lease_acquire - Lock resource

```json
{
  "resource": "file:src/auth.rs",
  "duration": 300,
  "agent_id": "claude-code-1"
}
```

**Output** (success):
```
Lease acquired: {
  "lease_id": "lease-abc123",
  "resource": "file:src/auth.rs",
  "holder": "claude-code-1",
  "expires_at": "2026-01-23T01:45:00Z"
}
```

**Output** (conflict):
```
HTTP 409 Conflict
Resource already leased by another agent
```

### amp_lease_release - Release lock

```json
{
  "lease_id": "lease-abc123"
}
```

**Output**:
```
Lease released: {"success": true, "message": "Lease released"}
```
