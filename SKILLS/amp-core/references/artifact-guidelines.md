# Artifact Guidelines

When and why to create artifacts for effective agent memory.

---

## What Artifacts Are For

Artifacts are **permanent long-term memory** for anything useful about the codebase. They persist forever and are fully searchable.

**Artifacts can store literally anything that would help a future agent understand the codebase:**

| Category | Example |
|----------|---------|
| **User preferences** | "User prefers functional components over class components" |
| **Project conventions** | "Use snake_case for DB columns, camelCase for API responses" |
| **Refactoring rationale** | "Extracted AuthService because auth logic was duplicated in 5 places" |
| **Architectural decisions** | "Chose microservices over monolith for independent scaling" |
| **Dependency choices** | "Using axum over actix-web for simpler lifetime management" |
| **Workarounds** | "setTimeout(0) defers execution due to React 18 batching race condition" |
| **Historical context** | "This module was part of the monolith, extracted in v2.0 migration" |
| **Production gotchas** | "Rate limiter resets at midnight UTC, not rolling windows" |
| **Team decisions** | "We avoid ORMs - team prefers raw SQL for performance visibility" |
| **External constraints** | "API rate limited to 100 req/min - batch operations required" |

**When in doubt, create an artifact.** Artifacts are cheap. Re-learning is expensive.

---

## How Artifacts Work

When you call `amp_write_artifact`, it writes to **three memory layers**:

```
┌─────────────────────────────────────────────────────────────┐
│                    amp_write_artifact                        │
│              type + title + content/decision                 │
└───────────────────────┬─────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│   TEMPORAL    │ │    VECTOR     │ │     GRAPH     │
│   (SurrealDB) │ │  (Embeddings) │ │ (Relationships│
├───────────────┤ ├───────────────┤ ├───────────────┤
│ - Permanent   │ │ - ALWAYS ON   │ │ - linked_files│
│   storage     │ │ - Semantic    │ │ - linked_     │
│ - All fields  │ │   search via  │ │   decisions   │
│ - Audit trail │ │   amp_query   │ │ - project_id  │
│               │ │ - Auto-index  │ │ - Traceable   │
└───────────────┘ └───────────────┘ └───────────────┘
```

**Key behaviors:**
- **Vector embeddings are ALWAYS generated** - Every artifact is automatically indexed for semantic search via `amp_query`
- `linked_files: ["src/auth.rs"]` → Creates `modifies` relationship to file node in graph
- `linked_decisions: ["decision-uuid"]` → Creates `justified_by` relationship
- `project_id: "my-project"` → Links artifact to project for filtering

---

## The Core Test

Before creating any artifact, ask:

> **"Would a future agent benefit from knowing this?"**

That future agent might be you after context resets, a different agent, or someone months later wondering why something was built this way.

**If yes** → create an artifact.
**If maybe** → create an artifact anyway. Better to have it than re-learn it.
**If no** → skip it, stick creating cache instead. (code is self-explanatory, common knowledge, etc.)

Remember: Artifacts can store **anything useful** - user preferences, conventions, rationale, workarounds, historical context, external constraints. Don't limit yourself to just decisions and changesets.

---

## Three Types of Artifacts

### 1. Decisions - Choices With Trade-offs

**Create when:** You chose between alternatives and the reasoning matters.

```json
{
  "type": "decision",
  "title": "Use WebSockets over polling for real-time updates",
  "status": "accepted",
  "context": "Need bidirectional communication. Evaluated polling, SSE, WebSockets.",
  "decision": "WebSockets for sub-100ms latency and full-duplex.",
  "consequences": "Requires sticky sessions or Redis pub/sub for scaling.",
  "alternatives": ["Long polling - 1-3s latency unacceptable", "SSE - one-way only"]
}
```

**Skip when:** Only one reasonable option, trivial choice, or obvious from code.

---

### 2. Notes - Anything Worth Remembering

**Create when:** You learned something, discovered a preference, or have context worth preserving.

Notes are the **most flexible** artifact type. Use them for anything that doesn't fit decisions or changesets:

| Category | Use When |
|----------|----------|
| `warning` | Something will break if not handled correctly |
| `insight` | Pattern, approach, or discovery worth preserving |
| `todo` | Work that should be tracked beyond this session |
| `question` | Uncertainty future agents should investigate |
| `reference` | User preferences, conventions, external constraints |

**Examples:**

```json
// Production gotcha
{
  "type": "note",
  "title": "Rate limiter resets at midnight UTC",
  "category": "warning",
  "content": "Token bucket resets at midnight UTC, not rolling windows. Use Quota::with_period() for rolling behavior.",
  "tags": ["rate-limiting", "production"]
}
```

```json
// User preference
{
  "type": "note",
  "title": "User prefers verbose error messages",
  "category": "reference",
  "content": "User wants detailed error messages with stack traces in development. Include file paths and line numbers when possible.",
  "tags": ["user-preference", "error-handling"]
}
```

```json
// Project convention
{
  "type": "note",
  "title": "API naming conventions",
  "category": "reference",
  "content": "Use kebab-case for API routes (/user-settings), snake_case for DB columns (user_id), camelCase for JSON responses (userId).",
  "tags": ["conventions", "api"]
}
```

```json
// Refactoring rationale
{
  "type": "note",
  "title": "Why AuthService was extracted",
  "category": "insight",
  "content": "Auth logic was duplicated across UserController, AdminController, and APIController. Extracted to single AuthService to ensure consistent token validation and session handling.",
  "linked_files": ["src/services/auth.rs"],
  "tags": ["refactoring", "architecture"]
}
```

```json
// Temporary workaround
{
  "type": "note",
  "title": "setTimeout workaround for React 18 batching",
  "category": "warning",
  "content": "Using setTimeout(0) in useEffect to defer state updates. React 18 automatic batching causes race condition with external library. Remove when library updates to support React 18.",
  "linked_files": ["src/hooks/useDataSync.tsx"],
  "tags": ["workaround", "react", "temporary"]
}
```

**Skip when:** It's in code comments, documented elsewhere, or standard knowledge.

---

### 3. Changesets - Completed Work With Context

**Changesets are the LEAST common artifact type.** Git already tracks what changed. Changesets only add value when they explain **WHY** something was done in a way that isn't obvious from the code or commit message.

**Create when:** 
- The reasoning behind the change would help future agents understand the codebase
- You're capturing architectural context that would be lost otherwise
- The "why" is significantly more valuable than the "what"

```json
{
  "type": "changeset",
  "title": "Implement semantic cache for token-efficient context",
  "description": "Reduces context from 2000+ to ~600 tokens using cosine similarity dedup. Previous approach loaded all items, causing context overflow in long sessions. Cosine threshold of 0.85 chosen after testing showed it eliminates 70% of redundant items without losing semantic coverage.",
  "files_changed": ["src/services/cache.rs", "src/handlers/cache.rs"],
  "diff_summary": "+450 lines. New CacheService with get_pack(), write_items(), gc(). TTL-based expiration, importance scoring.",
  "linked_decisions": ["use-surrealdb-for-memory"]
}
```

**Skip when (MOST of the time):**
- Git commit message captures the change adequately
- You're just listing which files changed (that's what `git diff` is for)
- There's no "why" that adds value beyond the code itself
- The change is routine/trivial (bug fix, typo, formatting)

### Changeset Anti-Pattern: The Changelog Trap

**DON'T do this:**
```json
{
  "type": "changeset",
  "title": "Updated documentation files",
  "files_changed": ["docs/api.md", "docs/concepts.md", "README.md"],
  "diff_summary": "Added RRF section, updated endpoints"
}
```

This is just a changelog. Git already tracks this. Future agents don't benefit from knowing "files were changed."

**DO this instead (or skip entirely):**
```json
{
  "type": "note",
  "title": "RRF is how AMP ranks hybrid query results",
  "category": "insight",
  "content": "Hybrid queries combine vector, graph, and temporal results using Reciprocal Rank Fusion (RRF) with k=60. Items appearing in multiple retrieval methods get boosted. This is the core ranking algorithm - document it well.",
  "tags": ["architecture", "retrieval", "rrf"]
}
```

The note captures the **insight** (RRF is the ranking algorithm) rather than the **activity** (I updated some docs).

**Rule of thumb:** If your changeset doesn't explain WHY in a way that helps future agents, don't create it. Use a note for the insight, or skip the artifact entirely.

---

## Cache vs Artifact

| | Cache | Artifact |
|--|-------|----------|
| **Lifetime** | 30 min TTL | Permanent |
| **Purpose** | Working memory | Historical record |
| **Size** | Compact facts | Detailed |

```
Is it valuable?
├─ NO → Don't store
└─ YES → Will it matter after 30 minutes?
    ├─ NO → Cache
    └─ YES → Will future agent benefit?
        ├─ NO → Cache (let it expire)
        └─ YES → Artifact
```

**Rule:** When in doubt, cache. Upgrade to artifact later if it proves valuable.

---

## Quality Guidelines

### Capture Why, Not Just What

| Low Value | High Value |
|-----------|------------|
| "Using Redis for caching" | "Using Redis because we need sub-ms reads and data loss on restart is acceptable" |
| "Added error handling" | "Added retry logic because external API has ~2% transient failures" |

### Be Specific and Actionable

| Vague | Specific |
|-------|----------|
| "Database is slow sometimes" | "Queries >100ms when user_events exceeds 1M rows - add index on created_at" |
| "Watch out for edge cases" | "Empty arrays cause panic in process_batch() - guard at line 142" |

### Link Related Objects

```json
{
  "linked_decisions": ["api-rate-limit-strategy"],
  "linked_files": ["src/middleware/rate_limit.rs"]
}
```

Creates graph relationships for `amp_trace` discovery.

---

## Anti-Patterns

### The Changelog Trap (Most Common Mistake)

**This is the #1 artifact anti-pattern.** Creating changesets that just list which files changed.

**Bad:**
```json
{
  "type": "changeset",
  "title": "RRF Documentation Added to Docs",
  "files_changed": ["docs/concepts/hybrid-retrieval.md", "docs/api/overview.md"],
  "diff_summary": "Created new file, updated references"
}
```

This adds zero value. Git already tracks file changes. Future agents don't need to know "an agent updated some docs."

**Good:** Either skip the artifact entirely, or capture the **insight**:
```json
{
  "type": "note",
  "title": "RRF (k=60) is the ranking algorithm for hybrid queries",
  "category": "insight",
  "content": "AMP uses Reciprocal Rank Fusion to combine vector, graph, and temporal results. Formula: RRF(d) = Σ 1/(k + rank(d)). Items in multiple result sets get boosted scores. k=60 is the standard value.",
  "tags": ["architecture", "retrieval"]
}
```

### Artifact Spam
**Bad:** One artifact per file touched.
**Good:** One changeset for the completed feature with all files listed.

### Duplicating the Obvious
**Bad:** "UserService has a getUser method"
**Good:** "getUser() caches for 60s - don't use for auth decisions, use getUserFresh()"

### Recording Common Knowledge
**Bad:** "Use async/await for I/O operations"
**Good:** "Use tokio over async-std - team experience + dependency compatibility"

### Empty Links
**Bad:** `"linked_decisions": []` when decisions exist
**Good:** Connect related artifacts for traceability

### Creating Artifacts for Activity, Not Insight
**Bad:** "Updated 5 files during refactoring session"
**Good:** "Extracted AuthService because auth logic was duplicated in 5 controllers"

The test: **Does this capture INSIGHT or just ACTIVITY?** Only insights belong in artifacts.

---

## Quick Reference

**Create Decision when:**
- Chose between 2+ viable alternatives
- Trade-offs aren't obvious from code
- Future agent might ask "why this approach?"

**Create Note when:**
- Discovered something non-obvious
- Hit a gotcha worth warning about
- Found a pattern that worked well
- Learned a user preference or project convention
- Want to capture rationale or historical context

**Create Changeset when (RARE):**
- The "WHY" behind the change is valuable and non-obvious
- Architectural context would be lost without it
- NOT just to record which files changed

**Skip when:**
- Just listing files changed (use git)
- Code is self-explanatory
- Common knowledge for the technology
- Already documented elsewhere
- Capturing activity, not insight
- Unsure → use cache first

---

## Summary

Artifacts are permanent memory for **anything useful about the codebase** - things that get lost when context resets:

1. **Preferences** - User and team preferences, conventions, style choices
2. **Choices** - Decisions made and why, alternatives considered
3. **Discoveries** - Non-obvious learnings, gotchas, production behavior
4. **Rationale** - Why refactors happened, why dependencies were chosen
5. **Context** - Historical background, external constraints, workarounds
6. **Patterns** - Approaches that worked well, anti-patterns to avoid

**Don't limit yourself to the three artifact types.** A "note" can contain anything - user preferences, project conventions, historical context, temporary workarounds, external API quirks, team decisions, or literally anything else worth remembering.

**Quality over quantity** One insightful artifact beats ten noisy ones - but useful artifacts beat zero.

---

## Complete Parameter Reference

### Common Fields (all types)

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | `decision`, `note`, `changeset`, `filelog` |
| `title` | Yes | Clear, descriptive title |
| `project_id` | No | Link to project for filtering |
| `tags` | No | Array of tags for categorization |
| `linked_files` | No | Files this artifact relates to (creates graph edges) |
| `linked_decisions` | No | Related decision artifacts |
| `linked_objects` | No | Generic object links |
| `agent_id` | No | Agent identifier for audit trail |
| `run_id` | No | Link to agent run |

### Decision Fields

```json
{
  "type": "decision",
  "title": "Use Redis for session caching",
  "status": "accepted",
  "context": "Need fast session storage with <10ms reads",
  "decision": "Redis with 24h TTL, cluster mode for HA",
  "consequences": "Requires Redis deployment, adds infra cost",
  "alternatives": ["PostgreSQL - too slow", "In-memory - no persistence"]
}
```

| Field | Description |
|-------|-------------|
| `status` | `proposed`, `accepted`, `deprecated`, `superseded` |
| `context` | Problem being solved, constraints |
| `decision` | What was decided |
| `consequences` | Impact, trade-offs |
| `alternatives` | Other options considered |

### Note Fields

```json
{
  "type": "note",
  "title": "API rate limits reset at midnight UTC",
  "category": "warning",
  "content": "Rate limiter uses fixed windows, not sliding. Burst traffic at midnight may exceed limits.",
  "tags": ["api", "rate-limiting"]
}
```

| Field | Description |
|-------|-------------|
| `category` | `warning`, `insight`, `todo`, `question` |
| `content` | The note content |

### Changeset Fields

```json
{
  "type": "changeset",
  "title": "Add authentication middleware",
  "description": "JWT validation with refresh token rotation",
  "files_changed": ["src/middleware/auth.rs", "src/handlers/login.rs"],
  "diff_summary": "+200 lines. New AuthMiddleware, token refresh endpoint.",
  "linked_decisions": ["use-jwt-over-sessions"]
}
```

| Field | Description |
|-------|-------------|
| `description` | What the changeset accomplishes |
| `files_changed` | Array of modified files |
| `diff_summary` | Brief summary of changes |

### FileLog Fields (internal use)

```json
{
  "type": "filelog",
  "title": "src/auth/login.rs",
  "file_path": "src/auth/login.rs",
  "summary": "Authentication handler with OAuth support",
  "symbols": ["login", "validate_token", "refresh"],
  "dependencies": ["oauth2", "jsonwebtoken"]
}
```

| Field | Description |
|-------|-------------|
| `file_path` | Path to the file |
| `summary` | AI-generated file summary |
| `symbols` | Key symbols in file |
| `dependencies` | File dependencies |

---

## Discovering Artifacts

After creating artifacts, they're discoverable via:

- **`amp_query`** - Semantic search finds artifacts by content similarity
- **`amp_trace`** - Follow relationships from files to linked artifacts
- **`amp_list`** - Browse artifacts by type (`type: "decision"`)
