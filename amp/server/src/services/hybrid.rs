use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::database::Database;
use crate::handlers::query::{GraphQuery, QueryFilters, QueryRequest, TraversalAlgorithm};
use crate::services::embedding::EmbeddingService;
use crate::services::graph::GraphTraversalService;
use crate::surreal_json::{normalize_object_ids, take_json_values};

#[derive(Debug, Error)]
pub enum HybridRetrievalError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Query timeout")]
    Timeout,

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Partial failure: {0}")]
    PartialFailure(String),
}

#[derive(Debug, Serialize)]
pub struct HybridResult {
    pub object: Value,
    pub total_score: f32,
    pub text_score: Option<f32>,
    pub vector_score: Option<f32>,
    pub graph_score: Option<f32>,
    pub explanation: String,
}

#[derive(Debug, Serialize)]
pub struct HybridResponse {
    pub results: Vec<HybridResult>,
    pub trace_id: Uuid,
    pub total_count: usize,
    pub execution_time_ms: u64,
    pub text_results_count: usize,
    pub vector_results_count: usize,
    pub graph_results_count: usize,
}

pub struct HybridRetrievalService {
    db: Arc<Database>,
    embedding_service: Arc<dyn EmbeddingService>,
    graph_service: Arc<GraphTraversalService>,
}

const DEFAULT_GRAPH_MAX_DEPTH: usize = 1;
const DEFAULT_GRAPH_CAP: usize = 50;
const DEFAULT_GRAPH_RELATIONS: [&str; 5] = [
    "depends_on",
    "calls",
    "implements",
    "modifies",
    "defined_in",
];

impl HybridRetrievalService {
    pub fn new(
        db: Arc<Database>,
        embedding_service: Arc<dyn EmbeddingService>,
        graph_service: Arc<GraphTraversalService>,
    ) -> Self {
        Self {
            db,
            embedding_service,
            graph_service,
        }
    }

    pub async fn execute_hybrid_query(
        &self,
        request: &QueryRequest,
    ) -> Result<HybridResponse, HybridRetrievalError> {
        let start_time = std::time::Instant::now();
        let trace_id = Uuid::new_v4();

        tracing::info!(
            "Executing hybrid query: trace_id={}, text={:?}, has_vector={}, has_graph={}",
            trace_id,
            request.text,
            request.vector.is_some(),
            request.graph.is_some()
        );

        // Execute queries (allow autoseed to run graph after text/vector)
        let hybrid_timeout = Duration::from_secs(15);

        // Autoseed mode: graph_autoseed=true, regardless of whether request.graph has overrides.
        // When graph has start_nodes, that's explicit graph mode (no autoseed).
        let use_autoseed = request.graph_autoseed.unwrap_or(false)
            && !request
                .graph
                .as_ref()
                .map(|g| !g.start_nodes.is_empty())
                .unwrap_or(false);

        let (text_results, vector_results, mut graph_results) = if use_autoseed {
            let query_results = timeout(hybrid_timeout, async {
                tokio::try_join!(
                    self.execute_text_search(request),
                    self.execute_vector_search(request)
                )
            })
            .await;

            let (text_results, vector_results) = match query_results {
                Ok(Ok(results)) => results,
                Ok(Err(e)) => {
                    tracing::error!("Hybrid query failed: {}", e);
                    return Err(e);
                }
                Err(_) => {
                    tracing::error!("Hybrid query timeout");
                    return Err(HybridRetrievalError::Timeout);
                }
            };

            let mut seeded_request = request.clone();
            let autoseed_query = self.build_autoseed_graph_query(
                &text_results,
                &vector_results,
                request.graph.as_ref(),
            );
            if autoseed_query.is_some() {
                seeded_request.graph = autoseed_query;
            }

            let graph_results = if seeded_request.graph.is_some() {
                self.execute_graph_search(&seeded_request).await?
            } else {
                Vec::new()
            };

            (text_results, vector_results, graph_results)
        } else {
            let query_results = timeout(hybrid_timeout, async {
                tokio::try_join!(
                    self.execute_text_search(request),
                    self.execute_vector_search(request),
                    self.execute_graph_search(request)
                )
            })
            .await;

            match query_results {
                Ok(Ok(results)) => results,
                Ok(Err(e)) => {
                    tracing::error!("Hybrid query failed: {}", e);
                    return Err(e);
                }
                Err(_) => {
                    tracing::error!("Hybrid query timeout");
                    return Err(HybridRetrievalError::Timeout);
                }
            }
        };

        if request.graph_intersect.unwrap_or(false) {
            let mut text_vector_ids: HashSet<String> = HashSet::new();
            for (obj, _, _) in text_results.iter().chain(vector_results.iter()) {
                if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                    text_vector_ids.insert(id.to_string());
                }
            }

            graph_results.retain(|(obj, _, _)| {
                obj.get("id")
                    .and_then(|v| v.as_str())
                    .map(|id| text_vector_ids.contains(id))
                    .unwrap_or(false)
            });
        }

        // Capture counts before merge
        let text_count = text_results.len();
        let vector_count = vector_results.len();
        let graph_count = graph_results.len();

        tracing::info!(
            "Pre-merge counts: text={}, vector={}, graph={}",
            text_count,
            vector_count,
            graph_count
        );

        // Merge and deduplicate results
        let merged_results = self.merge_results(text_results, vector_results, graph_results);

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            "Hybrid query complete: trace_id={}, merged={}, time={}ms",
            trace_id,
            merged_results.len(),
            execution_time_ms
        );

        Ok(HybridResponse {
            total_count: merged_results.len(),
            results: merged_results,
            trace_id,
            execution_time_ms,
            text_results_count: text_count,
            vector_results_count: vector_count,
            graph_results_count: graph_count,
        })
    }

    async fn execute_text_search(
        &self,
        request: &QueryRequest,
    ) -> Result<Vec<(Value, f32, String)>, HybridRetrievalError> {
        if request.text.is_none() {
            return Ok(Vec::new());
        }

        let query_str = self.build_text_query_string(request);

        tracing::debug!("Executing text search: {}", query_str);

        let query_result = timeout(
            Duration::from_secs(10),
            self.db.client.query(query_str.clone()),
        )
        .await;

        match query_result {
            Ok(Ok(mut response)) => {
                let mut results = take_json_values(&mut response, 0);
                tracing::info!("Text search returned {} raw results", results.len());
                normalize_object_ids(&mut results);

                let scored_results: Vec<_> = results
                    .into_iter()
                    .map(|obj| {
                        let score = self.calculate_text_score(&obj, request.text.as_ref());
                        let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                        tracing::debug!("Text result: id={}, score={:.4}", id, score);
                        let explanation = format!(
                            "Text match for '{}'",
                            request.text.as_ref().unwrap_or(&"".to_string())
                        );
                        (obj, score, explanation)
                    })
                    .collect();

                Ok(scored_results)
            }
            Ok(Err(e)) => {
                tracing::error!("Text search failed: {}", e);
                Err(HybridRetrievalError::DatabaseError(e.to_string()))
            }
            Err(_) => {
                tracing::warn!("Text search timeout");
                Ok(Vec::new()) // Graceful degradation
            }
        }
    }

    async fn execute_vector_search(
        &self,
        request: &QueryRequest,
    ) -> Result<Vec<(Value, f32, String)>, HybridRetrievalError> {
        tracing::info!(
            "execute_vector_search: has_vector={}, has_text={}, embedding_enabled={}",
            request.vector.is_some(),
            request.text.is_some(),
            self.embedding_service.is_enabled()
        );

        let query_vector = if let Some(vector) = &request.vector {
            tracing::info!("Using provided vector of {} dimensions", vector.len());
            Some(vector.clone())
        } else if let Some(text) = &request.text {
            if self.embedding_service.is_enabled() {
                tracing::info!("Generating embedding for text: '{}'", text);
                match self.embedding_service.generate_embedding(text).await {
                    Ok(vec) => {
                        tracing::info!("Generated embedding: {} dimensions", vec.len());
                        Some(vec)
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate embedding: {}", e);
                        None
                    }
                }
            } else {
                tracing::info!("Embedding service disabled, skipping vector generation");
                None
            }
        } else {
            tracing::info!("No text or vector provided");
            None
        };

        if query_vector.is_none() {
            return Ok(Vec::new());
        }

        let vector = query_vector.unwrap();
        let query_str = self.build_vector_query_string(request, &vector);

        tracing::info!(
            "Executing vector search with {} dimension vector",
            vector.len()
        );
        tracing::debug!(
            "Vector query (first 500 chars): {}",
            &query_str[..query_str.len().min(500)]
        );

        let query_result = timeout(
            Duration::from_secs(10),
            self.db.client.query(query_str.clone()),
        )
        .await;

        match query_result {
            Ok(Ok(mut response)) => {
                tracing::info!("Vector search query succeeded");
                let mut results = take_json_values(&mut response, 0);
                tracing::info!("Vector search returned {} raw results", results.len());
                if results.is_empty() {
                    tracing::warn!(
                        "Vector search returned empty results (vector_dim={}, has_text={}, embedding_enabled={}, query_len={})",
                        vector.len(),
                        request.text.is_some(),
                        self.embedding_service.is_enabled(),
                        query_str.len()
                    );
                }
                normalize_object_ids(&mut results);

                let scored_results: Vec<_> = results
                    .into_iter()
                    .map(|obj| {
                        let score = obj
                            .get("similarity")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                        tracing::debug!("Vector result: id={}, similarity={:.4}", id, score);
                        let explanation = "Vector similarity match".to_string();
                        (obj, score, explanation)
                    })
                    .collect();

                Ok(scored_results)
            }
            Ok(Err(e)) => {
                tracing::error!(
                    "Vector search failed: {} (query_len={})",
                    e,
                    query_str.len()
                );
                Err(HybridRetrievalError::DatabaseError(e.to_string()))
            }
            Err(_) => {
                tracing::warn!("Vector search timeout");
                Ok(Vec::new()) // Graceful degradation
            }
        }
    }

    async fn execute_graph_search(
        &self,
        request: &QueryRequest,
    ) -> Result<Vec<(Value, f32, String)>, HybridRetrievalError> {
        if request.graph.is_none() {
            return Ok(Vec::new());
        }

        let graph_query = self.apply_graph_defaults(request.graph.as_ref().unwrap());

        tracing::info!(
            "Executing graph search: start_nodes={}, relation_types={:?}, max_depth={:?}, direction={:?}, algorithm={:?}",
            graph_query.start_nodes.len(),
            graph_query.relation_types,
            graph_query.max_depth,
            graph_query.direction,
            graph_query.algorithm
        );

        match self.graph_service.execute_multi_hop(&graph_query).await {
            Ok(traversal_result) => {
                tracing::info!(
                    "Graph traversal returned {} nodes",
                    traversal_result.nodes.len()
                );
                let relation_weight = self.graph_relation_weight(&graph_query.relation_types);
                let mut scored_results: Vec<_> = traversal_result
                    .nodes
                    .into_iter()
                    .filter_map(|obj| {
                        let id = obj.get("id").and_then(|v| v.as_str())?;
                        let depth = traversal_result
                            .node_depths
                            .as_ref()
                            .and_then(|map| map.get(id).copied())
                            .unwrap_or(DEFAULT_GRAPH_MAX_DEPTH);
                        let score = relation_weight / (1.0 + depth as f32);
                        let explanation = format!(
                            "Graph traversal match (depth={}, weight={:.2})",
                            depth, relation_weight
                        );
                        Some((obj, score, explanation))
                    })
                    .collect();

                scored_results
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                if scored_results.len() > DEFAULT_GRAPH_CAP {
                    scored_results.truncate(DEFAULT_GRAPH_CAP);
                }

                Ok(scored_results)
            }
            Err(e) => {
                tracing::warn!("Graph search failed: {}", e);
                Ok(Vec::new()) // Graceful degradation
            }
        }
    }

    fn merge_results(
        &self,
        text_results: Vec<(Value, f32, String)>,
        vector_results: Vec<(Value, f32, String)>,
        graph_results: Vec<(Value, f32, String)>,
    ) -> Vec<HybridResult> {
        // Reciprocal Rank Fusion (RRF) implementation
        // Formula: RRF(d) = Σ(1 / (k + rank_i(d))) for all retrieval systems
        const RRF_K: f32 = 60.0; // Standard RRF constant

        let mut result_map: HashMap<String, HybridResult> = HashMap::new();

        // Sort each result set by score (descending) to get proper ranks
        let mut sorted_text: Vec<_> = text_results.into_iter().enumerate().collect();
        sorted_text.sort_by(|a, b| {
            b.1 .1
                .partial_cmp(&a.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut sorted_vector: Vec<_> = vector_results.into_iter().enumerate().collect();
        sorted_vector.sort_by(|a, b| {
            b.1 .1
                .partial_cmp(&a.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut sorted_graph: Vec<_> = graph_results.into_iter().enumerate().collect();
        sorted_graph.sort_by(|a, b| {
            b.1 .1
                .partial_cmp(&a.1 .1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Process text results with RRF scoring
        for (rank, (obj, original_score, explanation)) in sorted_text.into_iter() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                let rrf_score = 1.0 / (RRF_K + (rank + 1) as f32);
                result_map.insert(
                    id.to_string(),
                    HybridResult {
                        object: obj,
                        total_score: rrf_score,
                        text_score: Some(original_score),
                        vector_score: None,
                        graph_score: None,
                        explanation: format!(
                            "Text(rank:{}, rrf:{:.4}): {}",
                            rank + 1,
                            rrf_score,
                            explanation
                        ),
                    },
                );
            }
        }

        // Process vector results with RRF scoring
        for (rank, (obj, original_score, explanation)) in sorted_vector.into_iter() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                let rrf_score = 1.0 / (RRF_K + (rank + 1) as f32);

                if let Some(existing) = result_map.get_mut(id) {
                    existing.total_score += rrf_score;
                    existing.vector_score = Some(original_score);
                    existing.explanation = format!(
                        "{} + Vector(rank:{}, rrf:{:.4}): {}",
                        existing.explanation,
                        rank + 1,
                        rrf_score,
                        explanation
                    );
                } else {
                    result_map.insert(
                        id.to_string(),
                        HybridResult {
                            object: obj,
                            total_score: rrf_score,
                            text_score: None,
                            vector_score: Some(original_score),
                            graph_score: None,
                            explanation: format!(
                                "Vector(rank:{}, rrf:{:.4}): {}",
                                rank + 1,
                                rrf_score,
                                explanation
                            ),
                        },
                    );
                }
            }
        }

        // Process graph results with RRF scoring
        for (rank, (obj, original_score, explanation)) in sorted_graph.into_iter() {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                let rrf_score = 1.0 / (RRF_K + (rank + 1) as f32);

                if let Some(existing) = result_map.get_mut(id) {
                    existing.total_score += rrf_score;
                    existing.graph_score = Some(original_score);
                    existing.explanation = format!(
                        "{} + Graph(rank:{}, rrf:{:.4}): {}",
                        existing.explanation,
                        rank + 1,
                        rrf_score,
                        explanation
                    );
                } else {
                    result_map.insert(
                        id.to_string(),
                        HybridResult {
                            object: obj,
                            total_score: rrf_score,
                            text_score: None,
                            vector_score: None,
                            graph_score: Some(original_score),
                            explanation: format!(
                                "Graph(rank:{}, rrf:{:.4}): {}",
                                rank + 1,
                                rrf_score,
                                explanation
                            ),
                        },
                    );
                }
            }
        }

        // Sort by RRF total score (descending) and return
        let mut results: Vec<HybridResult> = result_map.into_values().collect();
        results.sort_by(|a, b| {
            b.total_score
                .partial_cmp(&a.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    fn build_text_query_string(&self, request: &QueryRequest) -> String {
        let mut query = "SELECT VALUE { id: string::concat(id), type: type, tenant_id: tenant_id, project_id: project_id, name: name, kind: kind, path: path, language: language, signature: signature, documentation: documentation, provenance: provenance, links: links, embedding: embedding } FROM objects".to_string();
        let mut conditions = Vec::new();

        if let Some(text) = &request.text {
            let text_escaped = text.replace("'", "\\'");
            conditions.push(format!(
                "(name CONTAINS '{}' OR title CONTAINS '{}' OR description CONTAINS '{}' OR documentation CONTAINS '{}')",
                text_escaped, text_escaped, text_escaped, text_escaped
            ));
        }

        self.add_filter_conditions(&mut conditions, &request.filters);

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let limit = request.limit.unwrap_or(10);
        query.push_str(&format!(" LIMIT {}", limit));

        query
    }

    fn build_vector_query_string(&self, request: &QueryRequest, vector: &[f32]) -> String {
        let vector_str = vector
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let mut inner_query = "SELECT id, type, tenant_id, project_id, name, kind, path, language, signature, documentation, provenance, links, embedding FROM objects WHERE embedding IS NOT NONE AND embedding IS NOT NULL".to_string();

        let mut conditions = Vec::new();
        self.add_filter_conditions(&mut conditions, &request.filters);

        if !conditions.is_empty() {
            inner_query.push_str(" AND ");
            inner_query.push_str(&conditions.join(" AND "));
        }

        let limit = request.limit.unwrap_or(10);
        let inner_ranked_query = format!(
            "SELECT id, type, tenant_id, project_id, name, kind, path, language, signature, documentation, provenance, links, embedding, vector::similarity::cosine(embedding, [{}]) AS similarity FROM ({}) ORDER BY similarity DESC LIMIT {}",
            vector_str, inner_query, limit
        );

        format!(
            "SELECT VALUE {{ id: string::concat(id), type: type, tenant_id: tenant_id, project_id: project_id, name: name, kind: kind, path: path, language: language, signature: signature, documentation: documentation, provenance: provenance, links: links, embedding: embedding, similarity: similarity }} FROM ({})",
            inner_ranked_query
        )
    }

    fn apply_graph_defaults(&self, graph_query: &GraphQuery) -> GraphQuery {
        let mut graph_query = graph_query.clone();

        if graph_query.algorithm.is_none() {
            graph_query.algorithm = Some(TraversalAlgorithm::Collect);
        }

        if graph_query.max_depth.is_none() {
            graph_query.max_depth = Some(DEFAULT_GRAPH_MAX_DEPTH);
        }

        if graph_query.relation_types.is_none() {
            graph_query.relation_types = Some(
                DEFAULT_GRAPH_RELATIONS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            );
        }

        graph_query
    }

    fn graph_relation_weight(&self, relation_types: &Option<Vec<String>>) -> f32 {
        let default_weight = 0.5;
        let weights = |rel: &str| -> f32 {
            match rel {
                "depends_on" => 1.0,
                "defined_in" => 0.95,
                "calls" => 0.9,
                "implements" => 0.85,
                "modifies" => 0.8,
                "justified_by" => 0.75,
                "produced" => 0.7,
                _ => default_weight,
            }
        };

        match relation_types {
            Some(types) if !types.is_empty() => {
                let total: f32 = types.iter().map(|t| weights(t)).sum();
                total / types.len() as f32
            }
            _ => {
                let total: f32 = DEFAULT_GRAPH_RELATIONS.iter().map(|t| weights(t)).sum();
                total / DEFAULT_GRAPH_RELATIONS.len() as f32
            }
        }
    }

    fn build_autoseed_graph_query(
        &self,
        text_results: &[(Value, f32, String)],
        vector_results: &[(Value, f32, String)],
        overrides: Option<&GraphQuery>,
    ) -> Option<GraphQuery> {
        let mut ids: Vec<Uuid> = Vec::new();
        let mut seen = HashSet::new();

        for (obj, _, _) in text_results.iter().chain(vector_results.iter()) {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                if let Ok(uuid) = Uuid::parse_str(id.trim_start_matches("objects:")) {
                    if seen.insert(uuid) {
                        ids.push(uuid);
                    }
                }
            }
            if ids.len() >= 10 {
                break;
            }
        }

        if ids.is_empty() {
            return None;
        }

        // Use agent-provided overrides for depth/relation_types/direction,
        // falling back to defaults when not specified.
        Some(GraphQuery {
            start_nodes: ids,
            relation_types: overrides
                .and_then(|o| o.relation_types.clone())
                .or_else(|| {
                    Some(
                        DEFAULT_GRAPH_RELATIONS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    )
                }),
            max_depth: overrides
                .and_then(|o| o.max_depth)
                .or(Some(DEFAULT_GRAPH_MAX_DEPTH)),
            direction: overrides
                .and_then(|o| o.direction.clone())
                .or(Some(crate::handlers::query::GraphDirection::Both)),
            algorithm: Some(TraversalAlgorithm::Collect),
            target_node: None,
        })
    }

    fn add_filter_conditions(&self, conditions: &mut Vec<String>, filters: &Option<QueryFilters>) {
        if let Some(filters) = filters {
            if let Some(types) = &filters.object_types {
                let types_str = types
                    .iter()
                    .map(|t| format!("'{}'", t.replace("'", "\\'")))
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("type IN [{}]", types_str));
            }

            if let Some(kinds) = &filters.kind {
                let kinds_str = kinds
                    .iter()
                    .map(|k| format!("'{}'", k.replace("'", "\\'")))
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("kind IN [{}]", kinds_str));
            }

            if let Some(project_id) = &filters.project_id {
                conditions.push(format!("project_id = '{}'", project_id.replace("'", "\\'")));
            }

            if let Some(tenant_id) = &filters.tenant_id {
                conditions.push(format!("tenant_id = '{}'", tenant_id.replace("'", "\\'")));
            }
        }
    }

    fn calculate_text_score(&self, obj: &Value, text_query: Option<&String>) -> f32 {
        if text_query.is_none() {
            return 1.0;
        }

        let query = text_query.unwrap().to_lowercase();

        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
            if name.to_lowercase() == query {
                return 1.0;
            }
            if name.to_lowercase().contains(&query) {
                return 0.8;
            }
        }

        if let Some(title) = obj.get("title").and_then(|v| v.as_str()) {
            if title.to_lowercase() == query {
                return 1.0;
            }
            if title.to_lowercase().contains(&query) {
                return 0.8;
            }
        }

        0.6 // Default for other matches
    }
}
