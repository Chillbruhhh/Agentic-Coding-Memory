# File Sync & Provenance Guide

`amp_file_sync` is the unified write endpoint for keeping the codebase index in sync
when files are created, edited, or deleted.

## What It Does

Syncs file state across all three memory layers:

```
┌─────────────────────────────────────────────────────────┐
│                    amp_file_sync                        │
│              path + action + summary                    │
└───────────────────────┬─────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│   TEMPORAL    │ │    VECTOR     │ │     GRAPH     │
│   (FileLog)   │ │  (Chunks +    │ │ (Relationships│
│               │ │  Embeddings)  │ │    depends_on)│
├───────────────┤ ├───────────────┤ ├───────────────┤
│ - Audit trail │ │ - Re-chunk    │ │ - Update deps │
│ - Symbols     │ │ - 100-token   │ │ - Link files  │
│ - Dependencies│ │   overlap     │ │               │
│ - Summary     │ │ - Embeddings  │ │               │
└───────────────┘ └───────────────┘ └───────────────┘
```

## Tool: amp_file_sync

```json
{
  "path": "src/auth/login.py",
  "action": "edit",
  "summary": "Added rate limiting to prevent brute force attacks",
  "run_id": "run-abc123",
  "agent_id": "claude-1"
}
```

**Parameters:**
- `path` (required): File path - flexible matching (relative or absolute)
- `action` (required): "create" | "edit" | "delete"
- `summary` (required): 1-4 sentences describing the change
- `run_id` (optional): Link to agent run for audit trail
- `agent_id` (optional): Agent identifier for audit trail

**Returns:**
```json
{
  "file_id": "file-74bc6687cfbd...",
  "action": "edit",
  "layers_updated": {
    "temporal": true,
    "vector": true,
    "graph": false
  },
  "audit_entry_added": true,
  "chunks_replaced": 3,
  "relationships_updated": 0
}
```

## Path Flexibility

The tool uses **tiered path matching** with ambiguity detection:

```
✓ "src/auth/login.py"                    (relative)
✓ "test-repo/python/sample.py"           (project-relative)
✓ "C:\\Users\\...\\src\\auth\\login.py"  (absolute Windows)
✓ "/home/user/project/src/auth/login.py" (absolute Unix)
✓ "\\\\?\\C:\\Users\\...\\login.py"      (Windows extended)
```

**Matching tiers (in order):**
1. **Tier 1 - Specific match**: Exact path, path contains input, normalized path
2. **Tier 2 - Basename match**: Falls back to filename only (with ambiguity check)

**Ambiguity detection:**
If basename-only matching finds multiple files (e.g., `utils.py` exists in multiple directories), the tool returns a successful response with `status: "ambiguous"` listing all matching paths:

```json
{
  "status": "ambiguous",
  "message": "Ambiguous path - multiple files match",
  "input_path": "utils.py",
  "matching_files": [
    "src/utils/utils.py",
    "tests/fixtures/utils.py",
    "lib/helpers/utils.py"
  ],
  "hint": "Please use a more specific path (e.g., include parent directory)"
}
```

This is a **successful tool call** - the tool found the files, it just needs clarification on which one. Use the `matching_files` list to select the correct path and retry.

**Best practice:** Always include at least the parent directory (e.g., `auth/login.py` instead of just `login.py`) to avoid ambiguity in larger codebases.

## Actions

### create

Use when a new file is created:
```json
{
  "path": "src/new-feature.py",
  "action": "create",
  "summary": "New module for user preferences API"
}
```

What happens:
- Creates new FileLog with audit entry
- Chunks file content with 100-token overlap
- Generates embeddings for each chunk
- Creates dependency graph relationships

### edit

Use when an existing file is modified:
```json
{
  "path": "src/auth/login.py",
  "action": "edit",
  "summary": "Added rate limiting, fixed SQL injection vulnerability"
}
```

What happens:
- Adds audit entry to existing FileLog
- Increments change_count
- Deletes old chunks, creates new ones
- Regenerates embeddings
- Updates dependency relationships

### delete

Use when a file is removed:
```json
{
  "path": "src/deprecated/old-api.py",
  "action": "delete",
  "summary": "Removed deprecated v1 API endpoints"
}
```

What happens:
- Adds deletion audit entry to FileLog (soft delete)
- Removes all chunks for file
- Removes relationships (depends_on, defined_in, etc.)

## Tool: amp_filelog_get

Read file audit trail, symbols, and dependencies:

```json
{
  "path": "src/auth/login.py"
}
```

**Returns:**
```json
{
  "file_path": "src/auth/login.py",
  "file_id": "file-74bc6687...",
  "summary": "Python file with: login, validate_token, ...",
  "key_symbols": ["login", "validate_token", "refresh_token"],
  "dependencies": ["jwt", "redis", "hashlib"],
  "audit_trail": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "action": "create",
      "summary": "Initial implementation"
    },
    {
      "timestamp": "2024-01-16T14:20:00Z",
      "action": "edit",
      "summary": "Added rate limiting"
    }
  ],
  "change_count": 2
}
```

## Writing Good Summaries

The summary appears in the audit trail and helps future agents understand changes.

**Good summaries:**
```
"Added input validation to prevent XSS attacks in form handler"
"Refactored database queries to use connection pooling"
"Fixed race condition in concurrent session handling"
"Removed deprecated v1 API endpoints, updated imports"
```

**Poor summaries:**
```
"Updated file"          // Too vague
"Changes"               // No information
"Bug fix"               // What bug?
"Refactoring"           // What was refactored?
```

## Workflow: Post-Edit Pattern

After any code modification:

```python
# 1. Sync the file
amp_file_sync({
    path: "src/auth/login.py",
    action: "edit",
    summary: "Added rate limiting to prevent brute force"
})

# 2. Optionally cache the context
amp_cache_write({
    scope_id: "project:my-app",
    kind: "decision",
    content: "Implemented 5-attempt rate limit with 15-min lockout",
    importance: 0.7
})
```

## Workflow: File Investigation

Before modifying unfamiliar code:

```python
# 1. Get file context
amp_filelog_get({ path: "src/auth/login.py" })

# 2. Check recent changes in audit trail
# 3. Review dependencies and symbols
# 4. Make informed modifications
# 5. Sync changes
amp_file_sync({ ... })
```

## Best Practices

1. **Sync after every edit** - Keep index current
2. **Sync files SEQUENTIALLY** - Never call multiple syncs in parallel (see below)
3. **Write descriptive summaries** - Help future agents
4. **Include context** - What and why, not just what
5. **Use run_id/agent_id** - Enables audit trail queries
6. **Check before overwriting** - Use amp_filelog_get first

## CRITICAL: Sequential Syncing

**NEVER call multiple `amp_file_sync` requests in parallel.** The server can timeout or fail when processing multiple sync operations simultaneously.

Each sync operation:
- Reads file content from disk
- Parses and extracts symbols
- Re-chunks content with embeddings
- Updates graph relationships

This is resource-intensive. Parallel requests overwhelm the server.

**CORRECT - Sync files one at a time:**

```
# Modified 3 files - sync them sequentially
amp_file_sync(path: "src/auth.py", action: "edit", summary: "Added login endpoint")
# Wait for response...
amp_file_sync(path: "src/middleware.py", action: "edit", summary: "Added auth middleware")
# Wait for response...
amp_file_sync(path: "src/routes.py", action: "edit", summary: "Registered auth routes")
```

**WRONG - Parallel syncing causes timeouts:**

```
# DO NOT DO THIS - calling all syncs in parallel
# This will cause server timeouts and failures
[parallel calls to amp_file_sync] ← WRONG
```

**Rule of thumb:** One sync, wait for response, next sync.

## Fresh vs Existing Repositories

`amp_file_sync` behaves differently depending on whether the codebase has been indexed:

### Fresh Repository (New Project)

When syncing files in a **brand-new codebase** that has never been indexed:

1. **Use `action: "create"`** - This triggers project node auto-creation
2. **Auto-creates project node** on first `amp_file_sync` call with `action: "create"`
3. Detects project root via:
   - `.git` directory (preferred)
   - `.amp-root` marker file (fallback)
4. Creates project FileLog for the repository
5. New file nodes attach to directory and project nodes via `defined_in`

**IMPORTANT:** You must use `action: "create"` (not "edit") to trigger the auto-creation. The "create" action is what initializes the project node.

No manual indexing required - just start syncing files as you create them:

### Existing Repository (Unindexed)

When working with an **existing codebase** that hasn't been indexed:

1. User must run `amp index` first
2. Without indexing:
   - `amp_file_sync` with action "edit" may fail
   - `amp_query` returns empty results
   - File context is missing

**If you encounter empty results or errors on an existing codebase, instruct the user:**

> "This repository hasn't been indexed yet. Please run `amp index` from your project root to initialize AMP memory for this codebase."

### Running `amp index`

**Prerequisites:**
- AMP CLI must be installed (`cargo install amp-cli` or from releases)
- Must be run from the **project root directory**

```bash
# Navigate to project root first
cd /path/to/your/project

# Run indexer
amp index
```

The indexer scans the codebase and populates all three memory layers (temporal, vector, graph) with existing files, symbols, and relationships.

