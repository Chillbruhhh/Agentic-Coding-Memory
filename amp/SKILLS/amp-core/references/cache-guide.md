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

### amp_cache_search

Search closed blocks by summary (two-phase retrieval).

```json
{
  "scope_id": "project:my-app",
  "query": "authentication JWT tokens",
  "limit": 5
}
```

**Returns:** Block IDs with summaries and relevance scores.

**Parameters:**
- `scope_id` (required): Scope to search within
- `query` (required): Search query
- `limit` (optional): Max results, default 5

**Two-phase retrieval pattern:**
1. Search returns summaries (~200 tokens each)
2. Evaluate relevance from summaries
3. Fetch full blocks only for high-relevance matches

### amp_cache_get

Get a specific block by ID or current open block.

```json
{
  "scope_id": "project:my-app",
  "block_id": "cache_block:abc123..."
}
```

**Parameters:**
- `scope_id` (required): Scope identifier
- `block_id` (optional): Specific block to retrieve

If `block_id` omitted, falls back to legacy memory pack behavior.

## Workflows

### Session Startup

```
1. amp_cache_search(scope: "project:X", query: "recent work context")
2. Review returned summaries for relevance
3. If high-relevance block found:
     amp_cache_get(block_id: "...")
4. Otherwise: proceed fresh
```

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
amp_cache_search({ scope_id: "project:X", query: "handoff context" })
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
