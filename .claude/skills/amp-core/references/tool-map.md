# AMP Tool Map

This file describes when to use AMP tools and how they relate to memory layers.

## Memory tools

- `amp_cache_read`
  - Purpose: Read from episodic cache - search, get block, or get current context.
  - Use when: You need to retrieve cached context or search past blocks.
  - Modes:
    - `query` only → search closed blocks (returns summaries)
    - `query` + `include_content: true` → search with full content
    - `block_id` → get specific block
    - neither → get current open block
  - Scope: `project:{id}` or `task:{id}`.

- `amp_cache_write`
  - Purpose: Store short-term memory items with dedup.
  - Use when: Capture facts, decisions, snippets, or warnings mid-task.
  - Items: Always objects with `kind`, `content`, and optional `file_ref`.

## Durable knowledge tools

- `amp_write_artifact`
  - Purpose: Persist durable knowledge with full graph relationships.
  - Types: `decision`, `changeset`, `note`, `filelog`
  - Use when:
    - `decision`: An architectural choice affects future work
    - `changeset`: A unit of work is completed and should be recorded
    - `note`: Insights, warnings, or references to preserve
    - `filelog`: File metadata and symbol tracking

## File provenance tools

- `amp_filelog_get`
  - Purpose: Read dependency + symbol log for a file.
  - Use when: You need context before modifying a file.

- `amp_file_sync`
  - Purpose: Sync file state across all 3 memory layers (temporal, vector, graph) after code changes.
  - Use when: After ANY create, edit, or delete of a file. Sync SEQUENTIALLY (one at a time).

- `amp_file_path_resolve`
  - Purpose: Resolve canonical path for ambiguous or relative file input.
  - Use when: Path is ambiguous (multiple files match) or using relative paths.

## Focus tracking

- `amp_focus`
  - Purpose: Track active focus and completed outputs for sessions.
  - Use when: You want to record current task, mark completion, or list active sessions.

## Discovery & search tools

- `amp_status`
  - Purpose: Health check and system analytics.
  - Use when: Starting a session to verify connectivity, or checking what's indexed.

- `amp_query`
  - Purpose: Hybrid search combining text, vector, and graph retrieval.
  - Use when: Searching for specific knowledge across all stored data.
  - Modes: `hybrid` (default), `text`, `vector`, `graph`
  - Filters: `{"type": ["symbol"], "kind": ["function"]}` — both must be **arrays**.
  - Key params: `query` (required), `filters` (required, use `{}` for no filter), `graph_options` (required, use `{}`), `limit`.

- `amp_list`
  - Purpose: Browse stored objects by type.
  - Use when: Exploring what exists without a specific search query.
  - Shortcut: `type: "project"` auto-maps to `symbol` + `kind: "project"`.

- `amp_trace`
  - Purpose: Follow object relationships and provenance.
  - Use when: Exploring connections from a known object ID.

## Utility tools

- `amp_file_content_get`
  - Purpose: Retrieve indexed file content from memory chunks without reading from disk.
  - Use when: You need to review what AMP "knows" about a file, or access content without filesystem access.
  - Key params: `path` (required), `max_chars` (optional — limits output for large files).
  - Returns: `content` (full reconstructed text) and `chunks` (individual indexed segments).


