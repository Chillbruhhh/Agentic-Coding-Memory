# Feature: Semantic Cache (Unity Layer) for Token-Efficient Agent Memory

The following plan should be complete, but it's important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Implement a **semantic cache layer** ("Unity Layer") that provides short-term shared working memory for AI agents. This cache stores thin projections of artifacts (facts, decisions, snippets) with TTL-based expiration, enabling agents to receive compact **Memory Packs** (300-900 tokens) instead of full artifact records (2000-5000 tokens). This delivers **4-10x token reduction** while maintaining semantic retrieval quality.

## User Story

As an **AI agent using AMP**
I want to **receive compact, relevant context bundles under a token budget**
So that **I can maintain shared situational awareness without exhausting context windows**

## Problem Statement

Currently, `amp_context` returns full artifact objects which consume 2000-5000 tokens per query. Agents calling context repeatedly waste tokens on redundant information. There's no short-term shared memory layer - each agent query recomputes from raw artifacts.

## Solution Statement

Add a semantic cache layer between artifacts and agent queries:
1. **cache_frame**: Scoped memory workspaces with rolling summaries
2. **cache_item**: Thin projections (facts/decisions/snippets) with embeddings
3. **PackBuilder**: Deterministic assembly under token budget
4. **Cache-enhanced amp_context**: Check cache first, build pack on miss

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: Medium (3-4 hours)
**Primary Systems Affected**: server/services, mcp-server/tools, spec/schema
**Dependencies**: Existing embedding service, hybrid retrieval, SurrealDB

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/spec/schema.surql` (lines 1-206) - Why: SurrealDB schema patterns, field types, index definitions
- `amp/server/src/services/hybrid.rs` (lines 1-100) - Why: Service structure pattern, Arc<Database>, error handling
- `amp/server/src/services/mod.rs` (lines 1-11) - Why: Module registration pattern
- `amp/server/src/services/embedding.rs` - Why: Embedding service trait and usage
- `amp/mcp-server/src/tools/context.rs` (lines 1-105) - Why: Current amp_context implementation to enhance
- `amp/mcp-server/src/tools/mod.rs` (lines 1-15) - Why: Tool module registration pattern
- `amp/mcp-server/src/main.rs` (lines 40-230) - Why: Tool registration and call_tool dispatch pattern
- `amp/server/src/handlers/artifacts.rs` - Why: Artifact write flow for cache promotion hook

### New Files to Create

- `amp/server/src/services/cache.rs` - Cache service with pack builder
- `amp/mcp-server/src/tools/cache.rs` - New MCP cache tools

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [SurrealDB Vector Search](https://surrealdb.com/docs/surrealdb/reference-guide/vector-search)
  - Section: MTREE index, vector::similarity::cosine
  - Why: Vector indexing for cache_item embeddings
- [SurrealDB TTL Pattern](https://surrealdb.com/docs/surrealdb/reference-guide/performance-best-practices)
  - Section: Index strategies
  - Why: Efficient TTL-based queries
- [rmcp Tool Implementation](https://docs.rs/rmcp)
  - Section: ServerHandler trait, tool macro
  - Why: MCP tool implementation pattern

### Patterns to Follow

**Naming Conventions:**
- Services: `CacheService`, `PackBuilder` (PascalCase structs)
- Fields: `scope_id`, `cache_item`, `token_count` (snake_case)
- MCP tools: `amp_cache_get`, `amp_cache_write` (snake_case with amp_ prefix)

**Error Handling:**
```rust
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Cache miss for scope: {0}")]
    CacheMiss(String),
}
```

**Service Constructor Pattern:**
```rust
impl CacheService {
    pub fn new(
        db: Arc<Database>,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        Self { db, embedding_service }
    }
}
```

**MCP Tool Pattern:**
```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpCacheGetInput {
    pub scope_id: String,
    #[serde(default)]
    pub token_budget: Option<usize>,
}
```

---

## IMPLEMENTATION PLAN

### Phase 1: Schema Extension

Add cache tables to SurrealDB schema with vector indexes and TTL support.

**Tasks:**
- Define cache_frame table (scoped memory workspaces)
- Define cache_item table (thin projections with embeddings)
- Add MTREE vector indexes for semantic search
- Add composite indexes for scope + recency queries

### Phase 2: Cache Service

Implement core cache service with pack builder logic.

**Tasks:**
- Create CacheService struct with db + embedding dependencies
- Implement get_pack() - retrieve or build memory pack
- Implement write_items() - add facts/decisions with dedup
- Implement PackBuilder - token-budgeted assembly
- Add TTL-based garbage collection

### Phase 3: MCP Tools

Expose cache operations via MCP protocol.

**Tasks:**
- Create amp_cache_get tool
- Create amp_cache_write tool
- Register tools in mcp-server/main.rs
- Enhance amp_context to use cache

### Phase 4: Integration & Testing

Connect cache to artifact write flow.

**Tasks:**
- Hook artifact writes to cache promotion
- Test cache hit/miss scenarios
- Validate token budget enforcement

---

## STEP-BY-STEP TASKS

### UPDATE amp/spec/schema.surql

- **IMPLEMENT**: Add cache_frame and cache_item tables with all fields
- **PATTERN**: Follow file_logs table pattern (lines 154-175)
- **GOTCHA**: Use SCHEMAFULL for strict validation, array<float> for embeddings
- **VALIDATE**: `cargo run -p amp-server` (schema loads on startup)

```sql
-- Semantic Cache: Scoped Memory Frames
DEFINE TABLE cache_frame SCHEMAFULL;
DEFINE FIELD id ON cache_frame TYPE record<cache_frame>;
DEFINE FIELD scope_id ON cache_frame TYPE string;
DEFINE FIELD summary ON cache_frame TYPE string;
DEFINE FIELD summary_embedding ON cache_frame TYPE option<array<float>>;
DEFINE FIELD version ON cache_frame TYPE int DEFAULT 0;
DEFINE FIELD token_count ON cache_frame TYPE int DEFAULT 0;
DEFINE FIELD updated_at ON cache_frame TYPE datetime DEFAULT time::now();
DEFINE FIELD ttl_expires_at ON cache_frame TYPE option<datetime>;
DEFINE FIELD created_at ON cache_frame TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_cache_frame_scope ON cache_frame COLUMNS scope_id UNIQUE;
DEFINE INDEX idx_cache_frame_ttl ON cache_frame COLUMNS ttl_expires_at;
DEFINE INDEX idx_cache_frame_embedding ON cache_frame COLUMNS summary_embedding MTREE DIMENSION 1536;

-- Semantic Cache: Thin Projections
DEFINE TABLE cache_item SCHEMAFULL;
DEFINE FIELD id ON cache_item TYPE record<cache_item>;
DEFINE FIELD scope_id ON cache_item TYPE string;
DEFINE FIELD artifact_id ON cache_item TYPE option<string>;
DEFINE FIELD kind ON cache_item TYPE string ASSERT $value IN ["fact", "decision", "snippet", "warning"];
DEFINE FIELD preview ON cache_item TYPE string;
DEFINE FIELD facts ON cache_item TYPE array<string> DEFAULT [];
DEFINE FIELD embedding ON cache_item TYPE option<array<float>>;
DEFINE FIELD importance ON cache_item TYPE float DEFAULT 0.5;
DEFINE FIELD access_count ON cache_item TYPE int DEFAULT 0;
DEFINE FIELD ttl_expires_at ON cache_item TYPE option<datetime>;
DEFINE FIELD version ON cache_item TYPE int DEFAULT 0;
DEFINE FIELD provenance ON cache_item TYPE object DEFAULT {};
DEFINE FIELD created_at ON cache_item TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at ON cache_item TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_cache_item_scope ON cache_item COLUMNS scope_id;
DEFINE INDEX idx_cache_item_kind ON cache_item COLUMNS kind;
DEFINE INDEX idx_cache_item_importance ON cache_item COLUMNS importance;
DEFINE INDEX idx_cache_item_ttl ON cache_item COLUMNS ttl_expires_at;
DEFINE INDEX idx_cache_item_embedding ON cache_item COLUMNS embedding MTREE DIMENSION 1536;
```

---

### CREATE amp/server/src/services/cache.rs

- **IMPLEMENT**: CacheService struct and MemoryPack types
- **PATTERN**: Mirror hybrid.rs structure (lines 51-78)
- **IMPORTS**:
  ```rust
  use std::sync::Arc;
  use serde::{Serialize, Deserialize};
  use thiserror::Error;
  use tokio::time::Duration;
  use crate::database::Database;
  use crate::services::embedding::EmbeddingService;
  ```
- **GOTCHA**: Use chars/4 heuristic for token estimation (fast, ~10% error)
- **VALIDATE**: `cargo build -p amp-server`

```rust
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use thiserror::Error;
use chrono::{DateTime, Utc, Duration as ChronoDuration};

use crate::database::Database;
use crate::services::embedding::EmbeddingService;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Cache miss for scope: {0}")]
    CacheMiss(String),
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheItem {
    pub id: Option<String>,
    pub scope_id: String,
    pub artifact_id: Option<String>,
    pub kind: CacheItemKind,
    pub preview: String,
    pub facts: Vec<String>,
    pub embedding: Option<Vec<f32>>,
    pub importance: f32,
    pub access_count: i32,
    pub provenance: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheItemKind {
    Fact,
    Decision,
    Snippet,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPack {
    pub scope_id: String,
    pub summary: String,
    pub facts: Vec<CacheItem>,
    pub decisions: Vec<CacheItem>,
    pub snippets: Vec<CacheItem>,
    pub warnings: Vec<CacheItem>,
    pub artifact_pointers: Vec<String>,
    pub token_count: usize,
    pub version: u64,
    pub is_fresh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheFrame {
    pub scope_id: String,
    pub summary: String,
    pub version: u64,
    pub token_count: usize,
    pub updated_at: DateTime<Utc>,
}

pub struct CacheService {
    db: Arc<Database>,
    embedding_service: Arc<dyn EmbeddingService>,
    default_ttl_minutes: i64,
    freshness_threshold_seconds: i64,
}

impl CacheService {
    pub fn new(
        db: Arc<Database>,
        embedding_service: Arc<dyn EmbeddingService>,
    ) -> Self {
        Self {
            db,
            embedding_service,
            default_ttl_minutes: 30,
            freshness_threshold_seconds: 300, // 5 minutes
        }
    }

    /// Estimate token count using chars/4 heuristic
    pub fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }

    /// Get or build memory pack for scope
    pub async fn get_pack(
        &self,
        scope_id: &str,
        token_budget: usize,
        query_embedding: Option<&[f32]>,
    ) -> Result<MemoryPack, CacheError> {
        // 1. Check for fresh cache_frame
        let frame = self.get_frame(scope_id).await?;
        let is_fresh = frame.as_ref()
            .map(|f| {
                let age = Utc::now() - f.updated_at;
                age.num_seconds() < self.freshness_threshold_seconds
            })
            .unwrap_or(false);

        // 2. Query cache_items for this scope
        let items = self.query_items(scope_id, query_embedding, 50).await?;

        // 3. Build pack under token budget
        let pack = self.build_pack(scope_id, &frame, items, token_budget, is_fresh);

        // 4. Update frame if we rebuilt
        if !is_fresh {
            self.update_frame(scope_id, &pack).await?;
        }

        Ok(pack)
    }

    async fn get_frame(&self, scope_id: &str) -> Result<Option<CacheFrame>, CacheError> {
        let query = format!(
            "SELECT * FROM cache_frame WHERE scope_id = '{}' LIMIT 1",
            scope_id.replace("'", "\\'")
        );

        let mut response = self.db.client.query(&query).await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let results: Vec<Value> = response.take(0)
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        if let Some(obj) = results.first() {
            Ok(Some(CacheFrame {
                scope_id: obj.get("scope_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                summary: obj.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                version: obj.get("version").and_then(|v| v.as_u64()).unwrap_or(0),
                token_count: obj.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                updated_at: obj.get("updated_at")
                    .and_then(|v| v.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
            }))
        } else {
            Ok(None)
        }
    }

    async fn query_items(
        &self,
        scope_id: &str,
        query_embedding: Option<&[f32]>,
        limit: usize,
    ) -> Result<Vec<CacheItem>, CacheError> {
        let query = if let Some(embedding) = query_embedding {
            let vec_str = embedding.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "SELECT *, vector::similarity::cosine(embedding, [{}]) AS similarity \
                 FROM cache_item \
                 WHERE scope_id = '{}' AND embedding IS NOT NONE \
                 ORDER BY similarity DESC, importance DESC \
                 LIMIT {}",
                vec_str,
                scope_id.replace("'", "\\'"),
                limit
            )
        } else {
            format!(
                "SELECT * FROM cache_item \
                 WHERE scope_id = '{}' \
                 ORDER BY importance DESC, updated_at DESC \
                 LIMIT {}",
                scope_id.replace("'", "\\'"),
                limit
            )
        };

        let mut response = self.db.client.query(&query).await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let results: Vec<Value> = response.take(0)
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let items = results.into_iter().filter_map(|obj| {
            Some(CacheItem {
                id: obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                scope_id: obj.get("scope_id").and_then(|v| v.as_str())?.to_string(),
                artifact_id: obj.get("artifact_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                kind: match obj.get("kind").and_then(|v| v.as_str())? {
                    "fact" => CacheItemKind::Fact,
                    "decision" => CacheItemKind::Decision,
                    "snippet" => CacheItemKind::Snippet,
                    "warning" => CacheItemKind::Warning,
                    _ => return None,
                },
                preview: obj.get("preview").and_then(|v| v.as_str())?.to_string(),
                facts: obj.get("facts")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default(),
                embedding: None, // Don't include in pack
                importance: obj.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                access_count: obj.get("access_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                provenance: obj.get("provenance").cloned().unwrap_or(Value::Object(Default::default())),
            })
        }).collect();

        Ok(items)
    }

    fn build_pack(
        &self,
        scope_id: &str,
        frame: &Option<CacheFrame>,
        items: Vec<CacheItem>,
        token_budget: usize,
        is_fresh: bool,
    ) -> MemoryPack {
        let mut pack = MemoryPack {
            scope_id: scope_id.to_string(),
            summary: frame.as_ref().map(|f| f.summary.clone()).unwrap_or_default(),
            facts: Vec::new(),
            decisions: Vec::new(),
            snippets: Vec::new(),
            warnings: Vec::new(),
            artifact_pointers: Vec::new(),
            token_count: 0,
            version: frame.as_ref().map(|f| f.version).unwrap_or(0),
            is_fresh,
        };

        // Reserve ~20% for summary
        let summary_budget = token_budget / 5;
        let items_budget = token_budget - summary_budget;

        // Truncate summary if needed
        if Self::estimate_tokens(&pack.summary) > summary_budget {
            let max_chars = summary_budget * 4;
            pack.summary = pack.summary.chars().take(max_chars).collect();
        }
        pack.token_count = Self::estimate_tokens(&pack.summary);

        // Allocate remaining budget across item types (40% facts, 30% decisions, 20% snippets, 10% warnings)
        let mut remaining = items_budget;

        for item in items {
            let item_tokens = Self::estimate_tokens(&item.preview)
                + item.facts.iter().map(|f| Self::estimate_tokens(f)).sum::<usize>();

            if item_tokens > remaining {
                continue;
            }

            remaining -= item_tokens;
            pack.token_count += item_tokens;

            if let Some(ref artifact_id) = item.artifact_id {
                if !pack.artifact_pointers.contains(artifact_id) {
                    pack.artifact_pointers.push(artifact_id.clone());
                }
            }

            match item.kind {
                CacheItemKind::Fact => pack.facts.push(item),
                CacheItemKind::Decision => pack.decisions.push(item),
                CacheItemKind::Snippet => pack.snippets.push(item),
                CacheItemKind::Warning => pack.warnings.push(item),
            }
        }

        pack
    }

    async fn update_frame(&self, scope_id: &str, pack: &MemoryPack) -> Result<(), CacheError> {
        let ttl = Utc::now() + ChronoDuration::minutes(self.default_ttl_minutes);
        let query = format!(
            "UPSERT cache_frame SET \
             scope_id = '{}', \
             summary = '{}', \
             version = version + 1, \
             token_count = {}, \
             updated_at = time::now(), \
             ttl_expires_at = '{}' \
             WHERE scope_id = '{}'",
            scope_id.replace("'", "\\'"),
            pack.summary.replace("'", "\\'"),
            pack.token_count,
            ttl.to_rfc3339(),
            scope_id.replace("'", "\\'")
        );

        self.db.client.query(&query).await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Write new cache items with semantic dedup
    pub async fn write_items(
        &self,
        scope_id: &str,
        items: Vec<CacheItem>,
    ) -> Result<usize, CacheError> {
        let mut written = 0;

        for item in items {
            // Generate embedding for dedup check
            let embedding = if self.embedding_service.is_enabled() {
                self.embedding_service.generate_embedding(&item.preview).await.ok()
            } else {
                None
            };

            // Check for semantic duplicates (similarity > 0.92)
            if let Some(ref emb) = embedding {
                let vec_str = emb.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
                let dedup_query = format!(
                    "SELECT id, vector::similarity::cosine(embedding, [{}]) AS sim \
                     FROM cache_item \
                     WHERE scope_id = '{}' AND embedding IS NOT NONE \
                     ORDER BY sim DESC LIMIT 1",
                    vec_str,
                    scope_id.replace("'", "\\'")
                );

                let mut response = self.db.client.query(&dedup_query).await
                    .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

                let results: Vec<Value> = response.take(0)
                    .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

                if let Some(existing) = results.first() {
                    if let Some(sim) = existing.get("sim").and_then(|v| v.as_f64()) {
                        if sim > 0.92 {
                            // Update existing instead of creating new
                            if let Some(existing_id) = existing.get("id").and_then(|v| v.as_str()) {
                                let update_query = format!(
                                    "UPDATE {} SET importance = importance + 0.1, access_count = access_count + 1, updated_at = time::now()",
                                    existing_id
                                );
                                self.db.client.query(&update_query).await
                                    .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
                                continue;
                            }
                        }
                    }
                }
            }

            // Insert new item
            let ttl = Utc::now() + ChronoDuration::minutes(self.default_ttl_minutes);
            let kind_str = match item.kind {
                CacheItemKind::Fact => "fact",
                CacheItemKind::Decision => "decision",
                CacheItemKind::Snippet => "snippet",
                CacheItemKind::Warning => "warning",
            };

            let embedding_str = embedding.as_ref()
                .map(|e| format!("[{}]", e.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ")))
                .unwrap_or_else(|| "NONE".to_string());

            let facts_str = serde_json::to_string(&item.facts).unwrap_or("[]".to_string());
            let provenance_str = serde_json::to_string(&item.provenance).unwrap_or("{}".to_string());

            let insert_query = format!(
                "CREATE cache_item SET \
                 scope_id = '{}', \
                 artifact_id = {}, \
                 kind = '{}', \
                 preview = '{}', \
                 facts = {}, \
                 embedding = {}, \
                 importance = {}, \
                 provenance = {}, \
                 ttl_expires_at = '{}'",
                scope_id.replace("'", "\\'"),
                item.artifact_id.as_ref().map(|id| format!("'{}'", id.replace("'", "\\'"))).unwrap_or("NONE".to_string()),
                kind_str,
                item.preview.replace("'", "\\'"),
                facts_str,
                embedding_str,
                item.importance,
                provenance_str,
                ttl.to_rfc3339()
            );

            self.db.client.query(&insert_query).await
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

            written += 1;
        }

        Ok(written)
    }

    /// Garbage collect expired items
    pub async fn gc(&self) -> Result<usize, CacheError> {
        let now = Utc::now().to_rfc3339();

        let query = format!(
            "DELETE FROM cache_item WHERE ttl_expires_at IS NOT NONE AND ttl_expires_at < '{}'; \
             DELETE FROM cache_frame WHERE ttl_expires_at IS NOT NONE AND ttl_expires_at < '{}';",
            now, now
        );

        self.db.client.query(&query).await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        Ok(0) // SurrealDB doesn't return delete count easily
    }
}
```

---

### UPDATE amp/server/src/services/mod.rs

- **IMPLEMENT**: Add cache module export
- **PATTERN**: Follow existing module pattern (line 1-10)
- **VALIDATE**: `cargo build -p amp-server`

```rust
pub mod cache;  // Add this line
```

---

### CREATE amp/mcp-server/src/tools/cache.rs

- **IMPLEMENT**: MCP tools for cache get/write
- **PATTERN**: Mirror context.rs structure (lines 1-105)
- **IMPORTS**:
  ```rust
  use anyhow::Result;
  use rmcp::model::Content;
  use schemars::JsonSchema;
  use serde::{Deserialize, Serialize};
  ```
- **VALIDATE**: `cargo build -p amp-mcp-server`

```rust
use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpCacheGetInput {
    /// Scope ID (e.g., "project:amp", "task:fix-auth")
    pub scope_id: String,
    /// Maximum tokens for the memory pack (default: 600)
    #[serde(default)]
    pub token_budget: Option<usize>,
    /// Optional query for semantic relevance scoring
    #[serde(default)]
    pub query: Option<String>,
    /// Version for delta pack (only items since this version)
    #[serde(default)]
    pub since_version: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpCacheWriteInput {
    /// Scope ID to write to
    pub scope_id: String,
    /// Items to write
    pub items: Vec<CacheItemInput>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CacheItemInput {
    /// Type: fact, decision, snippet, warning
    pub kind: String,
    /// Short preview text
    pub preview: String,
    /// Atomic facts extracted
    #[serde(default)]
    pub facts: Vec<String>,
    /// Source artifact ID
    #[serde(default)]
    pub artifact_id: Option<String>,
    /// Importance score 0.0-1.0
    #[serde(default)]
    pub importance: Option<f32>,
}

pub async fn handle_cache_get(
    client: &crate::amp_client::AmpClient,
    input: AmpCacheGetInput,
) -> Result<Vec<Content>> {
    let token_budget = input.token_budget.unwrap_or(600);

    // Build request to server
    let mut request = serde_json::json!({
        "scope_id": input.scope_id,
        "token_budget": token_budget,
    });

    if let Some(query) = &input.query {
        request["query"] = serde_json::Value::String(query.clone());
    }

    if let Some(version) = input.since_version {
        request["since_version"] = serde_json::Value::Number(version.into());
    }

    // Call server endpoint
    let result = client.post("/v1/cache/pack", request).await?;

    // Format response
    let summary = format_memory_pack(&result, &input)?;
    Ok(vec![Content::text(summary)])
}

fn format_memory_pack(result: &serde_json::Value, input: &AmpCacheGetInput) -> Result<String> {
    let mut output = format!("Memory Pack for scope: {}\n", input.scope_id);
    output.push_str("─".repeat(50).as_str());
    output.push('\n');

    if let Some(summary) = result.get("summary").and_then(|v| v.as_str()) {
        if !summary.is_empty() {
            output.push_str(&format!("Summary: {}\n\n", summary));
        }
    }

    // Facts
    if let Some(facts) = result.get("facts").and_then(|v| v.as_array()) {
        if !facts.is_empty() {
            output.push_str("Facts:\n");
            for fact in facts.iter().take(10) {
                if let Some(preview) = fact.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  • {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Decisions
    if let Some(decisions) = result.get("decisions").and_then(|v| v.as_array()) {
        if !decisions.is_empty() {
            output.push_str("Decisions:\n");
            for decision in decisions.iter().take(5) {
                if let Some(preview) = decision.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  ⚡ {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Snippets
    if let Some(snippets) = result.get("snippets").and_then(|v| v.as_array()) {
        if !snippets.is_empty() {
            output.push_str("Snippets:\n");
            for snippet in snippets.iter().take(3) {
                if let Some(preview) = snippet.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  📋 {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Warnings
    if let Some(warnings) = result.get("warnings").and_then(|v| v.as_array()) {
        if !warnings.is_empty() {
            output.push_str("Warnings:\n");
            for warning in warnings.iter().take(3) {
                if let Some(preview) = warning.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  ⚠️  {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Metadata
    if let Some(token_count) = result.get("token_count").and_then(|v| v.as_u64()) {
        output.push_str(&format!("Token count: {} / {}\n", token_count, input.token_budget.unwrap_or(600)));
    }

    if let Some(version) = result.get("version").and_then(|v| v.as_u64()) {
        output.push_str(&format!("Version: {}\n", version));
    }

    Ok(output)
}

pub async fn handle_cache_write(
    client: &crate::amp_client::AmpClient,
    input: AmpCacheWriteInput,
) -> Result<Vec<Content>> {
    let request = serde_json::json!({
        "scope_id": input.scope_id,
        "items": input.items,
    });

    let result = client.post("/v1/cache/write", request).await?;

    let written = result.get("written").and_then(|v| v.as_u64()).unwrap_or(0);
    let merged = result.get("merged").and_then(|v| v.as_u64()).unwrap_or(0);

    Ok(vec![Content::text(format!(
        "Cache write complete: {} items written, {} merged with existing",
        written, merged
    ))])
}
```

---

### UPDATE amp/mcp-server/src/tools/mod.rs

- **IMPLEMENT**: Add cache module export
- **PATTERN**: Line 1-6 pattern
- **VALIDATE**: `cargo build -p amp-mcp-server`

Add:
```rust
pub mod cache;
```

---

### UPDATE amp/mcp-server/src/main.rs

- **IMPLEMENT**: Register amp_cache_get and amp_cache_write tools
- **PATTERN**: Follow lines 62-103 for Tool definition, lines 241-268 for dispatch
- **GOTCHA**: Add to both list_tools and call_tool match arms
- **VALIDATE**: `cargo build -p amp-mcp-server`

In `list_tools` function, add after existing tools:
```rust
Tool {
    name: "amp_cache_get".into(),
    description: Some("Get memory pack for scope with token budget".into()),
    input_schema: to_schema(schemars::schema_for!(tools::cache::AmpCacheGetInput)),
    annotations: None,
    icons: None,
    meta: None,
    title: None,
    output_schema: None,
},
Tool {
    name: "amp_cache_write".into(),
    description: Some("Write facts/decisions/snippets to cache".into()),
    input_schema: to_schema(schemars::schema_for!(tools::cache::AmpCacheWriteInput)),
    annotations: None,
    icons: None,
    meta: None,
    title: None,
    output_schema: None,
},
```

In `call_tool` match, add:
```rust
"amp_cache_get" => {
    let input: tools::cache::AmpCacheGetInput =
        serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
            .map_err(to_invalid_params)?;
    tools::cache::handle_cache_get(client, input).await.map_err(to_internal_error)?
}
"amp_cache_write" => {
    let input: tools::cache::AmpCacheWriteInput =
        serde_json::from_value(serde_json::to_value(params.arguments).unwrap())
            .map_err(to_invalid_params)?;
    tools::cache::handle_cache_write(client, input).await.map_err(to_internal_error)?
}
```

---

### CREATE amp/server/src/handlers/cache.rs

- **IMPLEMENT**: HTTP handlers for cache endpoints
- **PATTERN**: Follow handlers/artifacts.rs structure
- **VALIDATE**: `cargo build -p amp-server`

```rust
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::services::cache::{CacheService, CacheItem, CacheItemKind};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GetPackRequest {
    pub scope_id: String,
    #[serde(default = "default_token_budget")]
    pub token_budget: usize,
    pub query: Option<String>,
    pub since_version: Option<u64>,
}

fn default_token_budget() -> usize { 600 }

#[derive(Debug, Deserialize)]
pub struct WriteItemsRequest {
    pub scope_id: String,
    pub items: Vec<WriteItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct WriteItemInput {
    pub kind: String,
    pub preview: String,
    #[serde(default)]
    pub facts: Vec<String>,
    pub artifact_id: Option<String>,
    #[serde(default = "default_importance")]
    pub importance: f32,
}

fn default_importance() -> f32 { 0.5 }

#[derive(Debug, Serialize)]
pub struct WriteResponse {
    pub written: usize,
    pub merged: usize,
}

pub async fn get_pack(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GetPackRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Generate query embedding if provided
    let query_embedding = if let Some(ref query) = request.query {
        if state.embedding_service.is_enabled() {
            state.embedding_service.generate_embedding(query).await.ok()
        } else {
            None
        }
    } else {
        None
    };

    let pack = state.cache_service
        .get_pack(&request.scope_id, request.token_budget, query_embedding.as_deref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::to_value(pack).unwrap()))
}

pub async fn write_items(
    State(state): State<Arc<AppState>>,
    Json(request): Json<WriteItemsRequest>,
) -> Result<Json<WriteResponse>, (StatusCode, String)> {
    let items: Vec<CacheItem> = request.items.into_iter().filter_map(|input| {
        let kind = match input.kind.as_str() {
            "fact" => CacheItemKind::Fact,
            "decision" => CacheItemKind::Decision,
            "snippet" => CacheItemKind::Snippet,
            "warning" => CacheItemKind::Warning,
            _ => return None,
        };

        Some(CacheItem {
            id: None,
            scope_id: request.scope_id.clone(),
            artifact_id: input.artifact_id,
            kind,
            preview: input.preview,
            facts: input.facts,
            embedding: None,
            importance: input.importance,
            access_count: 0,
            provenance: serde_json::json!({}),
        })
    }).collect();

    let written = state.cache_service
        .write_items(&request.scope_id, items)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(WriteResponse {
        written,
        merged: 0, // TODO: track merges
    }))
}
```

---

### UPDATE amp/server/src/main.rs

- **IMPLEMENT**: Register cache routes and add CacheService to AppState
- **PATTERN**: Follow existing route registration
- **GOTCHA**: Add cache_service to AppState struct
- **VALIDATE**: `cargo run -p amp-server`

Add to AppState:
```rust
pub cache_service: Arc<CacheService>,
```

Add to router:
```rust
.route("/v1/cache/pack", post(handlers::cache::get_pack))
.route("/v1/cache/write", post(handlers::cache::write_items))
```

Initialize CacheService after embedding_service:
```rust
let cache_service = Arc::new(CacheService::new(
    db.clone(),
    embedding_service.clone(),
));
```

---

## TESTING STRATEGY

### Unit Tests

Test in `amp/server/src/services/cache.rs`:
- `test_estimate_tokens` - verify chars/4 heuristic
- `test_build_pack_under_budget` - pack respects token limit
- `test_semantic_dedup` - similar items merge

### Integration Tests

- Test cache hit after write
- Test TTL expiration via GC
- Test amp_cache_get MCP tool end-to-end

### Edge Cases

- Empty scope (no items) returns empty pack
- Token budget of 0 returns minimal pack
- Extremely long preview text truncated
- Missing embeddings fallback to recency sort

---

## VALIDATION COMMANDS

### Level 1: Syntax & Style

```bash
cd amp && cargo fmt --all --check
cd amp && cargo clippy -p amp-server -p amp-mcp-server -- -D warnings
```

### Level 2: Unit Tests

```bash
cd amp && cargo test -p amp-server
cd amp && cargo test -p amp-mcp-server
```

### Level 3: Build & Run

```bash
cd amp && cargo build --release -p amp-server -p amp-mcp-server
cd amp && cargo run -p amp-server
```

### Level 4: Manual Validation

```bash
# Start server
cd amp && cargo run -p amp-server

# Test cache write (in another terminal)
curl -X POST http://localhost:8105/v1/cache/write \
  -H "Content-Type: application/json" \
  -d '{"scope_id":"project:test","items":[{"kind":"fact","preview":"The auth system uses JWT tokens","facts":["JWT with RS256","Tokens expire in 24h"]}]}'

# Test cache get
curl -X POST http://localhost:8105/v1/cache/pack \
  -H "Content-Type: application/json" \
  -d '{"scope_id":"project:test","token_budget":600}'
```

### Level 5: MCP Tool Validation

```bash
# With MCP inspector
npx @modelcontextprotocol/inspector cargo run -p amp-mcp-server
```

---

## ACCEPTANCE CRITERIA

- [x] cache_frame and cache_item tables defined in schema
- [ ] CacheService implements get_pack with token budgeting
- [ ] CacheService implements write_items with semantic dedup
- [ ] amp_cache_get MCP tool returns formatted memory packs
- [ ] amp_cache_write MCP tool stores items with embeddings
- [ ] Token budget enforced (pack.token_count <= budget)
- [ ] Fresh cache (< 5 min old) served without rebuild
- [ ] All validation commands pass with zero errors
- [ ] No regressions in existing functionality

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit + integration)
- [ ] No linting or type checking errors
- [ ] Manual testing confirms feature works
- [ ] Acceptance criteria all met
- [ ] Code reviewed for quality and maintainability

---

## NOTES

### Design Decisions

1. **Token estimation uses chars/4**: Fast heuristic with ~10% error. Exact tiktoken would add dependency and latency.

2. **TTL default 30 minutes**: Balances freshness with recomputation cost. Configurable via CacheService constructor.

3. **Semantic dedup threshold 0.92**: High enough to avoid false merges, low enough to catch paraphrases.

4. **Pack structure prioritizes facts**: 40% budget to facts reflects their highest utility for context.

### Future Enhancements (Not in This Plan)

- Episode integration (consolidate runs into episodes)
- Delta packs (only changes since version)
- AI-assisted pack compression
- Cache promotion on artifact write
- RRF integration (add cache hits as 4th scoring signal)

### Risk Mitigation

- If embedding service disabled, fallback to recency sort
- If database query fails, return empty pack (graceful degradation)
- If token budget exceeded, truncate from lowest importance items
