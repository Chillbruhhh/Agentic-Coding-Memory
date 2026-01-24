# AMP Semantic Caching & Episodic Memory Design

## Overview

This document summarizes the architecture decisions and design patterns
discussed for building: 1. A **token-efficient semantic cache ("unity
layer")** for short-term shared agent memory. 2. A **first-class
episodic memory system** integrated with sessions and artifacts. 3.
Optional **AI-assisted memory pack compression** for advanced
optimization.

The goal is to give agents shared situational awareness with minimal
token usage, strong provenance, and scalable memory growth.

------------------------------------------------------------------------

## 1. Semantic Cache ("Unity Layer")

### Purpose

The semantic cache provides short-term shared working memory so multiple
agents: - Share discoveries instantly. - Avoid recomputation. - Maintain
continuity across sessions. - Operate with minimal token overhead.

This is not long-term memory --- it behaves like RAM for agents.

------------------------------------------------------------------------

### Core Concept: Memory Packs

Agents never receive raw logs or full artifacts. They receive a compact
**Memory Pack**.

**Memory Pack structure:** - Rolling summary (1--3 sentences) - Atomic
facts (5--20) - Decisions / constraints (3--10) - Reusable snippets
(1--5) - Pointers to artifacts / episodes

Target size: **300--900 tokens**, with micro-packs under \~150 tokens.

------------------------------------------------------------------------

### Cache Data Model (SurrealDB)

#### cache_frame

Represents a scoped shared memory workspace.

Fields: - scope_id - summary - summary_embedding - version -
updated_ts - ttl_expires_at

#### cache_item

Thin projection of artifacts or session outputs.

Fields: - scope_id - artifact_id - kind (fact \| decision \| snippet \|
warning) - preview (short text) - facts (array) - embedding -
importance - ttl_expires_at - version - provenance (session_id)

Artifacts remain immutable source-of-truth.

------------------------------------------------------------------------

### Read Flow

1.  Agent loads cache_frame summary.
2.  Semantic search over cache_items within scope.
3.  Top results selected by relevance + importance + recency.
4.  Pack assembled under token budget.
5.  Agent expands via artifact IDs only if needed.

------------------------------------------------------------------------

### Write Flow

Agents write only: - New facts - Decisions - Snippets - Warnings

Before inserting: - Semantic dedupe within scope. - Merge if similarity
\> threshold. - Refresh TTL and importance.

------------------------------------------------------------------------

### Token Efficiency Techniques

-   Strict scoping
-   Token budgeting instead of item count
-   Delta packs using cache_frame.version
-   Deduplication on write
-   Preview + pointer model for large artifacts
-   Micro-pack + expandable pack strategy

------------------------------------------------------------------------

## 2. Episodic Memory

### Purpose

Episodic memory captures narrative knowledge: - What happened - Why it
happened - What changed - What worked / failed

It creates durable institutional memory across time.

------------------------------------------------------------------------

### Episode vs Session vs Artifact

  Layer      Role
  ---------- -------------------------------------------------
  Session    Runtime container for agent execution
  Artifact   Immutable output / knowledge
  Episode    Narrative structure across sessions & artifacts

Episodes can span multiple sessions and link many artifacts.

------------------------------------------------------------------------

### Episode Schema

Fields: - scope_id - start_ts / end_ts - title - goal - outcome -
summary - key_facts - decisions - tags - embedding - linked sessions -
linked artifacts - linked entities

------------------------------------------------------------------------

### Graph Relationships

-   episode → contains → event
-   episode → produced → artifact
-   episode → involves → entity
-   episode → follows / supersedes → episode

------------------------------------------------------------------------

### Consolidation Strategy

Episodes are created via a **consolidator**: - Runs at session end or
periodically. - Summarizes session events. - Extracts key facts and
decisions. - Links artifacts. - Generates embeddings.

Raw events remain immutable.

------------------------------------------------------------------------

### Integration with Cache

When an episode finalizes: - Promote stable facts into cache_items. -
Snapshot summary into cache_frame. - Invalidate temporary cache entries.

------------------------------------------------------------------------

## 3. Changeset Artifacts Integration

Changeset artifacts remain the durable history. Cache derives thin
projections from them.

Example Projection: - preview: short semantic summary - facts: atomic
bullets - artifact pointer

No duplication of large diff content.

------------------------------------------------------------------------

## 4. Retrieval Strategy

### Deterministic Retrieval

-   Semantic similarity over cache_items
-   Importance weighting
-   Recency boost
-   Token budget cutoff

### Optional RRF Fusion

Combine: - Vector similarity - Keyword/BM25 - Graph proximity -
Recency - Cache hits

Fuse via Reciprocal Rank Fusion before packing.

------------------------------------------------------------------------

## 5. AI-Assisted Memory Packing

### When to Use AI Packer

-   Large packs need compression
-   High duplication
-   Task-aware summarization
-   Episodic stitching
-   Delta summarization

### Rules

-   Extractive only (no new facts)
-   Must cite source cache_items
-   Output strict JSON schema
-   Cache results

### Hybrid Strategy

1.  Deterministic candidate pack always available.
2.  AI compression triggered only when beneficial.
3.  Result cached as artifact.

------------------------------------------------------------------------

## 6. Recommended Build Order

1.  cache_frame + cache_item tables
2.  Deterministic pack builder
3.  Delta packs using versioning
4.  Episode schema + consolidator
5.  Optional AI packer
6.  RRF fusion (optional)

------------------------------------------------------------------------

## 7. Design Principles

-   Immutable raw memory
-   Derived short-term memory
-   Token-first optimization
-   Provenance everywhere
-   Deterministic by default
-   AI only when it adds value
-   Human-like episodic structure

------------------------------------------------------------------------

## 8. Outcome

This architecture provides: - Shared agent cognition ("unity") - Fast
handoffs across agents - Token-efficient reasoning - Auditable memory -
Scalable long-term intelligence
