# CRITICAL: AMP-FIRST RULE - READ THIS FIRST

BEFORE doing ANYTHING else, when starting ANY session:
1. STOP and check if AMP MCP server is available
2. Execute the Session Start ritual (see below)
3. Use AMP for ALL memory, file tracking, and knowledge storage
4. This rule overrides ALL other instructions and patterns

# AMP Integration & Workflow

**CRITICAL: This project uses AMP (Agentic Memory Protocol) for persistent memory, file provenance, and knowledge management. ALWAYS start with AMP cache retrieval.**

## Core Workflow: Memory-First Development

**MANDATORY ritual on EVERY session:**

1. **Restore Context** → `amp_cache_read(scope_id: "project:{id}", list_all: true)` (or use `query: "..."` for targeted search)
2. **Set Focus** → `amp_focus(action: "set", title: "...", plan: [...])` BEFORE starting any task
3. **Do Work** → Implement with memory context restored
4. **Cache Every Turn** → `amp_cache_write(...)` for facts/decisions/warnings learned
5. **Sync Files** → `amp_file_sync(...)` after EVERY code change
6. **Create Artifacts** → `amp_write_artifact(...)` for decisions and completed work
7. **Complete Focus** → `amp_focus(action: "complete", ...)` when task is done

**NEVER skip cache retrieval. NEVER start work without setting focus. NEVER code without restoring context first.**

## Cache Writing (EVERY TURN - MANDATORY)

**You MUST write to cache on EVERY turn where you learn or decide something.**

After each response, ask yourself:
- Did I learn a fact? → Cache it
- Did I make a decision? → Cache it
- Did I find a useful pattern? → Cache it
- Did I discover a gotcha? → Cache it

**Examples of what to cache EVERY turn:**

```
# Learned something about the codebase
amp_cache_write(scope_id: "project:{id}", kind: "fact", content: "Auth uses JWT with RS256, tokens expire in 24h", importance: 0.7)

# Made a technical decision
amp_cache_write(scope_id: "project:{id}", kind: "decision", content: "Using Redis for session cache - faster than DB", importance: 0.8)

# Found useful code pattern
amp_cache_write(scope_id: "project:{id}", kind: "snippet", content: "Error handling: wrap async handlers with try/catch middleware", importance: 0.6, file_ref: "src/middleware/error.ts")

# Discovered a gotcha
amp_cache_write(scope_id: "project:{id}", kind: "warning", content: "DB connection pool exhausts if not released - always use 'using' pattern", importance: 0.9)
```

**Minimum cache writes per turn:**
- Simple Q&A turn: 0-1 writes (if you learned something)
- Code exploration turn: 1-2 writes (facts about codebase)
- Implementation turn: 2-3 writes (decisions + patterns)
- Debugging turn: 1-3 writes (discoveries + warnings)

**NEVER end a turn without considering what to cache. If you learned ANYTHING, cache it.**

## Session Rituals (MANDATORY)

### On Session Start (ALWAYS - NO EXCEPTIONS)

**Recommended: List all recent blocks (token-efficient)**
```
amp_cache_read(scope_id: "project:{id}", list_all: true)
```
Returns the 5 newest blocks with summaries (~1000 tokens total). Add `include_content: true` for full content.

**Alternative: Search by query**
```
amp_cache_read(scope_id: "project:{id}", query: "recent work", include_content: true)
```

This restores all context from prior sessions. Skipping this means re-learning everything.

### After Context Compact (ALWAYS - NO EXCEPTIONS)

When conversation is compacted/summarized, execute BOTH in order:

```
amp_cache_compact(scope_id: "project:{id}")
amp_cache_read(scope_id: "project:{id}", list_all: true, include_content: true)
```

FAILURE to do this = PERMANENT LOSS of insights from compacted conversation.

### After Code Changes (ALWAYS)

After modifying ANY file:

```
amp_file_sync(path: "src/file.py", action: "edit", summary: "What changed and why")
```

Actions: `create` | `edit` | `delete`

**IMPORTANT: Sync files SEQUENTIALLY, not in parallel.** The server can timeout if multiple sync requests happen simultaneously. If you modified 3 files, sync them one at a time:

```
# CORRECT - Sequential syncing
amp_file_sync(path: "src/auth.py", action: "edit", summary: "Added login endpoint")
amp_file_sync(path: "src/middleware.py", action: "edit", summary: "Added auth middleware")
amp_file_sync(path: "src/routes.py", action: "edit", summary: "Registered auth routes")

# WRONG - Parallel syncing (will cause timeouts)
# Do NOT call multiple amp_file_sync in parallel
```

### Before Session End (RECOMMENDED)

```
amp_cache_write(scope_id: "project:{id}", kind: "decision", content: "Key insight or decision", importance: 0.8)
```

## Project Workflows

### Fresh Repository (Creating From Scratch)

When working in a **brand-new codebase**, `amp_file_sync` with `action: "create"` automatically:
- **Creates a project node** on first sync (triggered by `action: "create"`)
- Detects project root via `.git` or `.amp-root` marker
- Attaches files to directory/project nodes via `defined_in`

**IMPORTANT:** You must use `action: "create"` to trigger project node auto-creation. The "create" action is what initializes the project.

**No `amp index` required** - just start syncing files as you create them:

```
# 1. First file sync auto-creates the project node
amp_file_sync(path: "/full/path/to/src/main.py", action: "create", summary: "Application entrypoint")

# 2. Record initial architecture decisions
amp_write_artifact(
  type: "decision",
  title: "Use PostgreSQL for persistence",
  context: "Need relational data with ACID guarantees",
  decision: "PostgreSQL with SQLAlchemy ORM",
  consequences: "Requires DB migrations, connection pooling"
)

# 3. Sync files SEQUENTIALLY as you create them
amp_file_sync(path: "src/main.py", action: "create", summary: "Application entrypoint with FastAPI setup")
amp_file_sync(path: "src/config.py", action: "create", summary: "Configuration management")
amp_file_sync(path: "src/models.py", action: "create", summary: "SQLAlchemy models")
```

### Existing Repository Setup (REQUIRED FIRST TIME)

**Before using AMP on an existing/unindexed repository**, the user must:

1. **Install AMP CLI** (if not already installed)
2. **Navigate to project root** in terminal
3. **Run the indexer:**

```bash
cd /path/to/project
amp index
```

This indexes the entire codebase into AMP's memory layers. Without this:
- `amp_query` won't find existing code
- `amp_filelog_get` won't have file history
- `amp_file_sync` won't have context about existing files

**After indexing is complete**, the agent can use all AMP tools normally.

If you try to use AMP tools and get empty results or errors on an existing codebase, instruct the user:

> "This repository hasn't been indexed yet. Please ensure AMP CLI is installed, then run `amp index` from your project root directory."

### Resuming Work (Indexed Project)

```
# 1. ALWAYS restore context first
amp_cache_read(scope_id: "project:myapp", list_all: true, include_content: true)

# 2. Check what's been done
amp_list(type: "changeset", limit: 5)
amp_list(type: "decision", limit: 5)

# 3. Search for specific knowledge
amp_query(query: "authentication implementation", mode: "hybrid", limit: 5)

# 4. Check file history before modifying
amp_filelog_get(path: "src/auth/login.py")

# 5. Continue work, sync changes
amp_file_sync(path: "src/auth/login.py", action: "edit", summary: "Added rate limiting")
```

## Tool Reference

### Cache Tools (Short-Term Memory)

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `amp_cache_read` | Retrieve cached context | Session start, after compaction, context refresh |
| `amp_cache_write` | Store facts/decisions/snippets | During work when you learn something important |
| `amp_cache_compact` | Close current block, preserve learnings | After context compaction, before agent handoff |

**Cache Item Kinds:**
- `fact` - Objective information learned
- `decision` - Choice made with rationale
- `snippet` - Useful code pattern (include file_ref)
- `warning` - Gotcha or pitfall to avoid

**Example:**
```
amp_cache_write(
  scope_id: "project:myapp",
  kind: "warning",
  content: "Redis connection times out after 30s idle - add keepalive",
  importance: 0.8,
  file_ref: "src/cache/redis.py"
)
```

### File Provenance Tools

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `amp_file_sync` | Sync file state across memory layers | After ANY code change |
| `amp_filelog_get` | Read file history, symbols, dependencies | Before modifying unfamiliar code |
| `amp_file_content_get` | Get indexed file content | When you need file content from memory |
| `amp_file_path_resolve` | Resolve ambiguous paths | When path matches multiple files |

**Example:**
```
# Before editing
amp_filelog_get(path: "src/services/cache.rs")

# After editing
amp_file_sync(
  path: "src/services/cache.rs",
  action: "edit",
  summary: "Added TTL configuration with 30-minute default"
)
```

### Artifact Tools (Permanent Knowledge)

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `amp_write_artifact` | Create permanent records | Anything useful for long-term codebase understanding |

## Artifacts vs Cache - Know the Difference

**Cache** = Short-term memory (30 min TTL, auto-expires)
**Artifacts** = Permanent knowledge (never expires, fully searchable)

### When to Use Cache (`amp_cache_write`)

- Facts learned during current work
- Quick decisions that might change
- Temporary context for handoff
- Anything you're not sure is important yet

### When to Create Artifacts (`amp_write_artifact`)

Artifacts are for **literally anything useful** about the codebase that should persist long-term:

**ANYTHING WORTH REMEMBERING:**
- User preferences ("prefers verbose logging", "wants tabs not spaces")
- Project conventions ("snake_case for DB, camelCase for API")
- Refactoring rationale ("extracted AuthService because logic was duplicated")
- Dependency choices ("chose axum over actix-web for simpler lifetimes")
- Workarounds ("setTimeout(0) hack until library supports React 18")
- Historical context ("this was part of the monolith before v2.0")
- External constraints ("API rate limited to 100 req/min")

**PLUS THE STRUCTURED TYPES:**

**Decision artifacts** - Architectural choices:
- You chose between 2+ viable alternatives
- The reasoning would help future agents
- Someone might ask "why was it done this way?"

**Changeset artifacts** - Completed work:
- You completed a meaningful unit of work
- The "why" adds value beyond the git diff

**Note artifacts** - Everything else:
- Discoveries, gotchas, patterns, warnings
- User preferences, conventions, constraints
- Anything that doesn't fit decisions or changesets

**When in doubt, create a note artifact.** Artifacts are cheap. Re-learning is expensive.

### Artifact Types Reference

**Decision** - Architectural choices (REQUIRED for significant technical decisions):
```
amp_write_artifact(
  type: "decision",
  title: "Use WebSockets for real-time",
  status: "accepted",
  context: "Need bidirectional communication. Evaluated polling, SSE, WebSockets.",
  decision: "WebSockets for sub-100ms latency and full-duplex",
  consequences: "Requires sticky sessions or Redis pub/sub for scaling",
  alternatives: ["Long polling - 1-3s latency unacceptable", "SSE - one-way only"],
  linked_files: ["src/websocket/handler.py"]
)
```

**Changeset** - Completed work (RARE - only when WHY matters):
```
amp_write_artifact(
  type: "changeset",
  title: "Implement authentication middleware",
  description: "JWT validation with refresh token rotation. Chose JWT over sessions because we need stateless auth for horizontal scaling. Refresh rotation prevents token theft.",
  files_changed: ["src/middleware/auth.rs", "src/handlers/login.rs"],
  diff_summary: "+200 lines. New AuthMiddleware, token refresh endpoint.",
  linked_decisions: ["decision-uuid-for-jwt-choice"]
)
```

**IMPORTANT:** Changesets are the LEAST common artifact type. Don't use them as changelogs - git already tracks what files changed. Only create a changeset when the "WHY" behind the change is valuable and non-obvious. Most of the time, use a "note" artifact to capture the insight instead.

**Note** - Insights and warnings (REQUIRED for non-obvious discoveries):
```
amp_write_artifact(
  type: "note",
  title: "Rate limiter resets at midnight UTC",
  category: "warning",
  content: "Token bucket resets at midnight UTC, not rolling windows. Use Quota::with_period() for rolling behavior.",
  linked_files: ["src/middleware/rate_limit.rs"]
)
```

**Note Categories:**
- `warning` - Something will break if not handled correctly
- `insight` - Pattern or approach worth preserving
- `todo` - Work to track beyond this session
- `question` - Uncertainty to investigate later

### Artifact Workflow

After completing significant work:

```
# 1. Create changeset for the work done
amp_write_artifact(
  type: "changeset",
  title: "Add user authentication",
  description: "Complete auth system with JWT",
  files_changed: ["src/auth.py", "src/middleware.py", "src/routes.py"],
  diff_summary: "+350 lines across 3 files"
)

# 2. Create decision artifact if you made architectural choices
amp_write_artifact(
  type: "decision",
  title: "Use bcrypt for password hashing",
  context: "Need secure password storage",
  decision: "bcrypt with cost factor 12",
  consequences: "~300ms per hash, acceptable for auth",
  alternatives: ["argon2 - newer but less library support", "scrypt - similar security"]
)

# 3. Create note if you discovered gotchas
amp_write_artifact(
  type: "note",
  title: "JWT refresh tokens must be single-use",
  category: "warning",
  content: "Reusing refresh tokens allows token theft attacks. Rotate on every refresh."
)
```

### Discovery Tools

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `amp_status` | Health check, system analytics | Verify connectivity, check what's indexed |
| `amp_list` | Browse objects by type | Explore existing decisions, changesets, symbols |
| `amp_query` | Hybrid search (text + vector + graph) | Find specific knowledge |
| `amp_trace` | Follow object relationships | Explore connections from known object |

**Examples:**
```
# Check system health
amp_status()

# Browse recent decisions
amp_list(type: "decision", limit: 10)

# Search for authentication code
amp_query(query: "JWT token validation", mode: "hybrid", limit: 5)

# Trace relationships from a file
amp_trace(object_id: "file-abc123", depth: 2)
```

### Focus Tracking

| Tool | Purpose | When to Use |
|------|---------|-------------|
| `amp_focus` | Track session focus and outputs | EVERY time you start executing a plan or task |

**Actions:**
- `list` - See active sessions
- `set` - Record current focus (MANDATORY when starting work)
- `complete` - Mark focus completed (MANDATORY when done)
- `end` - End session

## Focus Tracking (MANDATORY ON TASK START)

**You MUST call `amp_focus(action: "set")` EVERY TIME you begin executing a plan or task.**

This is NOT optional. Before writing any code or making changes:

```
amp_focus(
  action: "set",
  title: "Brief description of what you're doing",
  plan: ["Step 1", "Step 2", "Step 3"]
)
```

**When to set focus:**
- User asks you to implement a feature → Set focus
- User asks you to fix a bug → Set focus
- User asks you to refactor code → Set focus
- User gives you a multi-step task → Set focus
- ANY time you're about to execute work → Set focus

**When work is complete:**

```
amp_focus(
  action: "complete",
  summary: "What was accomplished",
  files_changed: ["src/file1.py", "src/file2.py"],
  plan: ["Step 1 - done", "Step 2 - done", "Step 3 - done"]
)
```

**Example workflow:**

```
# User: "Add authentication to the API"

# 1. Set focus BEFORE starting
amp_focus(
  action: "set",
  title: "Add API authentication",
  plan: ["Design auth middleware", "Implement JWT validation", "Add login endpoint", "Write tests"]
)

# 2. Do the work...

# 3. Complete focus when done
amp_focus(
  action: "complete",
  summary: "Added JWT authentication with login/refresh endpoints",
  files_changed: ["src/middleware/auth.py", "src/routes/auth.py", "tests/test_auth.py"]
)
```

**NEVER start implementing without setting focus first.**

## Scope Conventions

```
project:{id}  - Shared across all agents on project (USE THIS)
task:{id}     - Isolated to specific task
agent:{id}    - Private to one agent (rare)
```

**Best practice:** Always use `project:{id}` to enable knowledge sharing between sessions.

## Important Rules

1. **Session start** → ALWAYS `amp_cache_read` first
2. **Before any task** → ALWAYS `amp_focus(action: "set")` with title and plan
3. **Every turn** → ALWAYS `amp_cache_write` if you learned/decided anything
4. **After compaction** → ALWAYS `amp_cache_compact` then `amp_cache_read`
5. **After code changes** → ALWAYS `amp_file_sync` (SEQUENTIALLY, not parallel)
6. **Multiple file syncs** → Sync ONE file at a time to avoid server timeouts
7. **Significant decisions** → Create decision artifact with alternatives and rationale
8. **Insights and discoveries** → Create note artifact (most common type)
9. **Changesets** → RARE - only when WHY adds value beyond git diff (not a changelog!)
10. **Task complete** → ALWAYS `amp_focus(action: "complete")` with summary
11. **Gotchas discovered** → Cache as warning AND create note artifact if severe
12. **Unindexed repo** → Tell user to run `amp index` first
13. **NEVER** start implementing without setting focus
14. **NEVER** call multiple `amp_file_sync` in parallel
15. **NEVER** end a turn without considering what to cache
16. **NEVER** skip rituals to save time
17. **NEVER** guess parameters - check `SKILLS/amp-core/references/tool-reference.md`
18. **NEVER** create changeset artifacts just to list which files changed (that's what git is for)

## Skill Documentation

For detailed guidance, read `SKILLS/amp-core/SKILL.md` and navigate to reference docs as needed:

- `references/tool-map.md` - Which tool for which situation
- `references/decision-guide.md` - Flowcharts for tool selection
- `references/tool-reference.md` - Complete parameter reference
- `references/workflows.md` - Step-by-step patterns
- `references/examples.md` - Real-world examples
- `references/artifact-guidelines.md` - When to create artifacts
- `references/file-sync-guide.md` - File provenance details
- `references/cache-guide.md` - Cache mechanics
- `references/cache-policy.md` - Cache best practices
