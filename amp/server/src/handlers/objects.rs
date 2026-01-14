use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use uuid::Uuid;
use serde_json::Value;
use tokio::time::{timeout, Duration};

use crate::{models::AmpObject, AppState};

fn extract_object_id(obj: &AmpObject) -> Uuid {
    match obj {
        AmpObject::Symbol(s) => s.base.id,
        AmpObject::Decision(d) => d.base.id,
        AmpObject::ChangeSet(c) => c.base.id,
        AmpObject::Run(r) => r.base.id,
    }
}

pub async fn create_object(
    State(state): State<AppState>,
    Json(payload): Json<AmpObject>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let object_id = extract_object_id(&payload);

    // Insert with timeout
    let result: Result<Result<Option<Value>, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client
            .insert(("objects", object_id.to_string()))
            .content(payload)
    ).await;

    match result {
        Ok(Ok(_)) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": object_id,
                "created_at": chrono::Utc::now().to_rfc3339()
            })),
        )),
        Ok(Err(e)) => {
            tracing::error!("Failed to create object: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Database operation timed out for object {}", object_id);
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BatchResult {
    id: Uuid,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchResponse {
    results: Vec<BatchResult>,
    summary: BatchSummary,
}

#[derive(Debug, Serialize)]
pub struct BatchSummary {
    total: usize,
    succeeded: usize,
    failed: usize,
}

pub async fn create_objects_batch(
    State(state): State<AppState>,
    Json(payload): Json<Vec<AmpObject>>,
) -> Result<(StatusCode, Json<BatchResponse>), StatusCode> {
    let mut results = Vec::new();
    let total = payload.len();
    let mut succeeded = 0;
    let mut failed = 0;

    for obj in payload {
        let object_id = extract_object_id(&obj);

        let result: Result<Result<Option<Value>, _>, _> = timeout(
            Duration::from_secs(5),
            state.db.client
                .insert(("objects", object_id.to_string()))
                .content(obj)
        ).await;

        match result {
            Ok(Ok(_)) => {
                succeeded += 1;
                results.push(BatchResult {
                    id: object_id,
                    status: "created".to_string(),
                    error: None,
                });
            }
            Ok(Err(e)) => {
                failed += 1;
                tracing::error!("Failed to create object {}: {}", object_id, e);
                results.push(BatchResult {
                    id: object_id,
                    status: "failed".to_string(),
                    error: Some(e.to_string()),
                });
            }
            Err(_) => {
                failed += 1;
                tracing::error!("Timeout creating object {}", object_id);
                results.push(BatchResult {
                    id: object_id,
                    status: "failed".to_string(),
                    error: Some("timeout".to_string()),
                });
            }
        }
    }

    let status_code = if failed == 0 {
        StatusCode::CREATED
    } else if succeeded == 0 {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::from_u16(207).unwrap() // Multi-Status
    };

    Ok((status_code, Json(BatchResponse {
        results,
        summary: BatchSummary { total, succeeded, failed },
    })))
}

pub async fn get_object(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    let result: Result<Result<Option<Value>, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.select(("objects", id.to_string()))
    ).await;

    match result {
        Ok(Ok(Some(mut obj))) => {
            // Replace the record ID with just the UUID
            if let Some(obj_map) = obj.as_object_mut() {
                obj_map.insert("id".to_string(), serde_json::json!(id));
            } else {
                tracing::error!("Unexpected non-object response from database");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(Json(obj))
        }
        Ok(Ok(None)) => Err(StatusCode::NOT_FOUND),
        Ok(Err(e)) => {
            tracing::error!("Failed to retrieve object {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Database operation timed out for object {}", id);
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}
