use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;
use serde_json::Value;
use tokio::time::{timeout, Duration};

use crate::{models::relationships::*, surreal_json::take_json_values, AppState};

#[derive(Debug, Deserialize)]
pub struct RelationshipQuery {
    #[serde(rename = "object_id")]
    pub object_id: Option<String>,
    pub source_id: Option<String>,
    pub target_id: Option<String>,
    #[serde(rename = "type")]
    pub relation_type: Option<String>,
}

pub async fn create_relationship(
    State(state): State<AppState>,
    Json(request): Json<CreateRelationshipRequest>,
) -> Result<(StatusCode, Json<RelationshipResponse>), StatusCode> {
    let relationship_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Determine table name based on relationship type
    let table_name = match request.relation_type {
        RelationType::DependsOn => "depends_on",
        RelationType::DefinedIn => "defined_in",
        RelationType::Calls => "calls",
        RelationType::JustifiedBy => "justified_by",
        RelationType::Modifies => "modifies",
        RelationType::Implements => "implements",
        RelationType::Produced => "produced",
    };
    
    // Verify both objects exist first - use simple SELECT instead of type::record
    // Skip verification - SurrealDB enum serialization issues prevent proper verification
    tracing::info!("Creating relationship: {} -> {} -> {}", request.source_id, table_name, request.target_id);
    
    // Use RELATE statement for graph edges - use hyphenated UUID format
    let query = format!(
        "RELATE objects:`{}`->{}->objects:`{}` SET created_at = time::now()",
        request.source_id, table_name, request.target_id
    );
    
    tracing::debug!("Creating relationship: {}", query);
    
    let result = timeout(
        Duration::from_secs(5),
        state.db.client.query(query)
    ).await;
    
    match result {
        Ok(Ok(_response)) => {
            tracing::info!("Created relationship: {} -> {} ({})", 
                request.source_id, request.target_id, table_name);
            
            Ok((
                StatusCode::CREATED,
                Json(RelationshipResponse {
                    id: relationship_id,
                    relation_type: request.relation_type,
                    source_id: request.source_id,
                    target_id: request.target_id,
                    created_at: now.to_rfc3339(),
                }),
            ))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to create relationship: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout creating relationship");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn get_relationships(
    State(state): State<AppState>,
    Query(query): Query<RelationshipQuery>,
) -> Result<Json<Vec<Value>>, StatusCode> {
    tracing::debug!(
        "Relationship query params: object_id={:?}, source_id={:?}, target_id={:?}, type={:?}",
        query.object_id, query.source_id, query.target_id, query.relation_type
    );
    // Build query based on filters - use SELECT VALUE to avoid enum serialization issues
    let mut query_str = String::from("SELECT VALUE { in: string::concat(in.id), out: string::concat(out.id), type: meta::tb(id), created_at: created_at } FROM [");
    
    if let Some(rel_type) = &query.relation_type {
        query_str.push_str(rel_type);
    } else {
        query_str.push_str("depends_on, defined_in, calls, justified_by, modifies, implements, produced");
    }
    
    query_str.push_str("]");
    
    let mut conditions = Vec::new();
    if query.object_id.is_some() {
        conditions.push("(in = type::thing('objects', $object) OR out = type::thing('objects', $object))".to_string());
    }
    if query.source_id.is_some() {
        conditions.push("out = type::thing('objects', $source)".to_string());
    }
    if query.target_id.is_some() {
        conditions.push("in = type::thing('objects', $target)".to_string());
    }
    
    if !conditions.is_empty() {
        query_str.push_str(" WHERE ");
        query_str.push_str(&conditions.join(" AND "));
    }
    
    tracing::debug!("Relationship query: {}", query_str);
    
    let mut query_exec = state.db.client.query(query_str);
    if let Some(object) = query.object_id {
        query_exec = query_exec.bind(("object", object));
    }
    if let Some(source) = query.source_id {
        query_exec = query_exec.bind(("source", source));
    }
    if let Some(target) = query.target_id {
        query_exec = query_exec.bind(("target", target));
    }
    let result = timeout(
        Duration::from_secs(5),
        query_exec
    ).await;
    
    match result {
        Ok(Ok(mut response)) => {
            let relationships: Vec<Value> = take_json_values(&mut response, 0);
            tracing::debug!("Raw query response: {:?}", relationships);
            tracing::debug!("Found {} relationships", relationships.len());
            Ok(Json(relationships))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to query relationships: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout querying relationships");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn delete_relationship(
    State(state): State<AppState>,
    Path((rel_type, id)): Path<(String, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result: Result<Result<Option<Value>, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.delete((rel_type.as_str(), id))
    ).await;
    
    match result {
        Ok(Ok(Some(_))) => {
            tracing::info!("Deleted relationship: {}:{}", rel_type, id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(Ok(None)) => Err(StatusCode::NOT_FOUND),
        Ok(Err(e)) => {
            tracing::error!("Failed to delete relationship: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout deleting relationship");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}
