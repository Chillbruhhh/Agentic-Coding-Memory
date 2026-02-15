---
name: amp-core
description: Use AMP memory tools for context retrieval, artifact storage, file provenance, and multi-agent coordination. Load this skill when working with persistent memory or shared state.
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
| Which tool to use? | `references/tool-reference.md` |
| When to create artifacts | `references/artifact-guidelines.md` |

## Tool Categories (16 tools)

### Episodic Memory Cache (3 tools)
- `amp_cache_write` - Write item to current block (auto-closes at ~1800 tokens)
- `amp_cache_compact` - Close current block, open new one (call on conversation compact)
- `amp_cache_read` - Unified read: search blocks, get specific block, or get current context

### File Provenance (2 tools)
- `amp_file_sync` - Sync file across all 3 layers (temporal, vector, graph)
- `amp_filelog_get` - Read file audit trail, symbols, dependencies

### Discovery & Search (5 tools)
- `amp_status` - Health check and analytics
- `amp_list` - Browse objects by type
- `amp_context` - High-signal context for a goal
- `amp_query` - Hybrid search (text + vector + graph)
- `amp_trace` - Follow object relationships

### Writing Artifacts (1 tool)
- `amp_write_artifact` - Create decisions, changesets, notes with graph links

### Run Tracking (2 tools)
- `amp_run_start` - Begin agent execution tracking
- `amp_run_end` - Complete execution with outputs

### Coordination (2 tools)
- `amp_lease_acquire` - Lock shared resources
- `amp_lease_release` - Release locks

### Utility (1 tool)
- `amp_file_content_get` - Retrieve indexed file content from chunks

## Core Principle: Two-Phase Retrieval

Cache uses block-based storage with two-phase retrieval:
1. **Search summaries** (~200 tokens each) to find relevant blocks
2. **Fetch full blocks** only when needed

This reduces context from 2000-5000 tokens to 200-400 tokens for initial search.

## Startup Workflow

**IMPORTANT**: Always read cache at the start of every new session to restore context.

```
# Session start - restore context
amp_cache_read(scope_id: "project:{id}", query: "recent work", include_content: true)
```

If context was just compacted (conversation summary happened), also compact the cache to preserve what was learned:

```
# After context compact - preserve learnings
amp_cache_compact(scope_id: "project:{id}")
amp_cache_read(scope_id: "project:{id}", query: "recent work", include_content: true)
```

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

Artifacts exist to serve **future agents**. Before creating, ask:

> "Would a future agent benefit from knowing this?"

Create artifacts that capture:
- **Preferences** - Choices you made and why
- **Discoveries** - Non-obvious things you learned
- **Effective operations** - Patterns that worked well

Skip artifacts when the code is self-explanatory or it's common knowledge.

## Non-Goals

- Do NOT store large raw file contents in cache
- Do NOT store secrets or credentials
- Do NOT create artifacts for trivial changes
- Do NOT use cache for data needing ACID guarantees
