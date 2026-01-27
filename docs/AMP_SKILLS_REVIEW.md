# AMP Skills Effectiveness Review (Coding Context)

Scope: coding-focused agents with light brainstorming; optimize for accuracy + speed; keep skill docs short (200-300 lines max). This review is a checklist of coverage gaps with concise proposals for any missing or weak area.

## Checklist

- [x] Clear purpose of AMP and three memory layers
- [x] Tool inventory with categories
- [x] Core workflows (start, edit, handoff, end)
- [x] Path ambiguity handling for file tools
- [x] Cache vs artifact decision guidance
- [x] Examples for all tool categories
- [x] Scope conventions (project/task/agent/session)
- [x] Anti-patterns and quality guidance for artifacts
- [x] Post-edit file sync reminder

- [ ] **Latency/throughput guidance** (how to stay fast)
  - Proposal: Add a ?Speed Profile? section in `amp-core/SKILL.md` with default limits (e.g., `amp_query limit=5`, `amp_cache_read limit=5`, use `include_content` only when needed) and a ?fast path? sequence for typical coding tasks.

- [ ] **Error handling playbook per tool**
  - Proposal: Add a short table in `references/tool-reference.md` mapping common errors to immediate next action (retry/backoff, reduce limit, disambiguate path, use absolute path, or `amp_status`).

- [ ] **Graph hygiene/relationship expectations**
  - Proposal: Add a 6?8 line ?Graph Expectations? section in `references/file-sync-guide.md` describing the expected relationships (project->dir->file via `defined_in`, no direct project->file edge unless configured), and how to verify with `amp_trace`.

- [ ] **Canonical path normalization rules** (Windows vs container)
  - Proposal: Add a ?Path normalization? subsection in `file-sync-guide.md` clarifying path normalization (drive letters, slashes) and recommending absolute paths for reliability on Windows hosts.

- [ ] **Minimal, prescriptive coding workflow**
  - Proposal: Add a ?3-step coding loop? snippet in `amp-core/SKILL.md`:
    1) `amp_filelog_get` (only on unfamiliar file)
    2) edit
    3) `amp_file_sync` with 1?2 sentence summary

- [ ] **Brainstorming guardrails** (within coding environment)
  - Proposal: Add a short ?Brainstorming rules? section in `workflows.md` for when to use `amp_cache_write` vs `amp_write_artifact` for ideas; emphasize caching ideas unless they became decisions.

- [ ] **Token budget defaults in tool examples**
  - Proposal: Standardize example calls to include conservative defaults (limit=5, depth=2, include_content=false) to bias speed.

- [ ] **Update cadence for cache (freshness)**
  - Proposal: Add a ?When to refresh cache? bullet list in `cache-guide.md` (e.g., every 30?60 min, before/after big context shifts).

- [ ] **Tool selection quick card for coding**
  - Proposal: Add a ?Coding quick card? in `tool-map.md` with 4 lines: read cache, get filelog if unfamiliar, sync after edit, write artifact only for decisions/changesets.

- [ ] **Explicit ?do not? list for file sync**
  - Proposal: Add a small ?Do not? list: don?t sync unchanged files, don?t sync generated artifacts, don?t use basename-only paths if ambiguous.

- [ ] **Examples for failure recovery**
  - Proposal: Add 2?3 failure examples in `examples.md` (ambiguous path, empty query, 500 error) and the next correct call.

- [ ] **Agent handoff minimalism**
  - Proposal: Add a ?Handoff minimal set? section in `workflows.md` that specifies the 3 most important cache items to write for coding tasks.

## Notes on doc size

To keep each file under 200?300 lines, prefer:
- Short bullet lists over long prose
- Collapsible example blocks (one per tool category)
- A single ?fast path? example per workflow

## Recommended next steps

1) Approve targeted edits for the missing sections above.
2) I can implement them with minimal diffs while keeping each file under 200?300 lines.
