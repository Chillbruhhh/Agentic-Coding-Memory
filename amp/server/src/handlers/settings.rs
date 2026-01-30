use crate::models::settings::SettingsConfig;
use crate::AppState;
use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    match state.settings_service.load_settings().await {
        Ok(settings) => (StatusCode::OK, Json(settings)).into_response(),
        Err(e) => {
            tracing::error!("Failed to load settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to load settings: {}", e)
                })),
            )
                .into_response()
        }
    }
}

pub async fn update_settings(
    State(state): State<AppState>,
    payload: Result<Json<SettingsConfig>, JsonRejection>,
) -> impl IntoResponse {
    let settings = match payload {
        Ok(Json(s)) => s,
        Err(rejection) => {
            tracing::error!("Failed to parse settings JSON: {}", rejection);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid settings format: {}", rejection)
                })),
            )
                .into_response();
        }
    };

    match state.settings_service.save_settings(settings).await {
        Ok(saved_settings) => (StatusCode::OK, Json(saved_settings)).into_response(),
        Err(e) => {
            tracing::error!("Failed to save settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to save settings: {}", e)
                })),
            )
                .into_response()
        }
    }
}

pub async fn nuclear_delete(State(state): State<AppState>) -> impl IntoResponse {
    tracing::warn!("NUCLEAR DELETE initiated - deleting ALL data from AMP");

    // Execute nuclear delete queries
    let queries = vec![
        "DELETE FROM objects;",
        "DELETE FROM relationships;",
        "DELETE FROM defined_in WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);",
        "DELETE FROM depends_on WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);",
        "DELETE FROM calls WHERE in NOT IN (SELECT id FROM objects) OR out NOT IN (SELECT id FROM objects);",
    ];

    let mut deleted_counts = Vec::new();

    for query in queries {
        match state.db.client.query(query).await {
            Ok(_) => {
                tracing::info!("Executed: {}", query);
                deleted_counts.push(query.to_string());
            }
            Err(e) => {
                tracing::error!("Failed to execute {}: {}", query, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Nuclear delete failed: {}", e),
                        "query": query
                    })),
                )
                    .into_response();
            }
        }
    }

    tracing::warn!("NUCLEAR DELETE completed - all data removed");

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "All data deleted successfully",
            "queries_executed": deleted_counts.len()
        })),
    )
        .into_response()
}
