use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use crate::services::codebase_parser::{CodebaseParser, FileLog};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ParseCodebaseRequest {
    pub root_path: String,
    pub project_id: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ParseFileRequest {
    pub file_path: String,
    pub language: Option<String>,
    pub project_id: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFileLogRequest {
    pub file_path: String,
    pub change_description: String,
    pub changeset_id: Option<String>,
    pub run_id: Option<String>,
    pub decision_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParseCodebaseResponse {
    pub success: bool,
    pub files_parsed: usize,
    pub file_logs: HashMap<String, FileLog>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FileLogResponse {
    pub file_log: FileLog,
    pub markdown: String,
}

#[derive(Debug, Deserialize)]
pub struct GetFileLogsQuery {
    pub project_id: Option<String>,
    pub language: Option<String>,
    pub limit: Option<usize>,
}

/// Parse entire codebase and create file logs
pub async fn parse_codebase(
    State(_state): State<AppState>,
    Json(request): Json<ParseCodebaseRequest>,
) -> Result<Json<ParseCodebaseResponse>, StatusCode> {
    tracing::info!("Parsing codebase at: {}", request.root_path);
    
    let parser = CodebaseParser::new()
        .map_err(|e| {
            tracing::error!("Failed to create parser: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let root_path = PathBuf::from(&request.root_path);
    if !root_path.exists() {
        tracing::error!("Path does not exist: {}", request.root_path);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let file_logs = parser.parse_codebase(&root_path)
        .map_err(|e| {
            tracing::error!("Failed to parse codebase: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let files_parsed = file_logs.len();
    
    // Store file logs in AMP database
    for (file_path, _file_log) in &file_logs {
        // TODO: Implement actual storage
        tracing::debug!("Would store file log for: {}", file_path);
    }
    
    Ok(Json(ParseCodebaseResponse {
        success: true,
        files_parsed,
        file_logs,
        errors: Vec::new(),
    }))
}

/// Parse single file and create/update file log
pub async fn parse_file(
    State(_state): State<AppState>,
    Json(request): Json<ParseFileRequest>,
) -> Result<Json<FileLogResponse>, StatusCode> {
    tracing::info!("Parsing file: {}", request.file_path);
    
    let parser = CodebaseParser::new()
        .map_err(|e| {
            tracing::error!("Failed to create parser: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let file_path = PathBuf::from(&request.file_path);
    if !file_path.exists() {
        tracing::error!("File does not exist: {}", request.file_path);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Detect language if not provided
    let language = request.language.unwrap_or_else(|| {
        detect_language(&file_path)
    });
    
    let file_log = parser.parse_file(&file_path, &language)
        .map_err(|e| {
            tracing::error!("Failed to parse file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let markdown = parser.generate_file_log_markdown(&file_log);
    
    // TODO: Store file log in AMP database
    tracing::debug!("Would store file log for: {}", request.file_path);
    
    Ok(Json(FileLogResponse {
        file_log,
        markdown,
    }))
}

/// Update file log with new change information
pub async fn update_file_log(
    State(_state): State<AppState>,
    Json(request): Json<UpdateFileLogRequest>,
) -> Result<Json<FileLogResponse>, StatusCode> {
    tracing::info!("Updating file log for: {}", request.file_path);
    
    // First, re-parse the file to get current state
    let parser = CodebaseParser::new()
        .map_err(|e| {
            tracing::error!("Failed to create parser: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let file_path = PathBuf::from(&request.file_path);
    let language = detect_language(&file_path);
    
    let mut file_log = parser.parse_file(&file_path, &language)
        .map_err(|e| {
            tracing::error!("Failed to parse file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Add the change to recent changes
    let change_entry = format!(
        "{} · {} {}{}{}",
        chrono::Utc::now().format("%Y-%m-%d"),
        request.change_description,
        request.run_id.as_ref().map(|id| format!("(run: {})", id)).unwrap_or_default(),
        request.changeset_id.as_ref().map(|id| format!("(cs: {})", id)).unwrap_or_default(),
        request.decision_id.as_ref().map(|id| format!("(decision: {})", id)).unwrap_or_default(),
    );
    
    file_log.recent_changes.insert(0, change_entry);
    
    // Keep only last 10 changes
    if file_log.recent_changes.len() > 10 {
        file_log.recent_changes.truncate(10);
    }
    
    // Add linked decision if provided
    if let Some(decision_id) = request.decision_id {
        if !file_log.linked_decisions.contains(&decision_id) {
            file_log.linked_decisions.push(decision_id);
        }
    }
    
    let markdown = parser.generate_file_log_markdown(&file_log);
    
    // TODO: Store updated file log
    tracing::debug!("Would store updated file log for: {}", request.file_path);
    
    Ok(Json(FileLogResponse {
        file_log,
        markdown,
    }))
}

/// Get file logs with optional filtering
pub async fn get_file_logs(
    State(_state): State<AppState>,
    Query(_query): Query<GetFileLogsQuery>,
) -> Result<Json<Vec<FileLog>>, StatusCode> {
    // TODO: Query the AMP database for stored file logs
    Ok(Json(Vec::new()))
}

/// Get specific file log by path
pub async fn get_file_log(
    State(_state): State<AppState>,
    Path(file_path): Path<String>,
) -> Result<Json<FileLogResponse>, StatusCode> {
    tracing::info!("Getting file log for: {}", file_path);
    
    // TODO: Query the AMP database for the specific file log
    // For now, re-parse the file
    let parser = CodebaseParser::new()
        .map_err(|e| {
            tracing::error!("Failed to create parser: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let path = PathBuf::from(&file_path);
    let language = detect_language(&path);
    
    let file_log = parser.parse_file(&path, &language)
        .map_err(|e| {
            tracing::error!("Failed to parse file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let markdown = parser.generate_file_log_markdown(&file_log);
    
    Ok(Json(FileLogResponse {
        file_log,
        markdown,
    }))
}

// Helper functions

fn detect_language(file_path: &std::path::PathBuf) -> String {
    if let Some(extension) = file_path.extension() {
        match extension.to_string_lossy().as_ref() {
            "py" => "python".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}
