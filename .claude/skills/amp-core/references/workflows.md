# AMP Workflows

Step-by-step patterns for common agent tasks using AMP tools.

---

## Workflow 1: Starting a New Task

Use when beginning work that needs historical context.

```
1. amp_cache_read          -> Get memory pack for scope
2. amp_context            -> Get relevant symbols/decisions for goal
3. [Do work]
4. amp_cache_write        -> Store new insights as you go
5. amp_write_artifact     -> Record completed changes (type: "changeset")
```

**Example sequence**:
```
amp_cache_read(scope_id: "project:amp", token_budget: 600)
  -> Returns: summary, facts about prior work, relevant decisions

amp_context(goal: "add user authentication", scope: "project:amp", include_decisions: true)
  -> Returns: related symbols, past auth decisions, relevant files

[Implement authentication]

amp_cache_write(scope_id: "project:amp", items: [
  {kind: "decision", preview: "Using JWT for stateless auth", importance: 0.8}
])

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
2. amp_run_end            -> Close current run with summary
3. [New agent starts]
4. amp_cache_read          -> New agent retrieves context
5. amp_run_start          -> New agent begins tracking
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
2. amp_lease_acquire      -> Lock if multi-agent (optional)
3. [Make changes]
4. amp_filelog_update     -> Record what changed
5. amp_lease_release      -> Release lock (if acquired)
6. amp_cache_write        -> Cache important patterns discovered
```

**Example**:
```
amp_filelog_get(path: "src/services/cache.rs")
  -> Returns: functions (get_pack, write_items), dependencies, prior changes

[Modify cache.rs to add TTL support]

amp_filelog_update(
  path: "src/services/cache.rs",
  summary: "Added configurable TTL with default 30 minutes"
)

amp_cache_write(scope_id: "project:amp", items: [
  {kind: "fact", preview: "Cache TTL defaults to 30 minutes", importance: 0.6}
])
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

## Workflow 5: Multi-Agent Coordination

Use when multiple agents work on same codebase.

```
Agent A:
1. amp_lease_acquire(resource: "file:auth.rs", agent_id: "agent-a")
2. [Work on auth.rs]
3. amp_write_artifact     -> Document changes (type: "changeset")
4. amp_lease_release      -> Let others access

Agent B (concurrent):
1. amp_lease_acquire(resource: "file:auth.rs", agent_id: "agent-b")
   -> Returns CONFLICT if Agent A holds lease
2. [Wait or work on different file]
3. Retry after Agent A releases
```

**Lease best practices**:
- Acquire narrowly (specific file, not whole directory)
- Release promptly when done
- Use reasonable durations (300 seconds typical)

---

## Workflow 6: Architectural Decision

Use when making significant technical choices.

```
1. amp_query              -> Find related past decisions
2. amp_context            -> Get context for decision area
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
3. amp_list(type: "decision", limit: 5)-> Recent decisions (optional)
4. amp_run_start                       -> Begin tracking (optional)
```

This gives you:
- Prior context restored (prevents re-learning)
- Awareness of recent decisions
- Continuity across sessions

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
3. amp_run_end            -> Close run with summary
```

**What to cache at session end**:
- Decisions made
- Gotchas discovered
- Patterns found useful
- Next steps identified

---

## Anti-Patterns to Avoid

### Don't: Cache raw file contents
```
# BAD
amp_cache_write(items: [{kind: "snippet", preview: "[500 lines of code]"}])

# GOOD
amp_cache_write(items: [{kind: "snippet", preview: "Auth middleware pattern: wrap handler with validate_jwt()"}])
```

### Don't: Create decisions for minor choices
```
# BAD - too granular
amp_write_artifact(type: "decision", title: "Use camelCase for variable names")

# GOOD - significant impact
amp_write_artifact(type: "decision", title: "Adopt TypeScript strict mode project-wide")
```

### Don't: Skip cache writes
```
# BAD - knowledge lost at context limit
[Learn important pattern, don't write to cache]

# GOOD - preserve for future
amp_cache_write(items: [{kind: "fact", preview: "Pattern: X works well for Y"}])
```

### Don't: Use huge token budgets
```
# BAD - wasteful
amp_cache_read(token_budget: 5000)

# GOOD - efficient
amp_cache_read(token_budget: 600)  # Increase to 800-1000 only if needed
```
