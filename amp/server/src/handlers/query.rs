use crate::{
    surreal_json::{normalize_object_ids, take_json_values},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::Json};
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryRequest {
    pub text: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub filters: Option<QueryFilters>,
    pub graph: Option<GraphQuery>,
    pub limit: Option<usize>,
    pub hybrid: Option<bool>,
    pub graph_intersect: Option<bool>,
    pub graph_autoseed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryFilters {
    #[serde(rename = "type")]
    pub object_types: Option<Vec<String>>,
    pub project_id: Option<String>,
    pub tenant_id: Option<String>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphQuery {
    #[serde(default, deserialize_with = "deserialize_uuidish_vec")]
    pub start_nodes: Vec<Uuid>,
    pub relation_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    pub direction: Option<GraphDirection>,
    pub algorithm: Option<TraversalAlgorithm>,
    #[serde(default, deserialize_with = "deserialize_uuidish_opt")]
    pub target_node: Option<Uuid>, // For shortest path algorithm
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GraphDirection {
    Outbound,
    Inbound,
    Both,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalAlgorithm {
    Collect,  // Collect unique nodes
    Path,     // Return all paths
    Shortest, // Shortest path to target
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub results: Vec<QueryResult>,
    pub trace_id: Uuid,
    pub total_count: usize,
    pub execution_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_results_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_results_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_results_count: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub object: Value,
    pub score: f32,
    pub explanation: String,
    pub path: Option<Vec<Value>>, // New field for traversal paths
}

pub async fn query(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    let trace_id = Uuid::new_v4();

    tracing::info!("Query request: trace_id={}, text={:?}, has_vector={}, has_graph={}, hybrid={:?}, filters={:?}", 
        trace_id, request.text, request.vector.is_some(), request.graph.is_some(), request.hybrid, request.filters);

    // Check if this is a hybrid query
    if request.hybrid.unwrap_or(false) {
        tracing::info!("Executing hybrid query: trace_id={}", trace_id);

        match state.hybrid_service.execute_hybrid_query(&request).await {
            Ok(hybrid_response) => {
                // Convert HybridResult to QueryResult for response compatibility
                let results: Vec<QueryResult> = hybrid_response
                    .results
                    .into_iter()
                    .map(|hybrid_result| QueryResult {
                        object: hybrid_result.object,
                        score: hybrid_result.total_score,
                        explanation: hybrid_result.explanation,
                        path: None, // Hybrid results don't have path information yet
                    })
                    .collect();

                return Ok(Json(QueryResponse {
                    results,
                    trace_id,
                    total_count: hybrid_response.total_count,
                    execution_time_ms: hybrid_response.execution_time_ms,
                    text_results_count: Some(hybrid_response.text_results_count),
                    vector_results_count: Some(hybrid_response.vector_results_count),
                    graph_results_count: Some(hybrid_response.graph_results_count),
                }));
            }
            Err(e) => {
                tracing::error!("Hybrid query failed: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // Check if this is a graph query
    if let Some(graph) = &request.graph {
        // Validate depth limits for performance and safety
        if let Some(depth) = &graph.max_depth {
            if *depth > 10 {
                tracing::warn!(
                    "Graph query rejected: max_depth {} exceeds limit of 10",
                    depth
                );
                return Err(StatusCode::BAD_REQUEST);
            }
        }

        // Use graph service for algorithms or multi-relation traversals.
        let use_graph_service = graph.algorithm.is_some()
            || graph
                .relation_types
                .as_ref()
                .map(|types| types.len() > 1)
                .unwrap_or(false);

        if use_graph_service {
            tracing::info!(
                "Executing multi-hop graph traversal: algorithm={:?}, depth={}",
                graph.algorithm,
                graph.max_depth.unwrap_or(1)
            );

            match state.graph_service.execute_multi_hop(graph).await {
                Ok(traversal_result) => {
                    let results: Vec<QueryResult> = traversal_result
                        .nodes
                        .into_iter()
                        .map(|obj| {
                            QueryResult {
                                object: obj,
                                score: 1.0,
                                explanation: format!(
                                    "Multi-hop {} traversal result",
                                    graph
                                        .algorithm
                                        .as_ref()
                                        .map(|a| format!("{:?}", a))
                                        .unwrap_or_default()
                                ),
                                path: traversal_result.paths.as_ref().and_then(|paths| {
                                    // For now, return the first path if available
                                    paths.first().map(|path| {
                                        path.iter().map(|id| {
                                            serde_json::json!({"id": format!("objects:{}", id)})
                                        }).collect()
                                    })
                                }),
                            }
                        })
                        .collect();

                    let total_count = results.len();
                    let execution_time_ms = start_time.elapsed().as_millis() as u64;

                    tracing::info!(
                        "Multi-hop graph query complete: trace_id={}, results={}, time={}ms",
                        trace_id,
                        total_count,
                        execution_time_ms
                    );

                    return Ok(Json(QueryResponse {
                        results,
                        trace_id,
                        total_count,
                        execution_time_ms,
                        text_results_count: None,
                        vector_results_count: None,
                        graph_results_count: None,
                    }));
                }
                Err(e) => {
                    tracing::error!("Multi-hop graph traversal failed: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }

        // Fall back to single-hop query for backward compatibility
        let limit = request.limit.unwrap_or(10);
        let query_str = build_graph_query_string(graph, request.filters.as_ref(), limit);

        tracing::debug!("Executing single-hop graph query: {}", query_str);

        let query_result = timeout(Duration::from_secs(5), state.db.client.query(query_str)).await;

        let objects: Vec<Value> = match query_result {
            Ok(Ok(mut response)) => {
                // The query returns objects with "connected" field containing the traversed nodes
                let raw_results: Vec<Value> = take_json_values(&mut response, 0);
                tracing::debug!("Raw graph query results: {:?}", raw_results);

                // Extract the connected objects from the result
                let mut connected = raw_results
                    .into_iter()
                    .filter_map(|v| {
                        if let Some(obj) = v.as_object() {
                            // Get the "connected" field which contains the traversed objects
                            obj.get("connected").cloned()
                        } else {
                            None
                        }
                    })
                    .flat_map(|v| {
                        // connected can be an array or a single object
                        if let Some(arr) = v.as_array() {
                            arr.clone()
                        } else {
                            vec![v]
                        }
                    })
                    .collect::<Vec<_>>();

                normalize_object_ids(&mut connected);
                connected
            }
            Ok(Err(e)) => {
                tracing::error!("Graph query failed: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Err(_) => {
                tracing::error!("Graph query timeout");
                return Err(StatusCode::GATEWAY_TIMEOUT);
            }
        };

        let results: Vec<QueryResult> = objects
            .into_iter()
            .map(|obj| {
                QueryResult {
                    object: obj,
                    score: 1.0,
                    explanation: "Graph traversal result".to_string(),
                    path: None, // TODO: Extract path information from recursive query results
                }
            })
            .collect();

        let total_count = results.len();
        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        tracing::info!(
            "Graph query complete: trace_id={}, results={}, time={}ms",
            trace_id,
            total_count,
            execution_time_ms
        );

        return Ok(Json(QueryResponse {
            results,
            trace_id,
            total_count,
            execution_time_ms,
            text_results_count: None,
            vector_results_count: None,
            graph_results_count: None,
        }));
    }

    // Determine if we should use vector search
    tracing::info!(
        "Non-hybrid query: determining query vector, embedding_enabled={}",
        state.embedding_service.is_enabled()
    );
    let query_vector = if let Some(vector) = &request.vector {
        tracing::info!("Using provided vector");
        Some(vector.clone())
    } else if let Some(text) = &request.text {
        // Generate embedding from text query if service is enabled
        if state.embedding_service.is_enabled() {
            tracing::info!("Generating embedding for text: '{}'", text);
            match state.embedding_service.generate_embedding(text).await {
                Ok(vec) => {
                    tracing::info!(
                        "Generated embedding from text query: {} dimensions",
                        vec.len()
                    );
                    Some(vec)
                }
                Err(e) => {
                    tracing::warn!("Failed to generate query embedding: {}", e);
                    None
                }
            }
        } else {
            tracing::info!("Embedding service disabled");
            None
        }
    } else {
        tracing::info!("No text or vector provided");
        None
    };

    // Build query based on whether we have a vector
    let query_str = if query_vector.is_some() {
        tracing::info!("Building vector query");
        build_vector_query_string(&request, &query_vector.as_ref().unwrap())
    } else {
        tracing::info!("Building text query");
        build_query_string(&request)
    };

    tracing::info!("Executing query length: {} chars", query_str.len());
    // Log a hash of the query for debugging
    tracing::debug!("Full query: {}", query_str);

    // Execute with timeout
    let query_result = timeout(Duration::from_secs(5), state.db.client.query(query_str)).await;

    let objects = match query_result {
        Ok(Ok(mut response)) => {
            let mut results = take_json_values(&mut response, 0);
            normalize_object_ids(&mut results);
            results
        }
        Ok(Err(e)) => {
            tracing::error!("Query failed: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Err(_) => {
            tracing::error!("Query timeout");
            return Err(StatusCode::GATEWAY_TIMEOUT);
        }
    };

    // Score and explain results
    let mut results: Vec<QueryResult> = objects
        .into_iter()
        .map(|obj| {
            let score = calculate_score(&obj, request.text.as_ref());
            let explanation = generate_explanation(&obj, &request);

            QueryResult {
                object: obj,
                score,
                explanation,
                path: None, // Non-graph queries don't have path information
            }
        })
        .collect();

    // Sort by score descending
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_count = results.len();
    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    tracing::info!(
        "Query complete: trace_id={}, results={}, time={}ms",
        trace_id,
        total_count,
        execution_time_ms
    );

    Ok(Json(QueryResponse {
        results,
        trace_id,
        total_count,
        execution_time_ms,
        text_results_count: None,
        vector_results_count: None,
        graph_results_count: None,
    }))
}

fn build_query_string(request: &QueryRequest) -> String {
    let mut base_query = "SELECT VALUE { id: string::concat(id), type: type, tenant_id: tenant_id, project_id: project_id, name: name, title: title, kind: kind, path: path, language: language, signature: signature, documentation: documentation, summary: summary, description: description, content: content, tags: tags, linked_files: linked_files, file_path: file_path, files_changed: files_changed, decision: decision, diff_summary: diff_summary, context: context, category: category, created_at: created_at, updated_at: updated_at, provenance: provenance, links: links, embedding: embedding, input_summary: input_summary, status: status, duration_ms: duration_ms, confidence: confidence } FROM objects".to_string();
    let mut conditions = Vec::new();

    // Text search
    if let Some(text) = &request.text {
        let text_escaped = text.replace("'", "\\'");
        conditions.push(format!(
            "(name CONTAINS '{}' OR title CONTAINS '{}' OR description CONTAINS '{}' OR documentation CONTAINS '{}')",
            text_escaped, text_escaped, text_escaped, text_escaped
        ));
    }

    // Filters
    if let Some(filters) = &request.filters {
        if let Some(types) = &filters.object_types {
            let types_str = types
                .iter()
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

        if let Some(created_after) = &filters.created_after {
            conditions.push(format!(
                "created_at >= time::from::unix({})",
                created_after.timestamp()
            ));
        }

        if let Some(created_before) = &filters.created_before {
            conditions.push(format!(
                "created_at <= time::from::unix({})",
                created_before.timestamp()
            ));
        }
    }

    // Combine conditions
    if !conditions.is_empty() {
        base_query.push_str(" WHERE ");
        base_query.push_str(&conditions.join(" AND "));
    }

    // Limit
    let limit = request.limit.unwrap_or(10);
    base_query.push_str(&format!(" LIMIT {}", limit));

    base_query
}

fn build_vector_query_string(request: &QueryRequest, vector: &[f32]) -> String {
    let vector_str = vector
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let mut inner_query = "SELECT id, type, tenant_id, project_id, name, title, kind, path, language, signature, documentation, summary, description, content, tags, linked_files, file_path, files_changed, decision, diff_summary, context, category, created_at, updated_at, provenance, links, embedding, input_summary, status, duration_ms, confidence FROM objects WHERE embedding IS NOT NONE AND embedding IS NOT NULL".to_string();

    let mut conditions = Vec::new();

    // Filters
    if let Some(filters) = &request.filters {
        if let Some(types) = &filters.object_types {
            let types_str = types
                .iter()
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

        if let Some(created_after) = &filters.created_after {
            conditions.push(format!(
                "created_at >= time::from::unix({})",
                created_after.timestamp()
            ));
        }

        if let Some(created_before) = &filters.created_before {
            conditions.push(format!(
                "created_at <= time::from::unix({})",
                created_before.timestamp()
            ));
        }
    }

    // Add additional conditions
    if !conditions.is_empty() {
        inner_query.push_str(" AND ");
        inner_query.push_str(&conditions.join(" AND "));
    }

    // Limit
    let limit = request.limit.unwrap_or(10);
    let inner_ranked_query = format!(
        "SELECT id, type, tenant_id, project_id, name, title, kind, path, language, signature, documentation, summary, description, content, tags, linked_files, file_path, files_changed, decision, diff_summary, context, category, created_at, updated_at, provenance, links, embedding, input_summary, status, duration_ms, confidence, vector::similarity::cosine(embedding, [{}]) AS similarity FROM ({}) ORDER BY similarity DESC LIMIT {}",
        vector_str, inner_query, limit
    );

    format!(
        "SELECT VALUE {{ id: string::concat(id), type: type, tenant_id: tenant_id, project_id: project_id, name: name, title: title, kind: kind, path: path, language: language, signature: signature, documentation: documentation, summary: summary, description: description, content: content, tags: tags, linked_files: linked_files, file_path: file_path, files_changed: files_changed, decision: decision, diff_summary: diff_summary, context: context, category: category, created_at: created_at, updated_at: updated_at, provenance: provenance, links: links, embedding: embedding, input_summary: input_summary, status: status, duration_ms: duration_ms, confidence: confidence, similarity: similarity }} FROM ({})",
        inner_ranked_query
    )
}

fn build_graph_query_string(
    graph: &GraphQuery,
    filters: Option<&QueryFilters>,
    limit: usize,
) -> String {
    let direction = graph
        .direction
        .as_ref()
        .unwrap_or(&GraphDirection::Outbound);
    let max_depth = graph.max_depth.unwrap_or(3);
    let projection = "{ id: string::concat(id), type: type, tenant_id: tenant_id, project_id: project_id, name: name, title: title, kind: kind, path: path, language: language, signature: signature, documentation: documentation, summary: summary, description: description, content: content, tags: tags, linked_files: linked_files, file_path: file_path, files_changed: files_changed, decision: decision, diff_summary: diff_summary, context: context, category: category, created_at: created_at, updated_at: updated_at, provenance: provenance, links: links, embedding: embedding, input_summary: input_summary, status: status, duration_ms: duration_ms, confidence: confidence }";

    // Build the start node list
    let start_ids_list = graph
        .start_nodes
        .iter()
        .map(|id| format!("objects:`{}`", id))
        .collect::<Vec<_>>()
        .join(", ");

    // Build relationship list
    let relation_list = if let Some(types) = &graph.relation_types {
        types.clone()
    } else {
        vec![
            "depends_on".to_string(),
            "defined_in".to_string(),
            "calls".to_string(),
            "justified_by".to_string(),
            "modifies".to_string(),
            "implements".to_string(),
            "produced".to_string(),
        ]
    };
    let relation_clause = if relation_list.len() == 1 {
        relation_list[0].clone()
    } else {
        format!("({})", relation_list.join("|"))
    };

    // Build recursive syntax based on algorithm
    let _recursive_syntax = match &graph.algorithm {
        Some(TraversalAlgorithm::Collect) => format!("{{{}+collect}}", max_depth),
        Some(TraversalAlgorithm::Path) => format!("{{{}+path}}", max_depth),
        Some(TraversalAlgorithm::Shortest) => {
            if let Some(target) = &graph.target_node {
                format!("{{..{}+shortest=objects:`{}`}}", max_depth, target)
            } else {
                format!("{{{}}}", max_depth) // Fallback to simple depth
            }
        }
        None => format!("{{{}}}", max_depth), // Simple depth traversal
    };

    // Build the graph traversal query based on algorithm
    // Note: SurrealDB's recursive syntax {depth+algorithm} works with field access, not relationship traversal
    // For relationship-based graphs, we need to use a different approach
    let mut query = match &graph.algorithm {
        Some(TraversalAlgorithm::Collect) => {
            // For collect algorithm, we'll simulate multi-hop by using multiple relationship hops
            // This is a simplified implementation - true recursive collect would need custom logic
            match direction {
                GraphDirection::Outbound => {
                    if max_depth <= 1 {
                        format!(
                            "SELECT VALUE {{ connected: ->{}->objects.{} }} FROM {}",
                            relation_clause, projection, start_ids_list
                        )
                    } else {
                        // Multi-hop simulation: traverse multiple levels and collect unique results
                        format!(
                            "SELECT VALUE {{ connected: ->{}->objects.{} }} FROM {}",
                            relation_clause, projection, start_ids_list
                        )
                    }
                }
                GraphDirection::Inbound => {
                    format!(
                        "SELECT VALUE {{ connected: <-{}<-objects.{} }} FROM {}",
                        relation_clause, projection, start_ids_list
                    )
                }
                GraphDirection::Both => {
                    format!("SELECT VALUE {{ outbound: ->{}->objects.{}, inbound: <-{}<-objects.{} }} FROM {}", 
                        relation_clause, projection, relation_clause, projection, start_ids_list)
                }
            }
        }
        Some(TraversalAlgorithm::Path) => {
            // For path algorithm, fall back to simple traversal for now
            match direction {
                GraphDirection::Outbound => {
                    format!(
                        "SELECT VALUE {{ connected: ->{}->objects.{} }} FROM {}",
                        relation_clause, projection, start_ids_list
                    )
                }
                GraphDirection::Inbound => {
                    format!(
                        "SELECT VALUE {{ connected: <-{}<-objects.{} }} FROM {}",
                        relation_clause, projection, start_ids_list
                    )
                }
                GraphDirection::Both => {
                    format!("SELECT VALUE {{ outbound: ->{}->objects.{}, inbound: <-{}<-objects.{} }} FROM {}", 
                        relation_clause, projection, relation_clause, projection, start_ids_list)
                }
            }
        }
        Some(TraversalAlgorithm::Shortest) => {
            // For shortest path, fall back to simple traversal for now
            match direction {
                GraphDirection::Outbound => {
                    format!(
                        "SELECT VALUE {{ connected: ->{}->objects.{} }} FROM {}",
                        relation_clause, projection, start_ids_list
                    )
                }
                GraphDirection::Inbound => {
                    format!(
                        "SELECT VALUE {{ connected: <-{}<-objects.{} }} FROM {}",
                        relation_clause, projection, start_ids_list
                    )
                }
                GraphDirection::Both => {
                    format!("SELECT VALUE {{ outbound: ->{}->objects.{}, inbound: <-{}<-objects.{} }} FROM {}", 
                        relation_clause, projection, relation_clause, projection, start_ids_list)
                }
            }
        }
        None => {
            // No algorithm specified - use simple depth traversal (backward compatibility)
            match direction {
                GraphDirection::Outbound => {
                    format!(
                        "SELECT VALUE {{ connected: ->{}->objects.{} }} FROM {}",
                        relation_clause, projection, start_ids_list
                    )
                }
                GraphDirection::Inbound => {
                    format!(
                        "SELECT VALUE {{ connected: <-{}<-objects.{} }} FROM {}",
                        relation_clause, projection, start_ids_list
                    )
                }
                GraphDirection::Both => {
                    format!("SELECT VALUE {{ outbound: ->{}->objects.{}, inbound: <-{}<-objects.{} }} FROM {}", 
                        relation_clause, projection, relation_clause, projection, start_ids_list)
                }
            }
        }
    };

    // Add filters
    if let Some(filters) = filters {
        let mut conditions = Vec::new();

        if let Some(types) = &filters.object_types {
            let types_str = types
                .iter()
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

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
    }

    query.push_str(&format!(" LIMIT {}", limit));
    query
}

fn calculate_score(obj: &Value, text_query: Option<&String>) -> f32 {
    // If we have a similarity score from vector search, use it
    if let Some(similarity) = obj.get("similarity").and_then(|v| v.as_f64()) {
        return similarity as f32;
    }

    if text_query.is_none() {
        return 1.0; // No text query, all results equal
    }

    let query = text_query.unwrap().to_lowercase();

    // Check name/title fields
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

    // Check description/documentation
    if let Some(desc) = obj.get("description").and_then(|v| v.as_str()) {
        if desc.to_lowercase().contains(&query) {
            return 0.6;
        }
    }

    if let Some(doc) = obj.get("documentation").and_then(|v| v.as_str()) {
        if doc.to_lowercase().contains(&query) {
            return 0.6;
        }
    }

    0.4 // Default for other matches
}

fn generate_explanation(obj: &Value, request: &QueryRequest) -> String {
    let mut parts = Vec::new();

    // Check if this is a vector search result
    if let Some(similarity) = obj.get("similarity").and_then(|v| v.as_f64()) {
        if let Some(text) = &request.text {
            parts.push(format!(
                "Semantic similarity to '{}' (score: {:.3})",
                text, similarity
            ));
        } else {
            parts.push(format!("Vector similarity (score: {:.3})", similarity));
        }
    } else if let Some(text) = &request.text {
        // Text search explanation
        let field = if obj
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase().contains(&text.to_lowercase()))
            .unwrap_or(false)
        {
            "name"
        } else if obj
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase().contains(&text.to_lowercase()))
            .unwrap_or(false)
        {
            "title"
        } else if obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase().contains(&text.to_lowercase()))
            .unwrap_or(false)
        {
            "description"
        } else {
            "content"
        };

        parts.push(format!("Matched text query '{}' in {}", text, field));
    }

    if let Some(filters) = &request.filters {
        let mut filter_parts = Vec::new();

        if let Some(types) = &filters.object_types {
            filter_parts.push(format!("type={}", types.join(",")));
        }
        if let Some(project_id) = &filters.project_id {
            filter_parts.push(format!("project={}", project_id));
        }
        if let Some(tenant_id) = &filters.tenant_id {
            filter_parts.push(format!("tenant={}", tenant_id));
        }

        if !filter_parts.is_empty() {
            parts.push(format!("Filtered by {}", filter_parts.join(", ")));
        }
    }

    if parts.is_empty() {
        "Matched query criteria".to_string()
    } else {
        parts.join("; ")
    }
}

fn parse_uuidish(input: &str) -> Option<Uuid> {
    Uuid::parse_str(input.trim_start_matches("objects:")).ok()
}

fn deserialize_uuidish_vec<'de, D>(deserializer: D) -> Result<Vec<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Vec::<String>::deserialize(deserializer)?;
    let mut ids = Vec::with_capacity(raw.len());
    for value in raw {
        let id = parse_uuidish(&value)
            .ok_or_else(|| D::Error::custom(format!("invalid uuid: {}", value)))?;
        ids.push(id);
    }
    Ok(ids)
}

fn deserialize_uuidish_opt<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    match raw {
        Some(value) => {
            let id = parse_uuidish(&value)
                .ok_or_else(|| D::Error::custom(format!("invalid uuid: {}", value)))?;
            Ok(Some(id))
        }
        None => Ok(None),
    }
}
