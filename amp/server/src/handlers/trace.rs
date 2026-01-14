use axum::{extract::{Path, State}, http::StatusCode, response::Json};
use serde::Serialize;
use uuid::Uuid;

use crate::{handlers::query::QueryRequest, AppState};

#[derive(Debug, Serialize)]
pub struct TraceResponse {
    pub trace_id: Uuid,
    pub query: QueryRequest,
    pub steps: Vec<TraceStep>,
    pub total_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct TraceStep {
    pub step: String,
    pub description: String,
    pub time_ms: u64,
    pub results_count: usize,
}

pub async fn get_trace(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<TraceResponse>, StatusCode> {
    // TODO: Implement trace retrieval
    Err(StatusCode::NOT_IMPLEMENTED)
}
