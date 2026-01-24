# AMP Tool Reference

Complete reference for all 17 AMP MCP tools with parameters and examples.

---

## Episodic Memory Cache (4 tools)

### `amp_cache_write`

Write an item to the current open cache block.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `scope_id` | string | Yes | - | Scope (e.g., `project:amp`) |
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
| `scope_id` | string | Yes | Scope to compact |

```json
{ "scope_id": "project:amp" }
```

---

### `amp_cache_search`

Search closed block summaries (two-phase retrieval).

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `scope_id` | string | Yes | - | Scope to search |
| `query` | string | Yes | - | Search query |
| `limit` | number | No | 5 | Max results |

```json
{
  "scope_id": "project:amp",
  "query": "authentication implementation",
  "limit": 5
}
```

---

### `amp_cache_get`

Get a specific block by ID or legacy memory pack.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `scope_id` | string | Yes | Scope identifier |
| `block_id` | string | No | Block ID to retrieve |

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

## Discovery & Search (5 tools)

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

### `amp_context`

Get high-signal context bundle for a goal.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `goal` | string | Yes | - | What you're accomplishing |
| `scope` | string | Yes | - | Project scope |
| `include_recent` | boolean | No | false | Include recent activity |
| `include_decisions` | boolean | No | false | Prioritize decisions |

```json
{
  "goal": "implement user authentication",
  "scope": "project:myapp",
  "include_decisions": true
}
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
| `limit` | number | No | 5 | Max results |

```json
{
  "query": "authentication middleware",
  "mode": "hybrid",
  "filters": {"type": "symbol"},
  "graph_options": {},
  "limit": 10
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

## Run Tracking (2 tools)

### `amp_run_start`

Begin tracking an agent execution.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `goal` | string | Yes | Run objective |
| `repo_id` | string | Yes | Repository ID |
| `agent_name` | string | Yes | Agent identifier |

---

### `amp_run_end`

Complete an agent execution.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `run_id` | string | Yes | Run ID from start |
| `status` | string | Yes | `completed`, `failed`, `cancelled` |
| `outputs` | array | Yes | Output strings |
| `summary` | string | Yes | What was accomplished |

---

## Coordination (2 tools)

### `amp_lease_acquire`

Lock a shared resource.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `resource` | string | Yes | Resource identifier |
| `duration` | number | Yes | Lease seconds |
| `agent_id` | string | Yes | Requesting agent |

```json
{
  "resource": "file:src/auth.rs",
  "duration": 300,
  "agent_id": "claude-1"
}
```

---

### `amp_lease_release`

Release a held lease.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `lease_id` | string | Yes | Lease ID |

---

## Utility (1 tool)

### `amp_file_content_get`

Retrieve indexed file content from chunks.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | File path |
| `max_chars` | number | No | Limit content length |
