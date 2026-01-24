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
