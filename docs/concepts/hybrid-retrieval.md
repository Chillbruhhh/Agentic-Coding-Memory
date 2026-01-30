# Hybrid Retrieval

AMP uses hybrid retrieval to combine multiple search methods into a single, unified result set. This document explains how the system works and how results are ranked.

## Overview

When you query AMP, three retrieval methods run in parallel:

| Method | What it finds | Best for |
|--------|---------------|----------|
| **Vector Search** | Semantically similar content | "Find code related to authentication" |
| **Graph Traversal** | Connected objects via relationships | "What calls this function?" |
| **Temporal Filtering** | Recent or time-bounded results | "Changes in the last week" |

Each method returns a ranked list of results. AMP then fuses these lists using **Reciprocal Rank Fusion (RRF)**.

## Reciprocal Rank Fusion (RRF)

RRF is a rank aggregation algorithm that combines multiple ranked lists into a single unified ranking. It was introduced by Cormack, Clarke, and Buettcher (2009) and is widely used in information retrieval systems.

### The Algorithm

For each document `d` appearing in any of the ranked lists:

```
RRF_score(d) = Σ 1 / (k + rank_i(d))
```

Where:
- `k` is a constant (AMP uses k=60, the standard value)
- `rank_i(d)` is the rank of document `d` in list `i` (1-indexed)
- The sum is over all lists where `d` appears

### Example

Suppose we have three retrieval methods returning these rankings:

**Vector Search Results:**
1. auth.py (rank 1)
2. login.py (rank 2)
3. session.py (rank 3)

**Graph Traversal Results:**
1. login.py (rank 1)
2. middleware.py (rank 2)
3. auth.py (rank 3)

**Temporal Results:**
1. session.py (rank 1)
2. auth.py (rank 2)

**RRF Calculation (k=60):**

| Document | Vector | Graph | Temporal | RRF Score |
|----------|--------|-------|----------|-----------|
| auth.py | 1/(60+1) = 0.0164 | 1/(60+3) = 0.0159 | 1/(60+2) = 0.0161 | **0.0484** |
| login.py | 1/(60+2) = 0.0161 | 1/(60+1) = 0.0164 | - | **0.0325** |
| session.py | 1/(60+3) = 0.0159 | - | 1/(60+1) = 0.0164 | **0.0323** |
| middleware.py | - | 1/(60+2) = 0.0161 | - | **0.0161** |

**Final Ranking:**
1. auth.py (0.0484) - appears in all 3 lists
2. login.py (0.0325) - appears in 2 lists
3. session.py (0.0323) - appears in 2 lists
4. middleware.py (0.0161) - appears in 1 list

### Why RRF Works

1. **No score normalization needed** - RRF uses ranks, not scores, so different retrieval methods don't need calibration
2. **Diminishing returns** - Items ranked lower contribute less (1/61 vs 1/70)
3. **Multi-source boost** - Items appearing in multiple lists accumulate scores
4. **Outlier resistance** - A single method can't dominate the final ranking

### The k Parameter

The constant `k=60` controls how much weight lower-ranked items receive:

- **Lower k** (e.g., k=10): Top ranks dominate more strongly
- **Higher k** (e.g., k=100): Rankings are more evenly weighted
- **k=60**: Standard value balancing top-rank importance with diversity

AMP uses k=60 as recommended in the original paper.

## Retrieval Methods in Detail

### Vector Search

Vector search finds semantically similar content using embeddings:

1. Query text is converted to an embedding vector
2. Cosine similarity is computed against stored embeddings
3. Results are ranked by similarity score

**Embedding Providers:**
- Ollama (local, free)
- OpenAI (cloud, high quality)
- OpenRouter (cloud, multiple models)

**Best for:**
- Natural language queries ("find the login handler")
- Conceptual similarity ("code that validates user input")
- Fuzzy matching (doesn't require exact terms)

### Graph Traversal

Graph traversal follows relationships between objects:

1. Start from seed objects (from vector search or explicit IDs)
2. Traverse edges: `calls`, `imports`, `defined_in`, `depends_on`
3. Rank by graph distance and relationship type

**Relationship Types:**
| Relationship | Meaning |
|--------------|---------|
| `calls` | Function A calls function B |
| `imports` | File A imports from file B |
| `defined_in` | Symbol is defined in file |
| `depends_on` | Module A depends on module B |
| `authored_by` | Object created by agent/run |

**Best for:**
- Dependency analysis ("what uses this function?")
- Impact analysis ("what would break if I change this?")
- Provenance tracking ("who created this?")

### Temporal Filtering

Temporal filtering restricts results by time:

1. Apply date range filters to `created_at` or `updated_at`
2. Optionally boost recent results
3. Useful for change tracking and activity analysis

**Filter Options:**
- `created_after`: Only results created after timestamp
- `created_before`: Only results created before timestamp
- `updated_after`: Only results modified after timestamp

**Best for:**
- Recent changes ("what was modified today?")
- Historical analysis ("decisions made last month")
- Activity tracking ("recent agent runs")

## Query API

### Basic Query

```bash
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "authentication",
    "hybrid": true,
    "limit": 10
  }'
```

### With Filters

```bash
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "login handler",
    "hybrid": true,
    "limit": 20,
    "filters": {
      "object_types": ["symbol"],
      "project_id": "my-project",
      "created_after": "2026-01-01T00:00:00Z"
    }
  }'
```

### MCP Tool

```
amp_query(
  query: "authentication logic",
  mode: "hybrid",
  limit: 10,
  filters: { object_types: ["symbol", "decision"] }
)
```

## Response Format

Query responses include the RRF score and source information:

```json
{
  "results": [
    {
      "id": "uuid-1",
      "type": "symbol",
      "name": "authenticate_user",
      "score": 0.0484,
      "sources": ["vector", "graph", "temporal"],
      "ranks": {
        "vector": 1,
        "graph": 3,
        "temporal": 2
      }
    },
    {
      "id": "uuid-2",
      "type": "symbol", 
      "name": "login_handler",
      "score": 0.0325,
      "sources": ["vector", "graph"],
      "ranks": {
        "vector": 2,
        "graph": 1
      }
    }
  ],
  "total": 42,
  "limit": 10,
  "retrieval_stats": {
    "vector_count": 25,
    "graph_count": 18,
    "temporal_count": 12,
    "fused_count": 42
  }
}
```

## Tuning Retrieval

### When Vector Search Dominates

If you're getting too many semantically similar but unrelated results:
- Add graph traversal seeds to anchor the search
- Use more specific query terms
- Filter by `object_types` to narrow scope

### When Graph Traversal Misses

If relationship-based results aren't appearing:
- Ensure the codebase is fully indexed
- Check that relationships were created during indexing
- Use `amp_trace` to verify object connections

### When Temporal is Too Restrictive

If time filters eliminate good results:
- Widen the date range
- Use `created_after` only (no upper bound)
- Consider disabling temporal for evergreen queries

## Performance

| Operation | Typical Latency |
|-----------|-----------------|
| Vector search (10k objects) | 50-100ms |
| Graph traversal (2 hops) | 20-50ms |
| Temporal filter | 10-20ms |
| RRF fusion | <5ms |
| **Total hybrid query** | **100-200ms** |

## References

- Cormack, G. V., Clarke, C. L., & Buettcher, S. (2009). Reciprocal rank fusion outperforms condorcet and individual rank learning methods. *SIGIR '09*.
- [SurrealDB Vector Search](https://surrealdb.com/docs/surrealql/functions/vector)
- [AMP Query API](../api/overview.md#query--search)
