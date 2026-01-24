# Artifact Guidelines

When and why to create artifacts for effective agent memory.

---

## The Core Test

Before creating any artifact, ask:

> **"Would a future agent benefit from knowing this?"**

That future agent might be you after context resets, a different agent, or someone months later wondering why something was built this way.

**If yes** → create an artifact. **If unsure** → use cache instead (it expires, artifacts don't).

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

### 2. Notes - Non-Obvious Discoveries

**Create when:** You learned something that isn't obvious from reading the code.

| Category | Use When |
|----------|----------|
| `warning` | Something will break if not handled correctly |
| `insight` | Pattern or approach worth preserving |
| `todo` | Work that should be tracked beyond this session |
| `question` | Uncertainty future agents should investigate |

```json
{
  "type": "note",
  "title": "Rate limiter resets at midnight UTC",
  "category": "warning",
  "content": "Token bucket resets at midnight UTC, not rolling windows. Use Quota::with_period() for rolling behavior.",
  "tags": ["rate-limiting", "production"]
}
```

**Skip when:** It's in code comments, documented elsewhere, or standard knowledge.

---

### 3. Changesets - Completed Work With Context

**Create when:** You completed meaningful work and the "why" adds value beyond the diff.

```json
{
  "type": "changeset",
  "title": "Implement semantic cache for token-efficient context",
  "description": "Reduces context from 2000+ to ~600 tokens using cosine similarity dedup.",
  "files_changed": ["src/services/cache.rs", "src/handlers/cache.rs"],
  "diff_summary": "+450 lines. New CacheService with get_pack(), write_items(), gc(). TTL-based expiration, importance scoring.",
  "linked_decisions": ["use-surrealdb-for-memory"]
}
```

**Skip when:** Trivial fix, commit message captures everything, no reasoning to add.

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

**Create Changeset when:**
- Completed meaningful work (not trivial)
- "Why" adds value beyond the diff

**Skip when:**
- Code is self-explanatory
- Common knowledge for the technology
- Already documented elsewhere
- Unsure → use cache first

---

## Summary

Artifacts capture what gets lost when context resets:

1. **Preferences** - Choices and why
2. **Discoveries** - Non-obvious learnings
3. **Effective operations** - Patterns that worked

**Quality over quantity.** One insightful artifact beats ten noisy ones.
