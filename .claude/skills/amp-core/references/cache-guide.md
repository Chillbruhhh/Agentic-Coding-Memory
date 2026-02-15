# Episodic Memory Cache Guide

The cache provides rolling-window episodic memory using block-based storage.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Cache Scope                          │
│  scope_id: "project:my-app"                            │
├─────────────────────────────────────────────────────────┤
│  Block 1 (closed)  │  Block 2 (closed)  │  Block 3     │
│  ┌───────────────┐ │  ┌───────────────┐ │  (open)      │
│  │ Summary: ...  │ │  │ Summary: ...  │ │  ┌─────────┐ │
│  │ Items: [...]  │ │  │ Items: [...]  │ │  │ Items   │ │
│  │ Tokens: 1800  │ │  │ Tokens: 1750  │ │  │ [...]   │ │
│  └───────────────┘ │  └───────────────┘ │  │ 450 tok │ │
│                    │                    │  └─────────┘ │
└─────────────────────────────────────────────────────────┘
```

- **Rolling window**: Max 20 blocks per scope
- **Block size**: ~1800 tokens triggers auto-close
- **Summary**: ~200 tokens generated on close
- **Eviction**: Oldest block deleted when limit exceeded

## Tools

### amp_cache_write

Write an item to the current open block.

```json
{
  "scope_id": "project:my-app",
  "kind": "fact",
  "content": "User authentication uses JWT with 24h expiry",
  "importance": 0.7,
  "file_ref": "src/auth/jwt.ts"
}
```

**Parameters:**
- `scope_id` (required): Scope identifier (e.g., "project:amp")
- `kind` (required): One of "fact", "decision", "snippet", "warning"
- `content` (required): The content to store
- `importance` (optional): 0.0-1.0, default 0.5
- `file_ref` (optional): Associated file path (for snippets)

**Item kinds:**
- `fact` - Objective information learned during work
- `decision` - Choice made with rationale
- `snippet` - Code pattern or reference (include file_ref)
- `warning` - Pitfall or issue to avoid

**Auto-close behavior:**
- When block reaches ~1800 tokens, it auto-closes
- Summary + embedding generated from items
- New block opened for subsequent writes

### amp_cache_compact

Manually close the current block and open a new one.

```json
{
  "scope_id": "project:my-app"
}
```

**When to use:**
- On conversation compact/summarization
- When switching major topics
- Before agent handoff

**What happens:**
1. Current open block marked "closed"
2. Summary generated from block items (~200 tokens)
3. Summary embedding created for semantic search
4. New empty block opened with incremented sequence

### amp_cache_read

Unified tool for reading from the cache - search, get specific block, or get current block.

**Mode 1: Search (summaries only)**
```json
{
  "scope_id": "project:my-app",
  "query": "authentication JWT tokens",
  "limit": 5
}
```

**Mode 2: Search with full content**
```json
{
  "scope_id": "project:my-app",
  "query": "authentication JWT tokens",
  "include_content": true
}
```

**Mode 3: Get specific block**
```json
{
  "scope_id": "project:my-app",
  "block_id": "cache_block:abc123..."
}
```

**Mode 4: Get current open block**
```json
{
  "scope_id": "project:my-app"
}
```

**Parameters:**
- `scope_id` (required): Scope identifier (e.g., "project:amp")
- `query` (optional): Search closed blocks by summary
- `limit` (optional): Max blocks when searching, default 5
- `include_content` (optional): Fetch full content with search, default false
- `block_id` (optional): Get specific block by ID

**Behavior matrix:**
| query | block_id | include_content | Result |
|-------|----------|-----------------|--------|
| ✓ | - | false | Search → summaries only |
| ✓ | - | true | Search → full content |
| - | ✓ | - | Get specific block |
| - | - | - | Get current open block |

**Efficiency:** Use `include_content=true` when you know you need the full content - saves a round-trip vs search then get.

## Workflows

### Session Startup (REQUIRED)

**Always read cache at the start of every session** to restore prior context.

```
# Recommended: One-shot with full content
amp_cache_read(scope_id: "project:X", query: "recent work", include_content: true)
```

This ensures continuity across sessions and prevents re-learning what was already discovered.

### After Context Compact (REQUIRED)

When the conversation context is compacted (summarized), **immediately compact the cache** to preserve learnings from the compacted portion:

```
# 1. Close current block to preserve learnings
amp_cache_compact(scope_id: "project:X")

# 2. Restore context from cache
amp_cache_read(scope_id: "project:X", query: "recent work", include_content: true)
```

**Why this matters**: Context compaction discards conversation history. If you don't compact the cache first, insights from the discarded conversation are lost forever.

### During Work

Write facts, decisions, snippets as you work:

```
amp_cache_write({
  scope_id: "project:X",
  kind: "decision",
  content: "Using Redis for session storage - faster than DB queries",
  importance: 0.8
})
```

### On Conversation Compact

```
amp_cache_compact({ scope_id: "project:X" })
```

This preserves your work context for the next conversation turn.

### Agent Handoff

Before handoff:
```
amp_cache_compact({ scope_id: "project:X" })
```

After handoff:
```
amp_cache_read({ scope_id: "project:X", query: "handoff context", include_content: true })
```

## Best Practices

1. **Be concise** - 1-2 sentences per item
2. **Use appropriate kinds** - Facts vs decisions vs warnings
3. **Set importance** - 0.8+ for critical context
4. **Include file_ref** - For code snippets
5. **Compact on topic switch** - Keeps blocks cohesive
6. **Search before writing** - Avoid duplicating existing context

## Token Budget

| Component | Tokens |
|-----------|--------|
| Block (full) | ~1800 |
| Block summary | ~200 |
| Search result (5 blocks) | ~1000 |
| Single item | ~50-200 |

Two-phase retrieval typically uses 200-400 tokens vs 2000+ for full context.
