# AMP Workflows

Step-by-step patterns for common agent tasks using AMP tools.

---

## Workflow 1: Starting a New Task

Use when beginning work that needs historical context.

```
1. amp_cache_read          -> Get memory pack for scope
2. [Do work]
3. amp_cache_write        -> Store new insights as you go
4. amp_write_artifact     -> Record completed changes (type: "changeset")
```

**Example sequence**:
```
amp_cache_read(scope_id: "project:amp", token_budget: 600)
  -> Returns: summary, facts about prior work, relevant decisions


[Implement authentication]

amp_cache_write(scope_id: "project:amp", kind: "decision", content: "Using JWT for stateless auth", importance: 0.8)

amp_write_artifact(
  type: "changeset",
  title: "Added JWT authentication",
  description: "Added JWT authentication",
  files_changed: ["src/auth.rs", "src/middleware.rs"],
  diff_summary: "Added JWT validation middleware and token generation"
)
```

---

## Artifact Usage (When/Why/How)

Use `amp_write_artifact` when information should persist across sessions and be discoverable by other agents.
Good fits: changesets, architectural decisions, plans, and operational notes.

**Example (decision artifact):**
```
amp_write_artifact(
  type: "decision",
  title: "Use WebSockets for real-time updates",
  context: "Need bidirectional communication for live collaboration.",
  decision: "WebSockets chosen for full-duplex communication and lower latency.",
  consequences: "Requires sticky sessions or message broker.",
  alternatives: ["Long polling", "SSE"]
)
```

---

## Query Usage (When/Why/How)

Use `amp_query` to retrieve any stored data (artifacts, notes, file logs, symbols). Use `amp_list` to quickly browse by type.

**Example (retrieve artifacts):**
```
amp_query(
  query: "changeset indexer hierarchy",
  mode: "hybrid",
  limit: 5
)
```

**Example (list decisions):**
```
amp_list(type: "decision", limit: 5)
```

---

## Workflow 2: Agent Handoff

Use when transferring context to another agent.

```
1. amp_cache_write        -> Store critical context before handoff
2. amp_focus (complete)   -> Record what was completed
3. [New agent starts]
4. amp_cache_read         -> New agent retrieves context
5. amp_focus (set)        -> New agent sets current focus
```

**Key principle**: Write compact, actionable facts before handoff. The receiving agent should understand:
- What was accomplished
- What remains to do
- Critical decisions made
- Gotchas discovered

---

## Workflow 3: File Modification

Use when changing code files.

```
1. amp_filelog_get        -> Understand file structure first
2. [Make changes]
3. amp_file_sync          -> Record what changed
4. amp_cache_write        -> Cache important patterns discovered
```

**Example**:
```
amp_filelog_get(path: "src/services/cache.rs")
  -> Returns: functions (get_pack, write_items), dependencies, prior changes

[Modify cache.rs to add TTL support]

amp_file_sync(
  path: "src/services/cache.rs",
  action: "edit",
  summary: "Added configurable TTL with default 30 minutes"
)

amp_cache_write(scope_id: "project:amp", kind: "fact", content: "Cache TTL defaults to 30 minutes", importance: 0.6)
```

---

## Workflow 4: Research/Exploration

Use when searching for existing knowledge.

```
1. amp_status             -> Check what's indexed
2. amp_list               -> Browse by type
3. amp_query              -> Search with hybrid retrieval
4. amp_trace              -> Follow relationships if needed
5. amp_cache_write        -> Store findings for later
```

**Search strategy**:
- Start with `amp_list(type: "decision")` to see existing decisions
- Use `amp_query` for semantic search when you know what you're looking for
- Use `amp_trace` to explore connections from a known object

---

## Workflow 5: Multi-Agent Awareness

Use when multiple agents work on the same codebase.

```
1. amp_focus(action: "list") -> See active sessions and current focus
2. amp_focus(action: "set")  -> Record your current focus
3. amp_focus(action: "complete") -> Log completed work
```

---

## Workflow 6: Architectural Decision

Use when making significant technical choices.

```
1. amp_query              -> Find related past decisions
3. [Analyze and decide]
4. amp_write_artifact     -> Record the ADR (type: "decision")
5. amp_cache_write        -> Add to short-term memory
```

**Decision record structure**:
```
amp_write_artifact(
  type: "decision",
  title: "Use WebSockets for real-time updates",
  context: "Need bidirectional communication for live collaboration. Options: polling, SSE, WebSockets.",
  decision: "WebSockets chosen for full-duplex communication and lower latency",
  consequences: "Requires sticky sessions or message broker. More complex deployment.",
  alternatives: ["Long polling (simpler but higher latency)", "SSE (one-way only)"]
)
```

---

## Workflow 7: Session Start Ritual (REQUIRED)

**Always execute at the start of every session.**

```
1. amp_cache_read(scope_id: "project:X", query: "recent work", include_content: true)
   -> Restore context from prior sessions
2. amp_status                          -> Verify connectivity (optional)
3. amp_list(type: "decision", limit: 5) -> Recent decisions (optional)
4. amp_focus(action: "set", title: "Current task", plan: ["Step 1"]) -> Record focus
```

This gives you:
- Prior context restored (prevents re-learning)
- Awareness of recent decisions
- Clear active focus in the session view

---

## Workflow 7b: After Context Compact (REQUIRED)

**Execute immediately when conversation context is compacted/summarized.**

```
1. amp_cache_compact(scope_id: "project:X")
   -> Close current block, preserve learnings from compacted conversation
2. amp_cache_read(scope_id: "project:X", query: "recent work", include_content: true)
   -> Restore context from cache
```

**Why this matters**: When context is compacted, conversation history is discarded. Without compacting the cache first, all insights from that conversation are lost forever.

---

## Workflow 8: Session End Ritual

Recommended end for any agent session.

```
1. amp_cache_write        -> Store important learnings
2. amp_write_artifact     -> Document any code changes (type: "changeset")
3. amp_focus(action: "complete") -> Record completed focus
4. amp_focus(action: "end") -> Mark session completed
```

**What to cache at session end**:
- Decisions made
- Gotchas discovered
- Patterns found useful
- Next steps identified
