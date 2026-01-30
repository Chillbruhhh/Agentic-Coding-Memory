# AMP Tool Reference

Complete reference for AMP MCP tools with parameters and examples.

---

## Episodic Memory Cache (3 tools)

### `amp_cache_write`

Write an item to the current open cache block.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `scope_id` | string | No | run scope | Scope (e.g., `project:amp`) |
| `kind` | string | Yes | - | `fact`, `decision`, `snippet`, `warning` |
| `content` | string | Yes | - | Content to store |
| `importance` | number | No | 0.5 | 0.0-1.0 priority |
| `file_ref` | string | No | - | Associated file path |

```json
{
  "scope_id": "project:amp",
  "kind": "decision",
  "content": "Using Redis for session caching - faster than DB",
  "importance": 0.8
}
```

---

### `amp_cache_compact`

Close current block and open a new one. Call on conversation compact.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `scope_id` | string | No | Scope to compact (defaults to run scope) |

```json
{ "scope_id": "project:amp" }
```

---

### `amp_cache_read`

Unified cache read - list all blocks, search blocks, get specific block, or get current block.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `scope_id` | string | Yes | - | Scope identifier |
| `list_all` | boolean | No | false | List all blocks (newest first) |
| `query` | string | No | - | Search closed blocks by summary |
| `limit` | number | No | 5 | Max blocks when listing or searching |
| `include_content` | boolean | No | false | Return full content instead of summaries |
| `include_open` | boolean | No | false* | Include current open block in results |
| `block_id` | string | No | - | Get specific block by ID |

*Note: `include_open` defaults to `true` when `list_all=true` (since open block is current work).

**Mode selection (checked in order):**
1. `block_id` → get specific block (full content)
2. `list_all: true` → list newest blocks (summaries, includes open block)
3. `query` → search by summary (filtered results)
4. neither → get current open block only

**List all blocks (summaries - recommended for session start):**
```json
{
  "scope_id": "project:amp",
  "list_all": true
}
```

**List all blocks (with full content):**
```json
{
  "scope_id": "project:amp",
  "list_all": true,
  "include_content": true
}
```

**Search (summaries):**
```json
{
  "scope_id": "project:amp",
  "query": "authentication implementation",
  "limit": 5
}
```

**Search (full content):**
```json
{
  "scope_id": "project:amp",
  "query": "authentication implementation",
  "include_content": true
}
```

**Get specific block:**
```json
{
  "scope_id": "project:amp",
  "block_id": "cache_block:abc123..."
}
```

---

## File Provenance (2 tools)

### `amp_file_sync`

Sync file state across all memory layers (temporal, vector, graph).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | File path (flexible matching) |
| `action` | string | Yes | `create`, `edit`, `delete` |
| `summary` | string | Yes | 1-4 sentences describing change |
| `run_id` | string | No | Associated run ID |
| `agent_id` | string | No | Agent identifier |

```json
{
  "path": "src/auth/login.py",
  "action": "edit",
  "summary": "Added rate limiting to prevent brute force attacks"
}
```

**Path flexibility**: Accepts relative, absolute, or project-relative paths. Uses tiered matching with ambiguity detection.

**Fresh vs existing repos**: On first sync in a new codebase with `action: "create"`, auto-creates project node (detects root via `.git` or `.amp-root`). The "create" action triggers project initialization. For existing codebases, user must install AMP CLI and run `amp index` from the project root first.

**Ambiguous response** (when basename matches multiple files):
```json
{
  "status": "ambiguous",
  "message": "Ambiguous path - multiple files match",
  "input_path": "utils.py",
  "matching_files": ["src/utils.py", "lib/utils.py"],
  "hint": "Please use a more specific path"
}
```

---

### `amp_filelog_get`

Read file audit trail, symbols, and dependencies.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | File path |

```json
{ "path": "src/auth/login.py" }
```

**Ambiguity detection**: Same as `amp_file_sync` - returns `"status": "ambiguous"` with `matching_files` if basename matches multiple files.

---

## Discovery & Search (4 tools)

### `amp_status`

Health check and system analytics. No parameters.

---

### `amp_list`

Browse stored objects by type.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `type` | string | No | all | `symbol`, `decision`, `changeset`, `filelog`, `note` |
| `symbol_kind` | string | No | - | `file`, `function`, `class`, `project` |
| `limit` | number | No | 10 | Max results |

```json
{ "type": "decision", "limit": 5 }
```

---

### `amp_query`

Hybrid search combining text, vector, and graph retrieval.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `query` | string | Yes | - | Search query |
| `mode` | string | No | `hybrid` | `hybrid`, `text`, `vector`, `graph` |
| `filters` | object | Yes | {} | Type filters |
| `graph_options` | object | Yes | {} | Graph traversal options |
| `graph_autoseed` | boolean | No | false | Use text/vector hits as graph seed nodes |
| `graph_intersect` | boolean | No | false | Intersect graph results with text/vector |
| `limit` | number | No | 5 | Max results |

**Mode notes:**
- `hybrid` — Full text + vector + graph (RRF fusion). Requires embedding service; may timeout on slow connections. Falls back to `text` if it fails.
- `text` — Text search only. Fast, always works. Use as fallback.
- `graph_autoseed: true` — Takes top text/vector hits and traverses their graph edges to pull in connected nodes. Produces richer results. Supports `graph_options` overrides (see below).

**Autoseed with `graph_options` overrides:**

When `graph_autoseed: true`, you can pass `graph_options` **without** `start_nodes` to customize the autoseed traversal. The seed nodes are still auto-generated from text/vector hits, but depth, relation types, and direction use your values instead of defaults.

| `graph_options` field | Type | Default (autoseed) | Description |
|---|---|---|---|
| `max_depth` | number | `1` | How many hops from seed nodes. `1` = direct neighbors, `2` = neighbors of neighbors, etc. |
| `relation_types` | array | all 5 types | Edge types to follow: `depends_on`, `calls`, `implements`, `modifies`, `defined_in` |
| `direction` | string | `both` | Traversal direction: `outbound`, `inbound`, `both` |

**Autoseed examples:**

Default autoseed (depth 1, all relation types):
```json
{
  "query": "authentication middleware",
  "mode": "hybrid",
  "filters": {},
  "graph_options": {},
  "graph_autoseed": true,
  "limit": 10
}
```

Deeper autoseed (depth 2 — pulls in neighbors of neighbors):
```json
{
  "query": "authentication middleware",
  "filters": {},
  "graph_options": { "max_depth": 2 },
  "graph_autoseed": true,
  "limit": 10
}
```

Scoped autoseed (only follow hierarchy and dependency edges):
```json
{
  "query": "authentication middleware",
  "filters": {},
  "graph_options": { "max_depth": 2, "relation_types": ["defined_in", "depends_on"] },
  "graph_autoseed": true,
  "limit": 10
}
```

Outbound-only autoseed (what does this code depend on?):
```json
{
  "query": "cache service",
  "filters": {},
  "graph_options": { "direction": "outbound" },
  "graph_autoseed": true
}
```

---

### `amp_trace`

Trace object relationships and provenance.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `object_id` | string | Yes | - | Object ID to trace |
| `depth` | number | No | 2 | Traversal depth |

```json
{ "object_id": "abc123...", "depth": 2 }
```

---

## Writing Artifacts (1 tool)

### `amp_write_artifact`

Create artifacts with graph relationships.

**Common fields:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | Yes | `decision`, `changeset`, `note`, `filelog` |
| `title` | string | Yes | Artifact title |
| `project_id` | string | No | Project association |
| `linked_files` | array | No | Files to link |

**Decision fields:** `context`, `decision`, `consequences`, `alternatives`, `status`
**Changeset fields:** `description`, `files_changed`, `diff_summary`
**Note fields:** `content`, `category`

```json
{
  "type": "decision",
  "title": "Use Redis for caching",
  "context": "Need fast session storage",
  "decision": "Redis with 24h TTL",
  "consequences": "Requires Redis deployment"
}
```

---


## Focus Tracking (1 tool)

### `amp_focus`

Manage session focus and outputs.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `action` | string | Yes | `list`, `get`, `set`, `complete`, `end` |
| `run_id` | string | No | Defaults to current connection run |
| `title` | string | No | Focus title (for `set`) |
| `plan` | array | No | Plan steps (for `set`/`complete`) |
| `summary` | string | No | Completion summary (for `complete`) |
| `files_changed` | array | No | Files touched (for `complete`) |
| `project_id` | string | No | Filter `list` or set project on `set` |

**Examples**:
```json
{ "action": "list" }
```
```json
{ "action": "set", "title": "Fix cache UI", "plan": ["Repro", "Patch", "Verify"] }
```
```json
{ "action": "complete", "summary": "Cache UI fixed", "files_changed": ["ui/CachePanel.tsx"] }
```

## Utility (2 tools)

### `amp_file_content_get`

Retrieve indexed file content from chunks.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | File path |
| `max_chars` | number | No | Limit content length |

---

### `amp_file_path_resolve`

Resolve canonical stored path for ambiguous or relative file input.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | File path to resolve |

**When to use**:
- Path is ambiguous (basename matches multiple files)
- Relative path needs resolution
- Cross-platform path format issues

```json
{ "path": "utils.py" }
```

**Response**:
```json
{
  "input_path": "utils.py",
  "normalized_path": "utils.py",
  "tried_paths": ["utils.py", "src/utils.py"],
  "resolved_path": "c:/project/src/utils.py"
}
