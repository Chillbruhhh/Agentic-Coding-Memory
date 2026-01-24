# AMP Tool Map

This file describes when to use AMP tools and how they relate to memory layers.

## Memory tools

- `amp_cache_get`
  - Purpose: Retrieve short-term memory pack with a token budget.
  - Use when: You need a fast, compact memory refresh for current work.
  - Scope: `project:{id}` or `task:{id}`.
  - Note: Use `token_budget` around 600 unless told otherwise.

- `amp_cache_write`
  - Purpose: Store short-term memory items with dedup.
  - Use when: Capture facts, decisions, snippets, or warnings mid-task.
  - Items: Always objects with `kind`, `preview`, and optional `facts`.

## Durable knowledge tools

- `amp_write_artifact`
  - Purpose: Persist durable knowledge with full graph relationships.
  - Types: `decision`, `changeset`, `note`, `filelog`
  - Use when:
    - `decision`: An architectural choice affects future work
    - `changeset`: A unit of work is completed and should be recorded
    - `note`: Insights, warnings, or references to preserve
    - `filelog`: File metadata and symbol tracking

## File provenance tools

- `amp_filelog_get`
  - Purpose: Read dependency + symbol log for a file.
  - Use when: You need context before modifying a file.

- `amp_filelog_update`
  - Purpose: Update file log after changes.
  - Use when: You changed behavior or structure in a file.

## Run tracking

- `amp_run_start` / `amp_run_end`
  - Purpose: Track a bounded run with outputs.
  - Use when: You want a coherent execution record.

