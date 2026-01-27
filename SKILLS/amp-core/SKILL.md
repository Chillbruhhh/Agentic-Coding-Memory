---
name: amp-core
description: Use AMP memory tools for knowledge retrieval, artifact storage, file provenance, and multi-agent coordination. Load this skill when working with persistent memory or shared state.
---
# AMP Core Skill

AMP (Agentic Memory Protocol) provides persistent memory for AI agents across three layers:
- **Episodic Cache**: Rolling window of session blocks (~20 blocks, 1800 tokens each)
- **Durable Artifacts**: Decisions, changesets, notes that persist beyond sessions
- **File Provenance**: Symbol logs, chunks, and audit trails for codebase understanding

## When to Load This Skill

Load this skill when you need to:
- **Remember context** across conversation turns or agent handoffs
- **Store decisions** that affect future work
- **Track code changes** with file-level provenance
- **Coordinate** with other agents on shared resources
- **Search** existing knowledge (symbols, decisions, changesets)

## Quick Navigation

| Need | Reference |
|------|-----------|
| Cache & episodic memory | `references/cache-guide.md` |
| File sync & provenance | `references/file-sync-guide.md` |
| Which tool to use? | `references/tool-map.md` |
| Tool parameters | `references/tool-reference.md` |
| When to create artifacts | `references/artifact-guidelines.md` |

## Tool Categories (13 tools)

### Episodic Memory Cache (3 tools)
- `amp_cache_write` - Write item to current block (auto-closes at ~1800 tokens)
- `amp_cache_compact` - Close current block, open new one (call on conversation compact)
- `amp_cache_read` - Unified read: search blocks, get specific block, or get current context

### File Provenance (2 tools)
- `amp_file_sync` - Sync file across all 3 layers (temporal, vector, graph)
- `amp_filelog_get` - Read file audit trail, symbols, dependencies

### Discovery & Search (4 tools)
- `amp_status` - Health check and analytics
- `amp_list` - Browse objects by type
- `amp_query` - Hybrid search (text + vector + graph)
- `amp_trace` - Follow object relationships

### Writing Artifacts (1 tool)
- `amp_write_artifact` - Create decisions, changesets, notes with graph links

### Focus Tracking (1 tool)
- `amp_focus` - Manage session focus and recorded outputs (list, get, set, complete, end)

### Utility (2 tools)
- `amp_file_content_get` - Retrieve indexed file content from chunks
- `amp_file_path_resolve` - Resolve canonical path for ambiguous/relative file paths

## Core Principle: Two-Phase Retrieval

Cache uses block-based storage with two-phase retrieval:
1. **Search summaries** (~200 tokens each) to find relevant blocks
2. **Fetch full blocks** only when needed

This reduces context from 2000-5000 tokens to 200-400 tokens for initial search.

## REQUIRED RITUALS

These rituals are **MANDATORY** for proper memory continuity.

### Session Start (ALWAYS)

Execute at the start of every new session:

```
amp_cache_read(scope_id: "project:{id}", query: "recent work", include_content: true)
```

### After Context Compact (ALWAYS)

Execute immediately when conversation context is compacted/summarized:

```
amp_cache_compact(scope_id: "project:{id}")
amp_cache_read(scope_id: "project:{id}", query: "recent work", include_content: true)
```

**Why**: Context compaction discards conversation history. Without these rituals, insights from prior work are lost forever.

## Post-Edit Workflow

After any code change, sync the file:

```
amp_file_sync({
  path: "path/to/file.py",      // Flexible: relative or absolute
  action: "edit",                // create | edit | delete
  summary: "Added validation logic for user input"
})
```

Then optionally cache the context:

```
amp_cache_write({
  scope_id: "project:my-project",
  kind: "decision",              // fact | decision | snippet | warning
  content: "Added input validation to prevent XSS attacks",
  importance: 0.8
})
```

## Scope Conventions

```
project:{project_id}  - Shared across agents on same project
task:{task_id}        - Isolated to specific task
agent:{agent_id}      - Private to one agent
```

## Block Lifecycle

1. **Open block** - Accepts new items via `amp_cache_write`
2. **Auto-close** - When token count reaches ~1800
3. **Manual close** - Via `amp_cache_compact` (generates summary + embedding)
4. **Eviction** - Oldest block deleted when >20 blocks exist

## Artifact Philosophy

Artifacts are **permanent long-term memory** for anything useful about the codebase. They exist to serve **future agents** - including yourself after context resets.

Before creating, ask:

> "Would a future agent benefit from knowing this?"

**If yes or maybe → create an artifact.** Artifacts are cheap. Re-learning is expensive.

Artifacts can store **literally anything useful**:
- **User preferences** - "User wants verbose logging", "Prefers functional components"
- **Project conventions** - "snake_case for DB, camelCase for API"
- **Architectural choices** - Decisions made and why
- **Refactoring rationale** - Why code was restructured
- **Dependency choices** - Why one library over another
- **Workarounds** - Temporary hacks and when to remove them
- **Production gotchas** - Non-obvious runtime behavior
- **Historical context** - How things evolved over time

**Don't limit yourself to decisions and changesets.** Use "note" artifacts for anything that doesn't fit elsewhere.

Skip artifacts only when the code is self-explanatory or it's common knowledge.

## Non-Goals

- Do NOT store large raw file contents in cache
- Do NOT store secrets or credentials
- Do NOT create artifacts for trivial changes
- Do NOT use cache for data needing ACID guarantees
