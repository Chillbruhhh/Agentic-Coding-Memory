use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use tokio::time::{timeout, Duration};
use crate::{
    models::analytics::AnalyticsData,
    AppState,
};

pub async fn get_analytics(
    State(state): State<AppState>,
) -> Result<Json<AnalyticsData>, StatusCode> {
    let result = timeout(
        Duration::from_secs(5),
        state.analytics_service.get_analytics()
    ).await
    .map_err(|_| {
        tracing::error!("Analytics request timeout after 5 seconds");
        StatusCode::REQUEST_TIMEOUT
    })?
    .map_err(|e| {
        tracing::error!("Failed to get analytics: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(result))
}
