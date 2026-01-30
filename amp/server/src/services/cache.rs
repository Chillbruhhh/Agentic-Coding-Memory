#![allow(dead_code)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use thiserror::Error;

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
    pub fn new(db: Arc<Database>, embedding_service: Arc<dyn EmbeddingService>) -> Self {
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
        let is_fresh = frame
            .as_ref()
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
        if !is_fresh && !pack.facts.is_empty() || !pack.decisions.is_empty() {
            let _ = self.update_frame(scope_id, &pack).await;
        }

        Ok(pack)
    }

    async fn get_frame(&self, scope_id: &str) -> Result<Option<CacheFrame>, CacheError> {
        let query = format!(
            "SELECT VALUE {{ \
             scope_id: scope_id, \
             summary: summary, \
             version: version, \
             token_count: token_count, \
             updated_at: string::concat(updated_at) \
             }} FROM cache_frame WHERE scope_id = '{}' LIMIT 1",
            scope_id.replace('\'', "\\'")
        );

        let mut response = self
            .db
            .client
            .query(&query)
            .await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?
            .check()
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let results: Vec<Value> = response
            .take(0)
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        if let Some(obj) = results.first() {
            Ok(Some(CacheFrame {
                scope_id: obj
                    .get("scope_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                summary: obj
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                version: obj.get("version").and_then(|v| v.as_u64()).unwrap_or(0),
                token_count: obj.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                updated_at: obj
                    .get("updated_at")
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
            let vec_str = embedding
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "SELECT \
                 string::concat(id) AS id, \
                 scope_id, \
                 artifact_id, \
                 kind, \
                 preview, \
                 facts, \
                 importance, \
                 access_count, \
                 provenance, \
                 vector::similarity::cosine(embedding, [{}]) AS sim \
                 FROM cache_item \
                 WHERE scope_id = '{}' AND embedding IS NOT NONE \
                 ORDER BY sim DESC, importance DESC \
                 LIMIT {}",
                vec_str,
                scope_id.replace('\'', "\\'"),
                limit
            )
        } else {
            format!(
                "SELECT \
                 string::concat(id) AS id, \
                 scope_id, \
                 artifact_id, \
                 kind, \
                 preview, \
                 facts, \
                 importance, \
                 access_count, \
                 provenance, \
                 string::concat(updated_at) AS updated_at \
                 FROM cache_item \
                 WHERE scope_id = '{}' \
                 ORDER BY importance DESC, updated_at DESC \
                 LIMIT {}",
                scope_id.replace('\'', "\\'"),
                limit
            )
        };

        let mut response = self
            .db
            .client
            .query(&query)
            .await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?
            .check()
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let results: Vec<Value> = response
            .take(0)
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let items = results
            .into_iter()
            .filter_map(|obj| {
                Some(CacheItem {
                    id: obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    scope_id: obj.get("scope_id").and_then(|v| v.as_str())?.to_string(),
                    artifact_id: obj
                        .get("artifact_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    kind: match obj.get("kind").and_then(|v| v.as_str())? {
                        "fact" => CacheItemKind::Fact,
                        "decision" => CacheItemKind::Decision,
                        "snippet" => CacheItemKind::Snippet,
                        "warning" => CacheItemKind::Warning,
                        _ => return None,
                    },
                    preview: obj.get("preview").and_then(|v| v.as_str())?.to_string(),
                    facts: obj
                        .get("facts")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    embedding: None, // Don't include in pack
                    importance: obj
                        .get("importance")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.5) as f32,
                    access_count: obj
                        .get("access_count")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    provenance: obj
                        .get("provenance")
                        .cloned()
                        .unwrap_or(Value::Object(Default::default())),
                })
            })
            .collect();

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
            summary: frame
                .as_ref()
                .map(|f| f.summary.clone())
                .unwrap_or_default(),
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

        // Allocate remaining budget across item types
        let mut remaining = items_budget;

        for item in items {
            let item_tokens = Self::estimate_tokens(&item.preview)
                + item
                    .facts
                    .iter()
                    .map(|f| Self::estimate_tokens(f))
                    .sum::<usize>();

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
        let ttl_expr = format!("time::now() + {}m", self.default_ttl_minutes);

        // First try to update existing frame
        let exists_query = format!(
            "SELECT VALUE count() FROM cache_frame WHERE scope_id = '{}'",
            scope_id.replace('\'', "\\'")
        );

        let mut response = self
            .db
            .client
            .query(&exists_query)
            .await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?
            .check()
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let count: Vec<Value> = response
            .take(0)
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        let has_frame = count
            .get(0)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            > 0;

        let update_query = format!(
            "UPDATE cache_frame SET \
             summary = '{}', \
             version = version + 1, \
             token_count = {}, \
             updated_at = time::now(), \
             ttl_expires_at = {} \
             WHERE scope_id = '{}'",
            pack.summary.replace('\'', "\\'"),
            pack.token_count,
            ttl_expr,
            scope_id.replace('\'', "\\'")
        );

        if has_frame {
            self.db
                .client
                .query(&update_query)
                .await
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?
                .check()
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
        } else {
            let create_query = format!(
                "CREATE cache_frame SET \
                 scope_id = '{}', \
                 summary = '{}', \
                 version = 1, \
                 token_count = {}, \
                 updated_at = time::now(), \
                 ttl_expires_at = {}",
                scope_id.replace('\'', "\\'"),
                pack.summary.replace('\'', "\\'"),
                pack.token_count,
                ttl_expr
            );

            self.db
                .client
                .query(&create_query)
                .await
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?
                .check()
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
        }

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
                self.embedding_service
                    .generate_embedding(&item.preview)
                    .await
                    .ok()
            } else {
                None
            };

            // Check for semantic duplicates (similarity > 0.92)
            if let Some(ref emb) = embedding {
                let vec_str = emb
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let dedup_query = format!(
                    "SELECT string::concat(id) AS id, \
                     vector::similarity::cosine(embedding, [{}]) AS sim \
                     FROM cache_item \
                     WHERE scope_id = '{}' AND embedding IS NOT NONE \
                     ORDER BY sim DESC LIMIT 1",
                    vec_str,
                    scope_id.replace('\'', "\\'")
                );

                let mut response = self
                    .db
                    .client
                    .query(&dedup_query)
                    .await
                    .map_err(|e| CacheError::DatabaseError(e.to_string()))?
                    .check()
                    .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

                let results: Vec<Value> = response
                    .take(0)
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
                                self.db
                                    .client
                                    .query(&update_query)
                                    .await
                                    .map_err(|e| CacheError::DatabaseError(e.to_string()))?
                                    .check()
                                    .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
                                continue;
                            }
                        }
                    }
                }
            }

            // Insert new item
            let ttl_expr = format!("time::now() + {}m", self.default_ttl_minutes);
            let kind_str = match item.kind {
                CacheItemKind::Fact => "fact",
                CacheItemKind::Decision => "decision",
                CacheItemKind::Snippet => "snippet",
                CacheItemKind::Warning => "warning",
            };

            let embedding_str = embedding
                .as_ref()
                .map(|e| {
                    format!(
                        "[{}]",
                        e.iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .unwrap_or_else(|| "NONE".to_string());

            let facts_str = serde_json::to_string(&item.facts).unwrap_or_else(|_| "[]".to_string());
            let provenance_str =
                serde_json::to_string(&item.provenance).unwrap_or_else(|_| "{}".to_string());

            let artifact_id_str = item
                .artifact_id
                .as_ref()
                .map(|id| format!("'{}'", id.replace('\'', "\\'")))
                .unwrap_or_else(|| "NONE".to_string());

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
                 ttl_expires_at = {}",
                scope_id.replace('\'', "\\'"),
                artifact_id_str,
                kind_str,
                item.preview.replace('\'', "\\'"),
                facts_str,
                embedding_str,
                item.importance,
                provenance_str,
                ttl_expr
            );

            self.db
                .client
                .query(&insert_query)
                .await
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?
                .check()
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

        self.db
            .client
            .query(&query)
            .await
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?
            .check()
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        Ok(0) // SurrealDB doesn't return delete count easily
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(CacheService::estimate_tokens("hello"), 1); // 5 chars / 4 = 1
        assert_eq!(CacheService::estimate_tokens("hello world!"), 3); // 12 chars / 4 = 3
        assert_eq!(CacheService::estimate_tokens(""), 0);
    }

    #[test]
    fn test_cache_item_kind_serialization() {
        let kind = CacheItemKind::Fact;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"fact\"");

        let kind: CacheItemKind = serde_json::from_str("\"decision\"").unwrap();
        assert_eq!(kind, CacheItemKind::Decision);
    }
}
