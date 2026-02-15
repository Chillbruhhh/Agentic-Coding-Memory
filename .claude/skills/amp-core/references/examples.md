# AMP Tool Examples

Real-world examples for each AMP tool with complete parameters and expected outputs.

---

## Cache Tools

### amp_cache_read - Search with summaries

```json
{
  "scope_id": "project:amp",
  "query": "authentication implementation",
  "limit": 5
}
```

**Output**:
```
Cache search results for: "authentication implementation"
--------------------------------------------------

1. Block: cache_block:abc123... (relevance: 0.72)
   Created: 2026-01-23T10:30:00Z
   Summary: [decision] Using JWT for auth; [fact] Token expiry is 24h...

Tip: Use include_content=true to fetch full block content.
```

### amp_cache_read - Search with full content

```json
{
  "scope_id": "project:amp",
  "query": "authentication implementation",
  "include_content": true
}
```

**Output**:
```
Cache search results for: "authentication implementation" (with content)
==================================================

[1/2] Block: cache_block:abc123... (relevance: 0.72)
----------------------------------------
Summary: [decision] Using JWT for auth; [fact] Token expiry is 24h

  * [decision] Using JWT for stateless authentication
  - [fact] Token expiry is 24 hours with refresh tokens
  > [snippet] validate_jwt() in src/middleware/auth.rs
```

### amp_cache_read - Get specific block

```json
{
  "scope_id": "project:amp",
  "block_id": "cache_block:abc123..."
}
```

### amp_cache_read - Get current open block

```json
{
  "scope_id": "project:amp"
}
```

**Output**:
```
Memory Pack for scope: project:amp
--------------------------------------------------
Token count: 487 / 600
Version: 12
Fresh: yes
```

### amp_cache_write - Store an item

```json
{
  "scope_id": "project:amp",
  "kind": "fact",
  "content": "CacheService creates embeddings for semantic dedup",
  "importance": 0.7,
  "file_ref": "src/services/cache.rs"
}
```

**Output**:
```
Cache write complete: item written
```

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

### amp_query - Hybrid search with filters

```json
{
  "query": "error handling patterns",
  "filters": {"type": ["symbol"]},
  "graph_options": {},
  "limit": 5
}
```

**Output**:
```
Hybrid Query (RRF): error handling patterns

Found 5 results (ranked by Reciprocal Rank Fusion):

1. Symbol: handle_error (function) in src/handlers/error.rs
   id: 8a3b2c1d-xxxx-xxxx-xxxx-xxxxxxxxxxxx
   RRF Score: 0.0164

2. Symbol: ErrorResponse (struct) in src/models/error.rs
   id: 9d4e5f6a-xxxx-xxxx-xxxx-xxxxxxxxxxxx
   RRF Score: 0.0161
...
```

### amp_query - Filter by kind (e.g. find all projects)

```json
{
  "query": "project",
  "filters": {"type": ["symbol"], "kind": ["project"]},
  "graph_options": {},
  "limit": 20
}
```

**Output**:
```
Hybrid Query (RRF): project

Found 4 results (ranked by Reciprocal Rank Fusion):

1. Symbol: amp (project) in /app/amp
   id: 7de70d7c-e985-4158-bdf3-df16dac58f6e
   RRF Score: 0.0164

2. Symbol: myapp (project) in /home/user/myapp
   id: 223e2103-6080-4f2e-aebb-a3020ba7a293
   RRF Score: 0.0161
...
```

**Important:** `filters.type` and `filters.kind` must be **arrays**, not strings.

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

### amp_file_sync - Record change

```json
{
  "path": "src/services/cache.rs",
  "action": "edit",
  "summary": "Added garbage collection method for expired items"
}
```

### amp_file_content_get - Retrieve content

```json
{
  "path": "src/services/cache.rs",
  "max_chars": 5000
}
```

**Output**:
```json
{
  "path": "src/services/cache.rs",
  "content": "use crate::database::Database;\nuse serde::Serialize;\n\npub struct CacheService { ... }\n\nimpl CacheService {\n    pub async fn get_pack(...) { ... }\n    pub async fn write_items(...) { ... }\n}",
  "chunks": [
    "use crate::database::Database; use serde::Serialize; pub struct CacheService { ... }",
    "impl CacheService { pub async fn get_pack(...) { ... } pub async fn write_items(...) { ... } }"
  ]
}
```

The `content` field has the full reconstructed file. The `chunks` array shows the individual indexed segments. Use `max_chars` to limit output for large files.

### amp_file_path_resolve - Resolve ambiguous path

```json
{ "path": "utils.py" }
```

**Output (resolved)**:
```json
{
  "input_path": "utils.py",
  "normalized_path": "utils.py",
  "tried_paths": ["utils.py", "src/utils.py"],
  "resolved_path": "c:/project/src/utils.py"
}
```

**Output (ambiguous)**:
```json
{
  "input_path": "utils.py",
  "normalized_path": "utils.py",
  "tried_paths": ["utils.py"],
  "resolved_path": null,
  "error": "Multiple files match"
}
```

---


## Focus Tracking

### amp_focus - Set focus

```json
{
  "action": "set",
  "title": "Implement cache UI",
  "plan": [
    "Review CachePanel",
    "Add scope selector",
    "Verify scroll stability"
  ]
}
```

### amp_focus - Complete focus

```json
{
  "action": "complete",
  "summary": "Cache UI updated with scope selector and stable scroll.",
  "files_changed": ["amp/ui/src/components/CachePanel.tsx"],
  "plan": [
    "Review CachePanel",
    "Add scope selector",
    "Verify scroll stability"
  ]
}
```




