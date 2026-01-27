use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::AppState;

/// Default TTL for connections (10 minutes)
const DEFAULT_TTL_SECONDS: i64 = 600;

/// Request to register a new agent connection
#[derive(Debug, Deserialize)]
pub struct RegisterConnectionRequest {
    /// Unique agent identifier
    pub agent_id: String,
    /// Human-readable agent name (e.g., "Claude", "Cursor")
    pub agent_name: String,
    /// Optional run ID if this connection is for a specific run
    pub run_id: Option<String>,
    /// Optional project ID
    pub project_id: Option<String>,
    /// Custom TTL in seconds (default: 600 = 10 minutes)
    pub ttl_seconds: Option<i64>,
}

/// Response after registering a connection
#[derive(Debug, Serialize)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub run_id: Option<String>,
    pub project_id: Option<String>,
    pub status: String,
    pub last_heartbeat: String,
    pub connected_at: String,
    pub expires_at: String,
}

fn extract_datetime(value: Option<&Value>) -> String {
    if let Some(value) = value {
        if let Some(as_str) = value.as_str() {
            return as_str.to_string();
        }
        if let Some(obj) = value.as_object() {
            if let Some(as_str) = obj.get("$datetime").and_then(|v| v.as_str()) {
                return as_str.to_string();
            }
            if let Some(as_str) = obj.get("time").and_then(|v| v.as_str()) {
                return as_str.to_string();
            }
        }
    }
    String::new()
}

fn extract_connection_info(value: &Value) -> Option<ConnectionInfo> {
    let connection_id = value
        .get("connection_id")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("id").and_then(|v| v.as_str()))
        .map(|id| {
            id.trim_start_matches("agent_connections:")
                .trim_matches('`')
                .to_string()
        })?;

    Some(ConnectionInfo {
        connection_id,
        agent_id: value.get("agent_id")?.as_str()?.to_string(),
        agent_name: value.get("agent_name")?.as_str()?.to_string(),
        run_id: value.get("run_id").and_then(|v| v.as_str()).map(String::from),
        project_id: value.get("project_id").and_then(|v| v.as_str()).map(String::from),
        status: value
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        last_heartbeat: extract_datetime(value.get("last_heartbeat")),
        connected_at: extract_datetime(value.get("connected_at")),
        expires_at: extract_datetime(value.get("expires_at")),
    })
}

/// Request to send a heartbeat
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    /// Connection ID returned from register
    pub connection_id: String,
    /// Optional: update run_id if a run was started
    pub run_id: Option<String>,
    /// Optional: custom TTL extension in seconds
    pub ttl_seconds: Option<i64>,
}

/// Request to disconnect
#[derive(Debug, Deserialize)]
pub struct DisconnectRequest {
    pub connection_id: String,
}

/// Register a new agent connection
///
/// Creates a connection record with TTL-based expiry.
/// The connection will be considered active until expires_at.
pub async fn register_connection(
    State(state): State<AppState>,
    Json(request): Json<RegisterConnectionRequest>,
) -> Result<(StatusCode, Json<ConnectionInfo>), StatusCode> {
    let connection_id = Uuid::new_v4().to_string();
    let ttl_seconds = request.ttl_seconds.unwrap_or(DEFAULT_TTL_SECONDS);
    let now = chrono::Utc::now();

    tracing::info!(
        "Registering agent connection: {} ({}) - connection_id: {}",
        request.agent_name,
        request.agent_id,
        connection_id
    );

    // Use SurrealQL time arithmetic instead of passing datetime string
    let query = format!(
        r#"CREATE agent_connections:`{}` SET
            connection_id = $connection_id,
            agent_id = $agent_id,
            agent_name = $agent_name,
            run_id = $run_id,
            project_id = $project_id,
            status = "connected",
            last_heartbeat = time::now(),
            connected_at = time::now(),
            expires_at = time::now() + {}s"#,
        connection_id, ttl_seconds
    );

    let result = timeout(
        Duration::from_secs(5),
        state
            .db
            .client
            .query(query)
            .bind(("connection_id", connection_id.clone()))
            .bind(("agent_id", request.agent_id.clone()))
            .bind(("agent_name", request.agent_name.clone()))
            .bind(("run_id", request.run_id.clone()))
            .bind(("project_id", request.project_id.clone())),
    )
    .await;

    match result {
        Ok(Ok(response)) => {
            let response = match response.check() {
                Ok(response) => response,
                Err(_) => {
                    tracing::error!("Connection registration returned errors");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let fetch_query = r#"SELECT VALUE { id: string::concat(id), connection_id: connection_id, agent_id: agent_id, agent_name: agent_name, run_id: run_id, project_id: project_id, status: status, last_heartbeat: last_heartbeat, connected_at: connected_at, expires_at: expires_at } FROM agent_connections WHERE connection_id = $connection_id LIMIT 1"#;
            let fetch_result = timeout(
                Duration::from_secs(5),
                state
                    .db
                    .client
                    .query(fetch_query)
                    .bind(("connection_id", connection_id.clone())),
            )
            .await;

            if let Ok(Ok(fetch_response)) = fetch_result {
                let mut fetch_response = match fetch_response.check() {
                    Ok(response) => response,
                    Err(_) => {
                        tracing::error!("Connection fetch returned errors");
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };

                let values = crate::surreal_json::take_json_values(&mut fetch_response, 0);
                if let Some(info) = values.first().and_then(extract_connection_info) {
                    tracing::info!("Connection registered: {}", connection_id);
                    return Ok((StatusCode::CREATED, Json(info)));
                }
            }

            tracing::info!("Connection registered: {}", connection_id);
            let expires_at = now + chrono::Duration::seconds(ttl_seconds);
            Ok((
                StatusCode::CREATED,
                Json(ConnectionInfo {
                    connection_id,
                    agent_id: request.agent_id,
                    agent_name: request.agent_name,
                    run_id: request.run_id,
                    project_id: request.project_id,
                    status: "connected".to_string(),
                    last_heartbeat: now.to_rfc3339(),
                    connected_at: now.to_rfc3339(),
                    expires_at: expires_at.to_rfc3339(),
                }),
            ))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to register connection: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout registering connection");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

/// Send a heartbeat to keep the connection alive
///
/// Updates last_heartbeat and extends expires_at by TTL.
pub async fn heartbeat(
    State(state): State<AppState>,
    Json(request): Json<HeartbeatRequest>,
) -> Result<StatusCode, StatusCode> {
    let ttl_seconds = request.ttl_seconds.unwrap_or(DEFAULT_TTL_SECONDS);

    tracing::debug!("Heartbeat for connection: {}", request.connection_id);

    // Build update query - optionally update run_id if provided
    let query = if request.run_id.is_some() {
        format!(
            "UPDATE agent_connections SET
                last_heartbeat = time::now(),
                expires_at = time::now() + {}s,
                run_id = $run_id
             WHERE connection_id = $connection_id",
            ttl_seconds
        )
    } else {
        format!(
            "UPDATE agent_connections SET
                last_heartbeat = time::now(),
                expires_at = time::now() + {}s
             WHERE connection_id = $connection_id",
            ttl_seconds
        )
    };

    let result = timeout(
        Duration::from_secs(5),
        state
            .db
            .client
            .query(&query)
            .bind(("connection_id", request.connection_id.clone()))
            .bind(("run_id", request.run_id)),
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            tracing::debug!("Heartbeat recorded for: {}", request.connection_id);
            Ok(StatusCode::OK)
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to record heartbeat: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout recording heartbeat");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

/// Disconnect an agent connection
///
/// Marks the connection as disconnected (does not delete immediately).
pub async fn disconnect(
    State(state): State<AppState>,
    Json(request): Json<DisconnectRequest>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Disconnecting connection: {}", request.connection_id);

    let query = r#"UPDATE agent_connections SET
        status = "disconnected",
        expires_at = time::now()
     WHERE connection_id = $connection_id"#;

    let result = timeout(
        Duration::from_secs(5),
        state
            .db
            .client
            .query(query)
            .bind(("connection_id", request.connection_id.clone())),
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            tracing::info!("Connection disconnected: {}", request.connection_id);
            Ok(StatusCode::OK)
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to disconnect: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout disconnecting");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

/// List active connections
///
/// Returns connections where expires_at > now (TTL not expired).
/// Optionally filter by run_id or project_id.
#[derive(Debug, Deserialize)]
pub struct ListConnectionsQuery {
    pub run_id: Option<String>,
    pub project_id: Option<String>,
    /// Include expired connections (default: false)
    pub include_expired: Option<bool>,
}

pub async fn list_connections(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListConnectionsQuery>,
) -> Result<Json<Vec<ConnectionInfo>>, StatusCode> {
    let include_expired = query.include_expired.unwrap_or(false);

    // Build query with filters
    let mut conditions = Vec::new();

    if !include_expired {
        conditions.push("expires_at > time::now()".to_string());
    }

    if let Some(run_id) = &query.run_id {
        conditions.push(format!("run_id = '{}'", run_id.replace('\'', "\\'")));
    }

    if let Some(project_id) = &query.project_id {
        conditions.push(format!("project_id = '{}'", project_id.replace('\'', "\\'")));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let query_str = format!(
        r#"SELECT VALUE {{ id: string::concat(id), connection_id: connection_id, agent_id: agent_id, agent_name: agent_name, run_id: run_id, project_id: project_id, status: status, last_heartbeat: last_heartbeat, connected_at: connected_at, expires_at: expires_at }} FROM (SELECT id, connection_id, agent_id, agent_name, run_id, project_id, status, last_heartbeat, connected_at, expires_at FROM agent_connections{} ORDER BY last_heartbeat DESC)"#,
        where_clause
    );

    tracing::debug!("List connections query: {}", query_str);

    let result = timeout(Duration::from_secs(5), state.db.client.query(&query_str)).await;

    match result {
        Ok(Ok(response)) => {
            let mut response = match response.check() {
                Ok(response) => response,
                Err(_) => {
                    tracing::error!("List connections returned errors");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let connections: Vec<Value> = crate::surreal_json::take_json_values(&mut response, 0);

            let infos: Vec<ConnectionInfo> = connections
                .into_iter()
                .filter_map(|v| extract_connection_info(&v))
                .collect();

            tracing::debug!("Found {} active connections", infos.len());
            Ok(Json(infos))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to list connections: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout listing connections");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

/// Cleanup expired connections (optional background task endpoint)
pub async fn cleanup_expired(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Cleaning up expired connections");

    let query = "DELETE FROM agent_connections WHERE expires_at < time::now()";

    let result = timeout(Duration::from_secs(10), state.db.client.query(query)).await;

    match result {
        Ok(Ok(_)) => {
            tracing::info!("Expired connections cleaned up");
            Ok(Json(serde_json::json!({
                "status": "ok",
                "message": "Expired connections cleaned up"
            })))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to cleanup connections: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout cleaning up connections");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}
