use std::sync::Arc;
use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;
use tokio::time::{timeout, Duration};
use thiserror::Error;

use crate::database::Database;
use crate::services::embedding::EmbeddingService;
use crate::services::graph::GraphTraversalService;
use crate::handlers::query::{QueryRequest, QueryFilters};
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

    pub async fn execute_hybrid_query(&self, request: &QueryRequest) -> Result<HybridResponse, HybridRetrievalError> {
        let start_time = std::time::Instant::now();
        let trace_id = Uuid::new_v4();
        
        tracing::info!("Executing hybrid query: trace_id={}, text={:?}, has_vector={}, has_graph={}", 
            trace_id, request.text, request.vector.is_some(), request.graph.is_some());

        // Execute queries in parallel with timeout
        let hybrid_timeout = Duration::from_secs(5);
        
        let query_results = timeout(hybrid_timeout, async {
            tokio::try_join!(
                self.execute_text_search(request),
                self.execute_vector_search(request),
                self.execute_graph_search(request)
            )
        }).await;

        let (text_results, vector_results, graph_results) = match query_results {
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

        // Merge and deduplicate results
        let merged_results = self.merge_results(text_results, vector_results, graph_results);
        
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        tracing::info!("Hybrid query complete: trace_id={}, results={}, time={}ms", 
            trace_id, merged_results.len(), execution_time_ms);

        Ok(HybridResponse {
            total_count: merged_results.len(),
            results: merged_results,
            trace_id,
            execution_time_ms,
            text_results_count: 0, // Will be populated in merge_results
            vector_results_count: 0,
            graph_results_count: 0,
        })
    }

    async fn execute_text_search(&self, request: &QueryRequest) -> Result<Vec<(Value, f32, String)>, HybridRetrievalError> {
        if request.text.is_none() {
            return Ok(Vec::new());
        }

        let query_str = self.build_text_query_string(request);
        
        tracing::debug!("Executing text search: {}", query_str);
        
        let query_result = timeout(
            Duration::from_secs(3),
            self.db.client.query(query_str)
        ).await;
        
        match query_result {
            Ok(Ok(mut response)) => {
                let mut results = take_json_values(&mut response, 0);
                normalize_object_ids(&mut results);
                
                let scored_results = results.into_iter().map(|obj| {
                    let score = self.calculate_text_score(&obj, request.text.as_ref());
                    let explanation = format!("Text match for '{}'", request.text.as_ref().unwrap_or(&"".to_string()));
                    (obj, score, explanation)
                }).collect();
                
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

    async fn execute_vector_search(&self, request: &QueryRequest) -> Result<Vec<(Value, f32, String)>, HybridRetrievalError> {
        let query_vector = if let Some(vector) = &request.vector {
            Some(vector.clone())
        } else if let Some(text) = &request.text {
            if self.embedding_service.is_enabled() {
                match self.embedding_service.generate_embedding(text).await {
                    Ok(vec) => Some(vec),
                    Err(e) => {
                        tracing::warn!("Failed to generate embedding: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        if query_vector.is_none() {
            return Ok(Vec::new());
        }

        let vector = query_vector.unwrap();
        let query_str = self.build_vector_query_string(request, &vector);
        
        tracing::debug!("Executing vector search");
        
        let query_result = timeout(
            Duration::from_secs(3),
            self.db.client.query(query_str)
        ).await;
        
        match query_result {
            Ok(Ok(mut response)) => {
                let mut results = take_json_values(&mut response, 0);
                normalize_object_ids(&mut results);
                
                let scored_results = results.into_iter().map(|obj| {
                    let score = obj.get("similarity").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let explanation = "Vector similarity match".to_string();
                    (obj, score, explanation)
                }).collect();
                
                Ok(scored_results)
            }
            Ok(Err(e)) => {
                tracing::error!("Vector search failed: {}", e);
                Err(HybridRetrievalError::DatabaseError(e.to_string()))
            }
            Err(_) => {
                tracing::warn!("Vector search timeout");
                Ok(Vec::new()) // Graceful degradation
            }
        }
    }

    async fn execute_graph_search(&self, request: &QueryRequest) -> Result<Vec<(Value, f32, String)>, HybridRetrievalError> {
        if request.graph.is_none() {
            return Ok(Vec::new());
        }

        let graph_query = request.graph.as_ref().unwrap();
        
        tracing::debug!("Executing graph search");
        
        match self.graph_service.execute_multi_hop(graph_query).await {
            Ok(traversal_result) => {
                let scored_results = traversal_result.nodes.into_iter().map(|obj| {
                    let score = 1.0; // Graph results get uniform score for now
                    let explanation = "Graph traversal match".to_string();
                    (obj, score, explanation)
                }).collect();
                
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
        let mut result_map: HashMap<String, HybridResult> = HashMap::new();
        
        // Process text results
        for (obj, score, explanation) in text_results {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                result_map.insert(id.to_string(), HybridResult {
                    object: obj,
                    total_score: score * 0.3, // Text weight
                    text_score: Some(score),
                    vector_score: None,
                    graph_score: None,
                    explanation,
                });
            }
        }
        
        // Process vector results
        for (obj, score, explanation) in vector_results {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                if let Some(existing) = result_map.get_mut(id) {
                    existing.total_score += score * 0.4; // Vector weight
                    existing.vector_score = Some(score);
                    existing.explanation = format!("{} + {}", existing.explanation, explanation);
                } else {
                    result_map.insert(id.to_string(), HybridResult {
                        object: obj,
                        total_score: score * 0.4,
                        text_score: None,
                        vector_score: Some(score),
                        graph_score: None,
                        explanation,
                    });
                }
            }
        }
        
        // Process graph results
        for (obj, score, explanation) in graph_results {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                if let Some(existing) = result_map.get_mut(id) {
                    existing.total_score += score * 0.3; // Graph weight
                    existing.graph_score = Some(score);
                    existing.explanation = format!("{} + {}", existing.explanation, explanation);
                } else {
                    result_map.insert(id.to_string(), HybridResult {
                        object: obj,
                        total_score: score * 0.3,
                        text_score: None,
                        vector_score: None,
                        graph_score: Some(score),
                        explanation,
                    });
                }
            }
        }
        
        // Sort by total score and return
        let mut results: Vec<HybridResult> = result_map.into_values().collect();
        results.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap_or(std::cmp::Ordering::Equal));
        
        results
    }

    fn build_text_query_string(&self, request: &QueryRequest) -> String {
        let mut query = "SELECT * FROM objects".to_string();
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
        let vector_str = vector.iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        
        let mut query = format!(
            "SELECT *, vector::similarity::cosine(embedding, [{}]) AS similarity FROM objects WHERE embedding IS NOT NULL",
            vector_str
        );
        
        let mut conditions = Vec::new();
        self.add_filter_conditions(&mut conditions, &request.filters);
        
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY similarity DESC");
        
        let limit = request.limit.unwrap_or(10);
        query.push_str(&format!(" LIMIT {}", limit));
        
        query
    }

    fn add_filter_conditions(&self, conditions: &mut Vec<String>, filters: &Option<QueryFilters>) {
        if let Some(filters) = filters {
            if let Some(types) = &filters.object_types {
                let types_str = types.iter()
                    .map(|t| format!("'{}'", t.replace("'", "\\'")))
                    .collect::<Vec<_>>()
                    .join(", ");
                conditions.push(format!("type IN [{}]", types_str));
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
