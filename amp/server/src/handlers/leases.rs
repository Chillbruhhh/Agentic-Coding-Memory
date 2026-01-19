use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use tokio::time::{timeout, Duration};
use serde_json::Value;
use crate::{surreal_json::{take_json_value, take_json_values}, AppState};

#[derive(Debug, Deserialize)]
pub struct LeaseRequest {
    pub resource: String,
    #[serde(alias = "holder")]
    pub agent_id: String,
    #[serde(alias = "ttl_seconds")]
    pub duration: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct LeaseResponse {
    pub lease_id: Uuid,
    pub resource: String,
    pub holder: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseRequest {
    pub lease_id: Uuid,
}

pub async fn acquire_lease(
    State(state): State<AppState>,
    Json(request): Json<LeaseRequest>,
) -> Result<(StatusCode, Json<LeaseResponse>), StatusCode> {
    let lease_id = Uuid::new_v4();
    let ttl_seconds = request.duration.unwrap_or(300); // Default 5 minutes
    
    // Check for existing lease on this resource
    let query = format!(
        "SELECT * FROM leases WHERE resource = '{}' AND expires_at > time::now()",
        request.resource.replace("'", "\\'") // Escape single quotes
    );
    
    let check_result = timeout(
        Duration::from_secs(5),
        state.db.client.query(query)
    ).await;

    match check_result {
        Ok(Ok(mut response)) => {
            let results: Vec<Value> = take_json_values(&mut response, 0);
            if !results.is_empty() {
                tracing::warn!("Lease conflict for resource: {}", request.resource);
                return Err(StatusCode::CONFLICT);
            }
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to check existing leases: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Err(_) => {
            tracing::error!("Timeout checking leases");
            return Err(StatusCode::GATEWAY_TIMEOUT);
        }
    }

    // Calculate expiration
    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);

    let create_query = format!("CREATE leases:`{}` CONTENT {{ resource: $resource, holder: $holder, created_at: time::from::unix($created_at), expires_at: time::from::unix($expires_at) }}", lease_id);

    let create_result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client
            .query(create_query)
            .bind(("resource", request.resource.clone()))
            .bind(("holder", request.agent_id.clone()))
            .bind(("created_at", now.timestamp()))
            .bind(("expires_at", expires_at.timestamp()))
    ).await;

    match create_result {
        Ok(Ok(_)) => {
            tracing::info!("Lease acquired: {} by {}", request.resource, request.agent_id);
            Ok((
                StatusCode::CREATED,
                Json(LeaseResponse {
                    lease_id,
                    resource: request.resource,
                    holder: request.agent_id,
                    expires_at: expires_at.to_rfc3339(),
                }),
            ))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to create lease: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout creating lease");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn release_lease(
    State(state): State<AppState>,
    Json(request): Json<ReleaseRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Delete without RETURN to avoid serialization issues
    let query = format!("DELETE leases:`{}`", request.lease_id);
    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.query(query)
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            tracing::info!("Lease released: {}", request.lease_id);
            Ok(Json(serde_json::json!({"success": true, "message": "Lease released"})))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to release lease {}: {}", request.lease_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout releasing lease {}", request.lease_id);
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RenewRequest {
    pub lease_id: Uuid,
    #[serde(alias = "ttl_seconds")]
    pub duration: Option<u64>,
}

pub async fn renew_lease(
    State(state): State<AppState>,
    Json(request): Json<RenewRequest>,
) -> Result<(StatusCode, Json<LeaseResponse>), StatusCode> {
    let ttl_seconds = request.duration.unwrap_or(300);
    
    // Get existing lease
    let query = "SELECT * FROM type::record('leases', $lease_id)";
    let get_result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.query(query).bind(("lease_id", request.lease_id)),
    )
    .await;

    let lease_data = match get_result {
        Ok(Ok(mut response)) => match take_json_value(&mut response, 0) {
            Some(data) => data,
            None => {
                tracing::warn!("Lease not found for renewal: {}", request.lease_id);
                return Err(StatusCode::NOT_FOUND);
            }
        },
        Ok(Err(e)) => {
            tracing::error!("Failed to get lease: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Err(_) => {
            tracing::error!("Timeout getting lease");
            return Err(StatusCode::GATEWAY_TIMEOUT);
        }
    };

    // Extract resource and holder
    let resource = lease_data.get("resource")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();
    let holder = lease_data.get("holder")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // Calculate new expiration
    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);

    // Delete old lease
    let delete_query = "DELETE type::record('leases', $lease_id)";
    let _delete_result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.query(delete_query).bind(("lease_id", request.lease_id))
    ).await;

    // Create new lease with same ID and updated expiration
    let create_query = "CREATE type::record('leases', $lease_id) CONTENT { resource: $resource, holder: $holder, created_at: time::from::unix($created_at), expires_at: time::from::unix($expires_at) }";

    let create_result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client
            .query(create_query)
            .bind(("lease_id", request.lease_id))
            .bind(("resource", resource.clone()))
            .bind(("holder", holder.clone()))
            .bind(("created_at", now.timestamp()))
            .bind(("expires_at", expires_at.timestamp()))
    ).await;

    match create_result {
        Ok(Ok(_)) => {
            tracing::info!("Lease renewed: {}", request.lease_id);
            Ok((
                StatusCode::OK,
                Json(LeaseResponse {
                    lease_id: request.lease_id,
                    resource,
                    holder,
                    expires_at: expires_at.to_rfc3339(),
                }),
            ))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to renew lease: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout renewing lease");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}
