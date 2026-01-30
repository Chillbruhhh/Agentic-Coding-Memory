# Cache Policy (Semantic Cache / Unity Layer)

> **This guide covers BEST PRACTICES** - what to cache, token budgets, and multi-agent patterns.
> For **tool mechanics and parameters**, see `cache-guide.md`.

Guidelines for effective use of AMP's token-efficient short-term memory.

---

## Core Concepts

### What is the Cache?

The semantic cache (Unity Layer) provides:
- **Token-efficient memory** - 300-900 tokens vs 2000-5000 for raw context
- **Semantic deduplication** - Automatically merges similar items
- **TTL expiration** - Items expire after 30 minutes of inactivity
- **Scoped isolation** - Memory organized by project/task/agent

### Memory Pack Structure

When you call `amp_cache_read`, you receive a Memory Pack:

```
┌────────────────────────────────────────┐
│ Memory Pack                            │
├────────────────────────────────────────┤
│ summary: "Project context summary"     │
│                                        │
│ facts: [                               │
│   {preview, facts[], importance}       │
│ ]                                      │
│                                        │
│ decisions: [                           │
│   {preview, facts[], importance}       │
│ ]                                      │
│                                        │
│ snippets: [                            │
│   {preview, facts[], importance}       │
│ ]                                      │
│                                        │
│ warnings: [                            │
│   {preview, facts[], importance}       │
│ ]                                      │
│                                        │
│ artifact_pointers: ["id1", "id2"]      │
│ token_count: 487                       │
│ version: 12                            │
│ is_fresh: true                         │
└────────────────────────────────────────┘
```

---

## Scope Naming

### Conventions

| Pattern | Use Case | Sharing |
|---------|----------|---------|
| `project:{id}` | Shared project knowledge | All agents |
| `task:{id}` | Task-specific context | Task agents |
| `agent:{id}` | Private agent state | Single agent |
| `session:{id}` | Conversation-specific | Single session |

### Best Practices

```
# GOOD - Stable, shareable
scope_id: "project:amp"
scope_id: "project:myapp"
scope_id: "task:implement-auth"

# AVOID - Unstable, hard to share
scope_id: "temp-123"
scope_id: "my-stuff"
```

**Use the same scope_id** across agent handoffs to preserve context.

---

## Token Budget Guidelines

### Recommended Budgets

| Scenario | Budget | Rationale |
|----------|--------|-----------|
| Quick check | 300 | Just need key facts |
| Normal work | 600 | Good balance (default) |
| Complex task | 800-1000 | More context needed |
| Maximum | 1500 | Rarely needed |

### Budget Allocation

The cache allocates your budget as:
- ~20% for summary
- ~80% for items (facts, decisions, snippets, warnings)

If budget is 600:
- Summary: up to 120 tokens
- Items: up to 480 tokens

---

## Writing Effective Cache Items

### Item Kinds

| Kind | Use For | Example |
|------|---------|---------|
| `fact` | Learned information | "Auth uses JWT with RS256" |
| `decision` | Choices made | "Chose Redis for sessions" |
| `snippet` | Useful patterns | "Error handler wraps all routes" |
| `warning` | Gotchas, cautions | "DB timeout is only 5s" |

### Item Structure

```json
{
  "kind": "fact",
  "content": "Cache dedup uses 0.92 cosine similarity threshold",
  "importance": 0.7,
  "file_ref": "src/cache/dedup.rs"
}
```

### Writing Guidelines

**Content**:
- Keep to 1-2 sentences
- Make it scannable
- Include key terms


- 1-3 atomic facts per item
- Each fact stands alone
- No redundancy with preview

**Importance**:
- 0.0-0.3: Low priority, drop first
- 0.4-0.6: Normal priority (default 0.5)
- 0.7-0.8: High priority, keep longer
- 0.9-1.0: Critical, preserve

### Examples

```json
// GOOD - Compact, actionable
{
  "kind": "fact",
  "content": "Cache dedup uses 0.92 cosine similarity threshold",
  "importance": 0.7
}

// BAD - Too verbose
{
  "kind": "fact",
  "content": "I discovered that the caching system in this application uses a cosine similarity threshold of 0.92 to determine whether two items should be considered duplicates, which means that if two items have vectors that are more than 92% similar...",
  "importance": 0.5
}
```

---

## Semantic Deduplication

### How It Works

1. New item submitted via `amp_cache_write`
2. Embedding generated from preview text
3. Search for existing items with >0.92 similarity
4. If found: merge (boost importance, update timestamp)
5. If not found: insert new item

### Implications

- **Write freely** - Duplicates handled automatically
- **Importance accumulates** - Repeated topics get boosted
- **Access recency** - Merged items stay fresh longer

---

## TTL and Expiration

### Default Behavior

- **Item TTL**: 30 minutes from last access
- **Frame TTL**: 30 minutes from last update
- **Freshness threshold**: 5 minutes

### Freshness

Memory pack includes `is_fresh: true/false`:
- `true`: Cache updated within 5 minutes
- `false`: Cache may be stale, consider refreshing

### Garbage Collection

Expired items automatically cleaned up. To force cleanup:
```
POST /v1/cache/gc
```

---

## Content Rules

### DO Store

- Short summaries and facts
- Decisions and rationale
- Useful patterns (1-3 lines)
- Warnings and gotchas
- Key configuration values

### DON'T Store

- Raw file contents
- Large code blocks (>10 lines)
- Secrets or credentials
- Full error stack traces
- Binary data

### Size Limits

- Content: Keep under 200 characters
- Facts array: 1-3 items
- Total item: Estimate 50-100 tokens

---

## Multi-Agent Patterns

### Shared Project Memory

```
Agent A writes:
  amp_cache_write(scope_id: "project:amp", items: [...])

Agent B reads:
  amp_cache_read(scope_id: "project:amp")
  → Gets items from Agent A
```

### Task Isolation

```
Agent A on task 1:
  scope_id: "task:feature-1"

Agent B on task 2:
  scope_id: "task:feature-2"

→ Isolated, no cross-contamination
```

### Handoff Pattern

```
Agent A ending:
1. amp_cache_write(scope_id: "project:X", items: [
     {kind: "fact", preview: "Completed steps 1-3"},
     {kind: "warning", preview: "Step 4 has edge case Y"},
     {kind: "decision", preview: "Chose approach Z for step 5"}
   ])

Agent B starting:
1. amp_cache_read(scope_id: "project:X")
   → Receives Agent A's context
```

---

## Troubleshooting

### Empty cache response

**Causes**:
- New scope with no items
- All items expired
- Wrong scope_id
- Search query too specific (no matches)

**Solution**: Use `list_all: true` to see what blocks actually exist:
```
amp_cache_read(scope_id: "project:X", list_all: true)
```
This shows all blocks regardless of query, helping you verify if data exists.

### Search returns no results

**Causes**:
- Query doesn't match any block summaries
- Blocks exist but summaries don't contain your terms
- Open block has items but query doesn't match content

**Solution**: 
1. Use `list_all: true` to see all available blocks
2. Try broader search terms
3. Use `include_open: true` with your search to check open block

### Items not appearing

**Causes**:
- Merged with similar existing item
- Token budget too small
- Low importance, not selected

**Solution**: Increase token_budget, check importance values

### Stale data (is_fresh: false)

**Causes**:
- No writes in 5+ minutes
- Working with old context

**Solution**: Write new items to refresh, or accept staleness

### High token count

**Causes**:
- Too many items accumulated
- Verbose previews

**Solution**: Let TTL expire old items, write more concise items
