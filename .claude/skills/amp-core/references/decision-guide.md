# AMP Decision Guide

Flowcharts for choosing the right AMP tool for your situation.

> **For artifact creation guidance**, see `artifact-guidelines.md` - it covers when and why to create decisions, notes, and changesets.

---

## Required Cache Rituals

### On Session Start (ALWAYS)
```
amp_cache_read(scope_id: "project:X", query: "recent work", include_content: true)
```

### After Context Compact (ALWAYS)
```
amp_cache_compact(scope_id: "project:X")  # Preserve learnings first
amp_cache_read(scope_id: "project:X", query: "recent work", include_content: true)
```

---

## Which Tool Should I Use?

### Need to remember something?

```
Is it short-term working memory?
├─ YES: amp_cache_write
│   └─ Use for: facts, decisions, snippets, warnings during work
│
└─ NO: Is it a significant decision?
    ├─ YES: amp_write_artifact (type: "decision")
    │   └─ Use for: architectural choices, trade-off decisions
    │
    └─ NO: Is it documenting completed work?
        ├─ YES: amp_write_artifact (type: "changeset")
        │   └─ Use for: code changes, feature completion
        │
        └─ NO: amp_write_artifact (type: "note")
            └─ Use for: insights, todos, references
```

### Need to retrieve context?

```
Do you need compact working memory?
├─ YES: amp_cache_read
│   └─ Use for: session start, agent handoff, context refresh
│
└─ NO: Do you have a specific search query?
    ├─ YES: amp_query
    │   └─ Use for: finding specific knowledge, semantic search
    │
    └─ NO: Do you want context for a goal?
        ├─ YES: amp_context
        │   └─ Use for: starting new task, need relevant background
        │
        └─ NO: Just browsing?
            └─ amp_list
                └─ Use for: exploring what exists
```

### Working with files?

```
About to modify a file?
├─ YES: amp_filelog_get first
│   └─ Then: make changes, then amp_filelog_update
│
└─ NO: Need file content from memory?
    ├─ YES: amp_file_content_get
    │
    └─ NO: Path resolution issues?
        └─ amp_file_path_resolve
```

### Multi-agent scenario?

```
Multiple agents might touch same resource?
├─ YES: amp_lease_acquire before work
│   └─ Then: do work, then amp_lease_release
│
└─ NO: Proceed without coordination
```

---

## Cache vs Artifact: Which to Use?

| Characteristic | Cache (amp_cache_*) | Artifact (amp_write_*) |
|---------------|---------------------|------------------------|
| Lifetime | Short-term (30 min TTL) | Permanent |
| Purpose | Working memory | Historical record |
| Size | Compact (1-3 facts) | Detailed |
| Dedup | Automatic semantic | None |
| Query method | amp_cache_read | amp_query, amp_list |

**Use Cache when**:
- Learning something during work
- Need to remember across turns
- Agent handoff context
- Temporary insights

**Use Artifact when**:
- Recording decisions for future reference
- Documenting completed work
- Creating audit trail
- Knowledge that should persist

---

## Query Mode Selection

```
What are you searching for?

"I know roughly what I want"
└─ mode: "hybrid" (default)
   └─ Combines text matching + semantic similarity

"I have exact keywords"
└─ mode: "text"
   └─ Full-text search only

"I want semantically similar items"
└─ mode: "vector"
   └─ Embedding similarity only

"I want to explore connections from a known object"
└─ mode: "graph"
   └─ Requires: graph_options.start_nodes
```

---

## Token Budget Guidelines

| Situation | Recommended Budget |
|-----------|-------------------|
| Quick context check | 300-400 |
| Normal task start | 600 (default) |
| Complex task | 800-1000 |
| Full context needed | 1000-1500 |
| Never exceed | 2000 |

**Rule of thumb**: Start with 600, increase only if response says "truncated" or missing expected info.

---

## Scope Naming Conventions

```
project:{id}
├─ Shared across all agents on project
├─ Example: project:amp, project:myapp
└─ Use for: cross-agent memory, project facts

task:{id}
├─ Isolated to specific task
├─ Example: task:fix-auth-bug, task:add-caching
└─ Use for: task-specific context, temporary

agent:{id}
├─ Private to one agent
├─ Example: agent:claude-1, agent:research-bot
└─ Use for: agent-specific state (rare)

session:{id}
├─ Single conversation
├─ Example: session:abc123
└─ Use for: very short-term (rare)
```

**Best practice**: Use `project:{id}` for most cases. It enables knowledge sharing.

---

## Common Decision Patterns

### "Should I cache this?"

```
Is it:
- A fact I learned? → YES, cache it (kind: "fact")
- A decision I made? → YES, cache it (kind: "decision")
- A useful code pattern? → YES, cache it (kind: "snippet")
- A gotcha or warning? → YES, cache it (kind: "warning")
- Raw data or logs? → NO, don't cache
- Large code blocks? → NO, summarize first then cache
```

### "Should I create a decision record?"

> See `artifact-guidelines.md` for comprehensive guidance.

```
Does this decision:
- Involve choosing between alternatives? → YES, record it
- Have trade-offs worth remembering? → YES, record it
- Might a future agent question "why this approach"? → YES, record it
- Is it obvious from the code? → NO, skip
- Is it common knowledge? → NO, skip
```

### "Should I track this as a run?"

```
Is this:
- A bounded task with clear goal? → YES, track it
- Multiple steps to coordinate? → YES, track it
- Just answering a question? → NO, skip
- Quick one-off action? → NO, skip
```

---

## Error Handling Guide

| Error | Meaning | Action |
|-------|---------|--------|
| 409 Conflict | Resource leased | Wait and retry, or work on different resource |
| 404 Not Found | Object doesn't exist | Check ID, might be deleted |
| 500 Internal Error | Server issue | Retry with backoff |
| Timeout | Query too slow | Reduce limit, narrow filters |
| Empty results | Nothing matches | Broaden query, check scope |

---

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────┐
│ CACHE                                                    │
│   amp_cache_read(scope_id, query?, include_content?)    │
│   amp_cache_read(scope_id, block_id)  # get specific    │
│   amp_cache_write(scope_id, kind, content)              │
│   amp_cache_compact(scope_id)                           │
├─────────────────────────────────────────────────────────┤
│ SEARCH                                                   │
│   amp_query(query, mode="hybrid", limit=5)              │
│   amp_context(goal, scope, include_decisions=false)     │
│   amp_list(type, limit=10)                              │
│   amp_trace(object_id, depth=2)                         │
├─────────────────────────────────────────────────────────┤
│ WRITE                                                    │
│   amp_write_artifact(type, title, ...)                  │
│     type: "decision" | "changeset" | "note" | "filelog" │
├─────────────────────────────────────────────────────────┤
│ FILES                                                    │
│   amp_filelog_get(path)                                 │
│   amp_filelog_update(path, summary)                     │
├─────────────────────────────────────────────────────────┤
│ RUNS                                                     │
│   amp_run_start(goal, repo_id, agent_name)              │
│   amp_run_end(run_id, status, outputs, summary)         │
├─────────────────────────────────────────────────────────┤
│ COORDINATION                                             │
│   amp_lease_acquire(resource, duration, agent_id)       │
│   amp_lease_release(lease_id)                           │
└─────────────────────────────────────────────────────────┘
```
