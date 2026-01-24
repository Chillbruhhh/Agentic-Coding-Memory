use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use crate::services::codebase_parser::{CodebaseParser, FileLog};
use crate::services::index_llm::{AiFileLogInput, AiFileLogOutput, IndexLlmService};
use crate::{
    surreal_json::{normalize_object_ids, take_json_values},
    AppState,
};
use uuid::Uuid;

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
    #[serde(alias = "path")]
    pub file_path: String,
    #[serde(alias = "summary")]
    pub change_description: String,
    #[serde(alias = "linked_changeset")]
    pub changeset_id: Option<String>,
    #[serde(alias = "linked_run")]
    pub run_id: Option<String>,
    pub decision_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AiFileLogRequest {
    pub file_path: String,
    pub language: String,
    pub content_hash: String,
    pub content: String,
    pub symbols: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct AiFileLogResponse {
    pub file_log: AiFileLogOutput,
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

#[derive(Debug, Serialize)]
pub struct FileLogObjectResponse {
    pub file_log: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct FileContentResponse {
    pub path: String,
    pub content: String,
    pub chunks: Vec<String>,
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

    let parser = CodebaseParser::new().map_err(|e| {
        tracing::error!("Failed to create parser: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let root_path =
        map_windows_mount(&request.root_path).unwrap_or_else(|| PathBuf::from(&request.root_path));
    if !root_path.exists() {
        tracing::error!("Path does not exist: {}", request.root_path);
        return Err(StatusCode::BAD_REQUEST);
    }

    let file_logs = parser.parse_codebase(&root_path).map_err(|e| {
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

    let parser = CodebaseParser::new().map_err(|e| {
        tracing::error!("Failed to create parser: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut file_path = PathBuf::from(&request.file_path);
    if !file_path.exists() {
        if let Some(mapped) = map_windows_mount(&request.file_path) {
            file_path = mapped;
        }
    }
    if !file_path.exists() {
        tracing::error!("File does not exist: {}", request.file_path);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Detect language if not provided
    let language = request
        .language
        .unwrap_or_else(|| detect_language(&file_path));

    let file_log = parser.parse_file(&file_path, &language).map_err(|e| {
        tracing::error!("Failed to parse file: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let markdown = parser.generate_file_log_markdown(&file_log);

    // TODO: Store file log in AMP database
    tracing::debug!("Would store file log for: {}", request.file_path);

    Ok(Json(FileLogResponse { file_log, markdown }))
}

/// Update file log with new change information
pub async fn update_file_log(
    State(state): State<AppState>,
    Json(request): Json<UpdateFileLogRequest>,
) -> Result<Json<FileLogResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Updating file log for: {}", request.file_path);

    // First, re-parse the file to get current state
    let parser = CodebaseParser::new().map_err(|e| {
        tracing::error!("Failed to create parser: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create parser", "details": e.to_string()})),
        )
    })?;

    // Resolve the file path
    let file_path = match resolve_file_path(&request.file_path, &state).await {
        Ok(path) => path,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "File not found", "path": request.file_path})),
            ));
        }
    };

    let language = detect_language(&file_path);

    let mut file_log = parser.parse_file(&file_path, &language).map_err(|e| {
        tracing::error!("Failed to parse file: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to parse file", "details": e.to_string()})),
        )
    })?;

    // Add the change to recent changes
    let change_entry = format!(
        "{} · {} {}{}{}",
        chrono::Utc::now().format("%Y-%m-%d"),
        request.change_description,
        request
            .run_id
            .as_ref()
            .map(|id| format!("(run: {})", id))
            .unwrap_or_default(),
        request
            .changeset_id
            .as_ref()
            .map(|id| format!("(cs: {})", id))
            .unwrap_or_default(),
        request
            .decision_id
            .as_ref()
            .map(|id| format!("(decision: {})", id))
            .unwrap_or_default(),
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

    Ok(Json(FileLogResponse { file_log, markdown }))
}

pub async fn generate_ai_file_log(
    State(state): State<AppState>,
    Json(request): Json<AiFileLogRequest>,
) -> Result<Json<AiFileLogResponse>, (StatusCode, Json<serde_json::Value>)> {
    let settings = match state.settings_service.load_settings().await {
        Ok(settings) => settings,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to load settings: {}", err) })),
            ));
        }
    };

    let input = AiFileLogInput {
        path: request.file_path,
        language: request.language,
        content_hash: request.content_hash,
        content: request.content,
        symbols: request.symbols.unwrap_or_default(),
        dependencies: request.dependencies.unwrap_or_default(),
    };

    let service = IndexLlmService::new();
    match service.generate_file_log(&settings, input).await {
        Ok(file_log) => Ok(Json(AiFileLogResponse { file_log })),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("AI file log generation failed: {}", err) })),
        )),
    }
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
    State(state): State<AppState>,
    Path(file_path): Path<String>,
) -> Result<Json<FileLogResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Getting file log for: {}", file_path);

    // Resolve the file path - try multiple strategies
    let resolved_path = match resolve_file_path(&file_path, &state).await {
        Ok(path) => path,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "File not found", "path": file_path})),
            ));
        }
    };

    tracing::debug!("Resolved path: {:?}", resolved_path);

    // TODO: Query the AMP database for the specific file log
    // For now, re-parse the file
    let parser = CodebaseParser::new().map_err(|e| {
        tracing::error!("Failed to create parser: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create parser", "details": e.to_string()})),
        )
    })?;

    let language = detect_language(&resolved_path);

    let file_log = parser.parse_file(&resolved_path, &language).map_err(|e| {
        tracing::error!("Failed to parse file: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to parse file", "details": e.to_string()})),
        )
    })?;

    let markdown = parser.generate_file_log_markdown(&file_log);

    Ok(Json(FileLogResponse { file_log, markdown }))
}

/// Get stored AI file log object by path
pub async fn get_file_log_object(
    State(state): State<AppState>,
    Path(file_path): Path<String>,
) -> Result<Json<FileLogObjectResponse>, (StatusCode, Json<serde_json::Value>)> {
    if let Some(object_id) = parse_object_id(&file_path) {
        let mut response = match state
            .db
            .client
            .query("SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE type = 'FileLog' AND id = type::thing('objects', $id)")
            .bind(("id", object_id.clone()))
            .await
        {
            Ok(response) => response,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to query file log by id: {}", err) })),
                ));
            }
        };

        let mut values = take_json_values(&mut response, 0);
        if !values.is_empty() {
            normalize_object_ids(&mut values);
            let mut file_log = values.remove(0);
            if let Some(map) = file_log.as_object_mut() {
                if map.get("summary_markdown").is_none() {
                    if let Some(summary) = map.get("summary").cloned() {
                        map.insert("summary_markdown".to_string(), summary);
                    }
                }
            }
            return Ok(Json(FileLogObjectResponse { file_log }));
        }

        let mut response = match state
            .db
            .client
            .query("SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE type = 'FileLog' AND file_id = $id LIMIT 1")
            .bind(("id", object_id))
            .await
        {
            Ok(response) => response,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Failed to query file log by file_id: {}", err) })),
                ));
            }
        };

        let mut values = take_json_values(&mut response, 0);
        if !values.is_empty() {
            normalize_object_ids(&mut values);
            let mut file_log = values.remove(0);
            if let Some(map) = file_log.as_object_mut() {
                if map.get("summary_markdown").is_none() {
                    if let Some(summary) = map.get("summary").cloned() {
                        map.insert("summary_markdown".to_string(), summary);
                    }
                }
            }
            return Ok(Json(FileLogObjectResponse { file_log }));
        }
    }

    let normalized = normalize_lookup_path(&file_path);
    let basename = extract_basename(&file_path);

    // Check if input is basename-only (no path separators) - needs early ambiguity check
    let is_basename_only = !file_path.contains('/') && !file_path.contains('\\');

    if is_basename_only {
        // Query all matching file_paths - HashSet will deduplicate
        let ambiguity_query = "SELECT VALUE file_path FROM objects WHERE type = 'FileLog' AND file_path CONTAINS $basename";
        if let Ok(mut response) = state.db.client
            .query(ambiguity_query)
            .bind(("basename", file_path.clone()))
            .await
        {
            let values = take_json_values(&mut response, 0);
            // Values are raw strings from SELECT VALUE, collect unique ones
            let unique_paths: std::collections::HashSet<String> = values.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            if unique_paths.len() > 1 {
                let paths_list: Vec<String> = unique_paths.into_iter().collect();
                return Err((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "Ambiguous path - multiple files match",
                        "input_path": file_path,
                        "matching_files": paths_list,
                        "hint": "Please use a more specific path (e.g., include parent directory)"
                    })),
                ));
            }
        }
    }

    // Tier 1: Try specific path matches first (exact, contains path/norm)
    let specific_query = "SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE type = 'FileLog' AND (file_path = $path OR file_path CONTAINS $path OR file_path = $norm OR file_path CONTAINS $norm) LIMIT 1";
    let mut values = match state
        .db
        .client
        .query(specific_query)
        .bind(("path", file_path.clone()))
        .bind(("norm", normalized.clone()))
        .await
    {
        Ok(mut response) => take_json_values(&mut response, 0),
        Err(err) => {
            tracing::warn!("File log query failed, falling back to scan: {}", err);
            fetch_file_log_fallback(&state, &file_path, &normalized, &basename).await?
        }
    };

    // Tier 2: If no specific match, try basename with ambiguity check
    if values.is_empty() {
        let basename_query = "SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE type = 'FileLog' AND file_path CONTAINS $basename";

        if let Ok(mut response) = state.db.client
            .query(basename_query)
            .bind(("basename", basename.clone()))
            .await
        {
            let basename_values = take_json_values(&mut response, 0);

            // Check for ambiguity - multiple different file paths
            let unique_paths: std::collections::HashSet<String> = basename_values.iter()
                .filter_map(|v| v.get("file_path").and_then(|p| p.as_str()).map(|s| s.to_string()))
                .collect();

            if unique_paths.len() > 1 {
                let paths_list: Vec<String> = unique_paths.into_iter().collect();
                return Err((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "Ambiguous path - multiple files match",
                        "input_path": file_path,
                        "matching_files": paths_list,
                        "hint": "Please use a more specific path (e.g., include parent directory)"
                    })),
                ));
            }

            values = basename_values;
        }
    }

    // Tier 3: Try FileChunk lookup if FileLog not found
    if values.is_empty() {
        let chunk_query = "SELECT DISTINCT file_id, file_path FROM objects WHERE type = 'FileChunk' AND (file_path = $path OR file_path CONTAINS $path OR file_path CONTAINS $norm OR file_path CONTAINS $basename)";
        let mut chunk_response = match state
            .db
            .client
            .query(chunk_query)
            .bind(("path", file_path.clone()))
            .bind(("norm", normalized.clone()))
            .bind(("basename", basename.clone()))
            .await
        {
            Ok(response) => response,
            Err(err) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        serde_json::json!({ "error": format!("Failed to query file chunks: {}", err) }),
                    ),
                ));
            }
        };

        let chunk_values = take_json_values(&mut chunk_response, 0);

        // Check for ambiguity in chunk matches
        let unique_chunk_paths: std::collections::HashSet<String> = chunk_values.iter()
            .filter_map(|v| v.get("file_path").and_then(|p| p.as_str()).map(|s| s.to_string()))
            .collect();

        if unique_chunk_paths.len() > 1 {
            let paths_list: Vec<String> = unique_chunk_paths.into_iter().collect();
            return Err((
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "Ambiguous path - multiple files match",
                    "input_path": file_path,
                    "matching_files": paths_list,
                    "hint": "Please use a more specific path (e.g., include parent directory)"
                })),
            ));
        }

        let found_file_id = chunk_values
            .first()
            .and_then(|v| v.get("file_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(file_id) = found_file_id {
            values = match state
                .db
                .client
                .query("SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE type = 'FileLog' AND file_id = $file_id LIMIT 1")
                .bind(("file_id", file_id))
                .await
            {
                Ok(mut response) => take_json_values(&mut response, 0),
                Err(err) => {
                    tracing::warn!("File log id query failed, falling back to scan: {}", err);
                    fetch_file_log_fallback(&state, &file_path, &normalized, &basename).await?
                }
            };
        }

        if values.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "File log not found", "path": file_path })),
            ));
        }
    }

    normalize_object_ids(&mut values);
    let mut file_log = values.remove(0);
    if let Some(map) = file_log.as_object_mut() {
        if map.get("summary_markdown").is_none() {
            if let Some(summary) = map.get("summary").cloned() {
                map.insert("summary_markdown".to_string(), summary);
            }
        }
    }

    Ok(Json(FileLogObjectResponse { file_log }))
}

fn normalize_lookup_path(path: &str) -> String {
    let mut normalized = path.replace('/', "\\");
    if let Some(stripped) = normalized.strip_prefix(r"\\?\") {
        normalized = stripped.to_string();
    }
    if let Some(stripped) = normalized.strip_prefix(".\\") {
        normalized = stripped.to_string();
    }
    normalized.to_lowercase()
}

fn normalize_file_content_path(path: &str) -> String {
    let mut normalized = path.replace('/', "\\");
    if let Some(stripped) = normalized.strip_prefix(r"\\?\") {
        normalized = stripped.to_string();
    }
    if let Some(stripped) = normalized.strip_prefix(".\\") {
        normalized = stripped.to_string();
    }
    normalized
}

fn parse_object_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let candidate = trimmed.strip_prefix("objects:").unwrap_or(trimmed);
    Uuid::parse_str(candidate).ok().map(|id| id.to_string())
}

fn extract_basename(input: &str) -> String {
    input
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(input)
        .to_lowercase()
}

fn extract_basename_raw(input: &str) -> String {
    input
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(input)
        .to_string()
}

async fn fetch_file_log_fallback(
    state: &AppState,
    raw_path: &str,
    normalized: &str,
    basename: &str,
) -> Result<Vec<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut response = match state
        .db
        .client
        .query("SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE type = 'FileLog' LIMIT 2000")
        .await
    {
        Ok(response) => response,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to scan file logs: {}", err) })),
            ));
        }
    };

    let values = take_json_values(&mut response, 0);
    if values.is_empty() {
        return Ok(values);
    }

    let scored = values.iter().filter_map(|value| {
        let path = value.get("file_path")?.as_str()?.to_lowercase();
        let score = if path == raw_path.to_lowercase() {
            5
        } else if path == normalized {
            4
        } else if path.contains(normalized) {
            3
        } else if path.contains(basename) {
            2
        } else if path.contains(&basename.to_lowercase()) {
            1
        } else {
            0
        };
        if score == 0 {
            None
        } else {
            Some((score, value.clone()))
        }
    });

    let mut best = scored.collect::<Vec<_>>();
    best.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(best.into_iter().take(1).map(|(_, value)| value).collect())
}

#[derive(Debug, Deserialize)]
pub struct FileContentQuery {
    pub max_chars: Option<usize>,
}

/// Get stored file content by path (assembled from FileChunk objects)
pub async fn get_file_content(
    State(state): State<AppState>,
    Path(file_path): Path<String>,
    Query(query): Query<FileContentQuery>,
) -> Result<Json<FileContentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let normalized = normalize_file_content_path(&file_path);
    let basename = extract_basename_raw(&file_path);
    let basename_lower = basename.to_lowercase();
    let query_str = "SELECT content, chunk_index FROM objects WHERE type = 'FileChunk' AND (file_path = $path OR file_path CONTAINS $path OR file_path = $norm OR file_path CONTAINS $norm OR file_path CONTAINS $basename OR file_path CONTAINS $basename_lower) ORDER BY chunk_index ASC";
    let mut response = match state
        .db
        .client
        .query(query_str)
        .bind(("path", file_path.clone()))
        .bind(("norm", normalized.clone()))
        .bind(("basename", basename.clone()))
        .bind(("basename_lower", basename_lower.clone()))
        .await
    {
        Ok(response) => response,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": format!("Failed to query file content: {}", err) }),
                ),
            ));
        }
    };

    let mut values = take_json_values(&mut response, 0);
    normalize_object_ids(&mut values);
    if values.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "File content not found", "path": file_path })),
        ));
    }

    values.sort_by_key(|value| {
        value
            .get("chunk_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    });

    let mut chunks = Vec::new();
    let mut combined = String::new();
    for value in values {
        if let Some(content) = value.get("content").and_then(|v| v.as_str()) {
            chunks.push(content.to_string());
            combined.push_str(content);
        }
    }

    let limited = match query.max_chars {
        Some(limit) => combined.chars().take(limit).collect(),
        None => combined,
    };

    Ok(Json(FileContentResponse {
        path: file_path,
        content: limited,
        chunks,
    }))
}

/// Resolve file path using multiple strategies
async fn resolve_file_path(file_path: &str, state: &AppState) -> Result<PathBuf, StatusCode> {
    if let Some(mapped) = map_windows_mount(file_path) {
        if mapped.exists() {
            return Ok(mapped);
        }
    }
    // Strategy 1: Try as absolute path
    let path = PathBuf::from(file_path);
    if path.is_absolute() && path.exists() {
        return Ok(path);
    }

    // Strategy 2: Try relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let path = cwd.join(file_path);
        if path.exists() {
            return Ok(path);
        }
    }

    // Strategy 3: Try relative to project root if configured
    if let Ok(project_root) = std::env::var("PROJECT_ROOT") {
        let path = PathBuf::from(project_root).join(file_path);
        if path.exists() {
            return Ok(path);
        }
    }

    if let Ok(project_roots) = fetch_project_roots(state).await {
        for root in project_roots {
            if root.as_os_str().is_empty() || root == PathBuf::from(".") {
                continue;
            }
            let candidate = root.join(file_path);
            if candidate.exists() {
                return Ok(candidate);
            }
            if let Some(mapped) = map_windows_mount(&candidate.to_string_lossy()) {
                if mapped.exists() {
                    return Ok(mapped);
                }
            }
        }
    }

    // Strategy 4: Try going up directories to find the file
    if let Ok(cwd) = std::env::current_dir() {
        let mut current = cwd.clone();
        for _ in 0..5 {
            // Try up to 5 levels up
            let path = current.join(file_path);
            if path.exists() {
                return Ok(path);
            }
            if !current.pop() {
                break;
            }
        }
    }

    tracing::error!("Could not resolve file path: {}", file_path);
    tracing::error!("Current directory: {:?}", std::env::current_dir());
    Err(StatusCode::NOT_FOUND)
}

async fn fetch_project_roots(state: &AppState) -> Result<Vec<PathBuf>, StatusCode> {
    let mut response = state
        .db
        .client
        .query("SELECT VALUE path FROM objects WHERE kind = 'project'")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let values = take_json_values(&mut response, 0);
    let mut roots = Vec::new();
    for value in values {
        if let Some(path) = value.as_str() {
            roots.push(PathBuf::from(path));
        }
    }
    Ok(roots)
}

fn map_windows_mount(path: &str) -> Option<PathBuf> {
    let host_root = env::var("AMP_WINDOWS_MOUNT_ROOT").unwrap_or_else(|_| "C:\\Users".to_string());
    let container_root =
        env::var("AMP_WORKSPACE_MOUNT").unwrap_or_else(|_| "/workspace".to_string());

    let normalized = path.strip_prefix(r"\\?\").unwrap_or(path);
    let normalized = normalized.replace('/', "\\");
    let host_root_norm = host_root.replace('/', "\\");

    if !normalized
        .to_lowercase()
        .starts_with(&host_root_norm.to_lowercase())
    {
        return None;
    }

    let rel = normalized[host_root_norm.len()..].trim_start_matches('\\');
    let container_root = container_root.trim_end_matches('/');
    let mapped = if rel.is_empty() {
        container_root.to_string()
    } else {
        format!("{}/{}", container_root, rel.replace('\\', "/"))
    };
    Some(PathBuf::from(mapped))
}

// Helper functions

fn detect_language(file_path: &std::path::PathBuf) -> String {
    if let Some(extension) = file_path.extension() {
        match extension.to_string_lossy().to_lowercase().as_ref() {
            "py" | "pyi" | "pyw" => "python".to_string(),
            "ts" | "tsx" | "mts" | "cts" => "typescript".to_string(),
            "rs" => "rust".to_string(),
            "js" | "jsx" | "mjs" | "cjs" => "javascript".to_string(),
            "go" => "go".to_string(),
            "java" => "java".to_string(),
            "c" | "h" => "c".to_string(),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp".to_string(),
            "cs" => "csharp".to_string(),
            "rb" => "ruby".to_string(),
            "php" => "php".to_string(),
            "swift" => "swift".to_string(),
            "kt" | "kts" => "kotlin".to_string(),
            "scala" => "scala".to_string(),
            "md" | "markdown" => "markdown".to_string(),
            "json" => "json".to_string(),
            "yaml" | "yml" => "yaml".to_string(),
            "toml" => "toml".to_string(),
            "xml" => "xml".to_string(),
            "html" | "htm" => "html".to_string(),
            "css" | "scss" | "sass" | "less" => "css".to_string(),
            "sql" => "sql".to_string(),
            "sh" | "bash" | "zsh" => "shell".to_string(),
            "ps1" | "psm1" | "psd1" => "powershell".to_string(),
            "txt" => "text".to_string(),
            _ => "config".to_string(),
        }
    } else {
        // Handle files without extensions by name
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        match file_name.as_ref() {
            "makefile" | "gnumakefile" => "makefile".to_string(),
            "dockerfile" => "dockerfile".to_string(),
            "jenkinsfile" => "groovy".to_string(),
            "vagrantfile" => "ruby".to_string(),
            "rakefile" | "gemfile" => "ruby".to_string(),
            ".gitignore" | ".gitattributes" | ".gitmodules" => "git".to_string(),
            ".env" | ".env.example" | ".env.local" => "env".to_string(),
            "license" | "licence" => "text".to_string(),
            _ => "config".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteCodebaseRequest {
    pub codebase_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteCodebaseResponse {
    pub success: bool,
    pub message: String,
    pub deleted_counts: DeletedCounts,
}

#[derive(Debug, Serialize)]
pub struct DeletedCounts {
    pub objects: usize,
    pub relationships: usize,
    pub orphaned_edges: usize,
}

/// Delete entire codebase and all related data
pub async fn delete_codebase(
    State(state): State<AppState>,
    Json(request): Json<DeleteCodebaseRequest>,
) -> Result<Json<DeleteCodebaseResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Deleting codebase: {}", request.codebase_id);

    // Delete edges for this codebase only, using the project_id filter.
    let relationship_tables = [
        "defined_in",
        "depends_on",
        "calls",
        "justified_by",
        "modifies",
        "implements",
        "produced",
        "relationships",
    ];

    let mut relationships_result = 0;
    for table in relationship_tables {
        let query = format!(
            "DELETE FROM {} WHERE in IN (SELECT id FROM objects WHERE project_id = $codebase_id) OR out IN (SELECT id FROM objects WHERE project_id = $codebase_id)",
            table
        );
        match state
            .db
            .client
            .query(query)
            .bind(("codebase_id", request.codebase_id.clone()))
            .await
        {
            Ok(mut response) => {
                let values = take_json_values(&mut response, 0);
                relationships_result += values.len();
            }
            Err(err) => {
                tracing::warn!("Failed to delete relationships from {}: {}", table, err);
            }
        }
    }

    // Delete all objects associated with this codebase
    let delete_objects_query = "DELETE FROM objects WHERE project_id = $codebase_id";
    let objects_result = match state
        .db
        .client
        .query(delete_objects_query)
        .bind(("codebase_id", request.codebase_id.clone()))
        .await
    {
        Ok(mut response) => {
            let values = take_json_values(&mut response, 0);
            values.len()
        }
        Err(err) => {
            tracing::error!("Failed to delete objects: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to delete objects: {}", err) })),
            ));
        }
    };

    tracing::info!(
        "Deleted codebase {}: {} objects, {} relationships",
        request.codebase_id,
        objects_result,
        relationships_result
    );

    Ok(Json(DeleteCodebaseResponse {
        success: true,
        message: format!("Successfully deleted codebase {}", request.codebase_id),
        deleted_counts: DeletedCounts {
            objects: objects_result,
            relationships: relationships_result,
            orphaned_edges: 0,
        },
    }))
}

// ============================================================================
// File Sync - Synchronize file state across all memory layers
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct FileSyncRequest {
    pub path: String,
    pub action: String, // "create", "edit", "delete"
    pub summary: String,
    pub run_id: Option<String>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileSyncResponse {
    pub file_id: String,
    pub action: String,
    pub layers_updated: LayersUpdated,
    pub audit_entry_added: bool,
    pub chunks_replaced: usize,
    pub relationships_updated: usize,
}

#[derive(Debug, Serialize)]
pub struct LayersUpdated {
    pub temporal: bool,
    pub vector: bool,
    pub graph: bool,
}

/// Sync file state across all memory layers (temporal, vector, graph)
/// This is the unified write endpoint that keeps the codebase index in sync
pub async fn sync_file(
    State(state): State<AppState>,
    Json(request): Json<FileSyncRequest>,
) -> Result<Json<FileSyncResponse>, (StatusCode, Json<serde_json::Value>)> {
    use crate::services::chunking::ChunkingService;

    tracing::info!("Syncing file: {} (action: {})", request.path, request.action);

    let action = request.action.to_lowercase();
    let mut layers_updated = LayersUpdated {
        temporal: false,
        vector: false,
        graph: false,
    };
    let mut chunks_replaced = 0;
    let mut relationships_updated = 0;

    // Try to find existing file_id and file_path by flexible path matching
    // Use tiered matching: exact/specific first, then basename (with ambiguity check)
    let normalized = normalize_lookup_path(&request.path);
    let basename = extract_basename(&request.path);

    // Check if input is basename-only (no path separators) - needs ambiguity check
    let is_basename_only = !request.path.contains('/') && !request.path.contains('\\');

    // If basename-only, check for ambiguity FIRST before any matching
    if is_basename_only {
        // Query all matching file_paths - HashSet will deduplicate
        let ambiguity_query = "SELECT VALUE file_path FROM objects WHERE type = 'FileLog' AND file_path CONTAINS $basename";
        if let Ok(mut response) = state.db.client
            .query(ambiguity_query)
            .bind(("basename", request.path.clone()))
            .await
        {
            let values = take_json_values(&mut response, 0);
            // Values are raw strings from SELECT VALUE, collect unique ones
            let unique_paths: std::collections::HashSet<String> = values.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            if unique_paths.len() > 1 {
                let paths_list: Vec<String> = unique_paths.into_iter().collect();
                tracing::warn!("Ambiguous basename '{}' matches {} files", request.path, paths_list.len());
                return Err((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "Ambiguous path - multiple files match",
                        "input_path": request.path,
                        "matching_files": paths_list,
                        "hint": "Please use a more specific path (e.g., include parent directory)"
                    })),
                ));
            }
        }
    }

    // Tier 1: Try exact or specific path matches first
    let specific_query = "SELECT file_id, file_path FROM objects WHERE (type = 'FileLog' OR type = 'FileChunk') AND (file_path = $path OR file_path CONTAINS $path OR file_path = $norm OR file_path CONTAINS $norm) LIMIT 1";

    let (mut existing_file_id, mut existing_file_path) = match state.db.client
        .query(specific_query)
        .bind(("path", request.path.clone()))
        .bind(("norm", normalized.clone()))
        .await
    {
        Ok(mut response) => {
            let values = take_json_values(&mut response, 0);
            if let Some(record) = values.first() {
                let file_id = record.get("file_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let file_path = record.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string());
                (file_id, file_path)
            } else {
                (None, None)
            }
        }
        Err(_) => (None, None),
    };

    // Tier 2: If no specific match, try basename - but check for ambiguity
    if existing_file_id.is_none() {
        let basename_query = "SELECT DISTINCT file_id, file_path FROM objects WHERE (type = 'FileLog' OR type = 'FileChunk') AND file_path CONTAINS $basename";

        if let Ok(mut response) = state.db.client
            .query(basename_query)
            .bind(("basename", basename.clone()))
            .await
        {
            let values = take_json_values(&mut response, 0);

            // Deduplicate by file_path (FileLog and FileChunk may have same path)
            let unique_paths: std::collections::HashSet<String> = values.iter()
                .filter_map(|v| v.get("file_path").and_then(|p| p.as_str()).map(|s| s.to_string()))
                .collect();

            if unique_paths.len() > 1 {
                // Ambiguous match - multiple files with same basename
                let paths_list: Vec<String> = unique_paths.into_iter().collect();
                tracing::warn!("Ambiguous path '{}' matches {} files", request.path, paths_list.len());
                return Err((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "Ambiguous path - multiple files match",
                        "input_path": request.path,
                        "matching_files": paths_list,
                        "hint": "Please use a more specific path (e.g., include parent directory)"
                    })),
                ));
            } else if let Some(record) = values.first() {
                // Single match - safe to use
                existing_file_id = record.get("file_id").and_then(|v| v.as_str()).map(|s| s.to_string());
                existing_file_path = record.get("file_path").and_then(|v| v.as_str()).map(|s| s.to_string());
            }
        }
    }

    // Use existing file_id if found, otherwise generate new one from resolved path
    use sha2::{Sha256, Digest};
    let file_id = if let Some(id) = existing_file_id {
        tracing::debug!("Found existing file_id: {}", id);
        id
    } else {
        // Generate from the input path (for new files)
        tracing::debug!("No existing file_id found, generating new one for: {}", request.path);
        let mut hasher = Sha256::new();
        hasher.update(request.path.as_bytes());
        format!("file-{}", hex::encode(&hasher.finalize()[..16]))
    };

    // Handle delete action
    if action == "delete" {
        // Delete FileChunks
        let delete_chunks_query = "DELETE FROM objects WHERE type = 'FileChunk' AND file_id = $file_id";
        if let Ok(mut response) = state.db.client
            .query(delete_chunks_query)
            .bind(("file_id", file_id.clone()))
            .await
        {
            chunks_replaced = take_json_values(&mut response, 0).len();
            layers_updated.vector = true;
        }

        // Delete relationships for this file
        let relationship_tables = ["defined_in", "depends_on", "calls", "modifies"];
        for table in relationship_tables {
            let query = format!(
                "DELETE FROM {} WHERE in IN (SELECT id FROM objects WHERE file_id = $file_id) OR out IN (SELECT id FROM objects WHERE file_id = $file_id)",
                table
            );
            if let Ok(mut response) = state.db.client
                .query(&query)
                .bind(("file_id", file_id.clone()))
                .await
            {
                relationships_updated += take_json_values(&mut response, 0).len();
            }
        }
        layers_updated.graph = true;

        // Update FileLog with deletion audit entry (soft delete - keep the log)
        let audit_entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "action": "delete",
            "summary": request.summary,
            "run_id": request.run_id,
            "agent_id": request.agent_id
        });

        let update_query = "UPDATE objects SET audit_trail = array::push(audit_trail, $entry), updated_at = time::now() WHERE type = 'FileLog' AND file_id = $file_id";
        if state.db.client
            .query(update_query)
            .bind(("file_id", file_id.clone()))
            .bind(("entry", audit_entry))
            .await
            .is_ok()
        {
            layers_updated.temporal = true;
        }

        return Ok(Json(FileSyncResponse {
            file_id,
            action,
            layers_updated,
            audit_entry_added: true,
            chunks_replaced,
            relationships_updated,
        }));
    }

    // For create/edit, we need to parse the file
    // First try the stored path if we found one, then fall back to resolution
    let file_path = if let Some(stored_path) = &existing_file_path {
        // Try the stored path first
        match resolve_file_path(stored_path, &state).await {
            Ok(path) => path,
            Err(_) => {
                // Fall back to request path resolution
                match resolve_file_path(&request.path, &state).await {
                    Ok(path) => path,
                    Err(_) => {
                        tracing::error!("Could not resolve file path: {} or stored path: {}", request.path, stored_path);
                        return Err((
                            StatusCode::NOT_FOUND,
                            Json(serde_json::json!({ "error": "File not found", "path": request.path, "stored_path": stored_path })),
                        ));
                    }
                }
            }
        }
    } else {
        match resolve_file_path(&request.path, &state).await {
            Ok(path) => path,
            Err(_) => {
                tracing::error!("Could not resolve file path: {}", request.path);
                if let Ok(cwd) = std::env::current_dir() {
                    tracing::error!("Current directory: {:?}", cwd);
                }
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({ "error": "File not found", "path": request.path })),
                ));
            }
        }
    };

    // Read file content
    let content = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to read file: {}", err) })),
            ));
        }
    };

    let language = detect_language(&file_path);
    let parser = CodebaseParser::new().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to create parser: {}", e) })),
        )
    })?;

    let file_log = parser.parse_file(&file_path, &language).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to parse file: {}", e) })),
        )
    })?;

    // Extract symbol names and dependencies from parsed FileLog
    let symbol_names: Vec<String> = file_log.symbols.iter().map(|s| s.name.clone()).collect();
    let deps: Vec<String> = file_log.dependencies.imports.clone();

    // Generate a summary from symbols
    let summary = if symbol_names.is_empty() {
        format!("{} file", language)
    } else {
        let top_symbols: Vec<&str> = symbol_names.iter().take(5).map(|s| s.as_str()).collect();
        format!("{} file with: {}", language, top_symbols.join(", "))
    };

    // --- TEMPORAL LAYER: Update/Create FileLog with audit trail ---
    let audit_entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "action": action,
        "summary": request.summary,
        "run_id": request.run_id,
        "agent_id": request.agent_id
    });

    // Check if FileLog exists
    let check_query = "SELECT VALUE count() FROM objects WHERE type = 'FileLog' AND file_id = $file_id";
    let exists = match state.db.client
        .query(check_query)
        .bind(("file_id", file_id.clone()))
        .await
    {
        Ok(mut response) => {
            let values = take_json_values(&mut response, 0);
            values.first().and_then(|v| v.as_i64()).unwrap_or(0) > 0
        }
        Err(_) => false,
    };

    if exists {
        // Update existing FileLog
        let update_query = r#"
            UPDATE objects SET
                summary = $summary,
                key_symbols = $symbols,
                dependencies = $deps,
                audit_trail = array::push(audit_trail, $entry),
                change_count = change_count + 1,
                updated_at = time::now()
            WHERE type = 'FileLog' AND file_id = $file_id
        "#;

        if state.db.client
            .query(update_query)
            .bind(("file_id", file_id.clone()))
            .bind(("summary", summary.clone()))
            .bind(("symbols", symbol_names.clone()))
            .bind(("deps", deps.clone()))
            .bind(("entry", audit_entry))
            .await
            .is_ok()
        {
            layers_updated.temporal = true;
        }
    } else {
        // Create new FileLog
        let create_query = r#"
            CREATE objects SET
                id = type::thing('objects', $id),
                type = 'FileLog',
                file_path = $path,
                file_id = $file_id,
                summary = $summary,
                key_symbols = $symbols,
                dependencies = $deps,
                audit_trail = [$entry],
                change_count = 1,
                created_at = time::now(),
                updated_at = time::now()
        "#;

        let log_id = Uuid::new_v4().to_string();
        if state.db.client
            .query(create_query)
            .bind(("id", log_id))
            .bind(("path", request.path.clone()))
            .bind(("file_id", file_id.clone()))
            .bind(("summary", summary.clone()))
            .bind(("symbols", symbol_names.clone()))
            .bind(("deps", deps.clone()))
            .bind(("entry", audit_entry))
            .await
            .is_ok()
        {
            layers_updated.temporal = true;
        }
    }

    // --- VECTOR LAYER: Re-chunk and generate embeddings ---

    // First, delete existing chunks for this file
    let delete_chunks_query = "DELETE FROM objects WHERE type = 'FileChunk' AND file_id = $file_id";
    let _ = state.db.client
        .query(delete_chunks_query)
        .bind(("file_id", file_id.clone()))
        .await;

    // Chunk the content with 100-token overlap
    let chunking_service = ChunkingService::new();
    let chunks = chunking_service.chunk_file(&content, &language);

    // Generate embeddings and store chunks
    for (idx, chunk) in chunks.iter().enumerate() {
        let embedding = if state.embedding_service.is_enabled() {
            state.embedding_service.generate_embedding(&chunk.content).await.ok()
        } else {
            None
        };

        let chunk_id = Uuid::new_v4().to_string();
        let embedding_str = embedding
            .as_ref()
            .map(|e| format!("[{}]", e.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ")))
            .unwrap_or_else(|| "NONE".to_string());

        let insert_query = format!(r#"
            CREATE objects SET
                id = type::thing('objects', $id),
                type = 'FileChunk',
                file_path = $path,
                file_id = $file_id,
                chunk_index = $idx,
                start_line = $start,
                end_line = $end,
                token_count = $tokens,
                content = $content,
                content_hash = $hash,
                language = $lang,
                embedding = {},
                created_at = time::now(),
                updated_at = time::now()
        "#, embedding_str);

        if state.db.client
            .query(&insert_query)
            .bind(("id", chunk_id))
            .bind(("path", request.path.clone()))
            .bind(("file_id", file_id.clone()))
            .bind(("idx", idx as i32))
            .bind(("start", chunk.start_line as i32))
            .bind(("end", chunk.end_line as i32))
            .bind(("tokens", chunk.token_count as i32))
            .bind(("content", chunk.content.clone()))
            .bind(("hash", chunk.hash.clone()))
            .bind(("lang", language.clone()))
            .await
            .is_ok()
        {
            chunks_replaced += 1;
        }
    }

    if chunks_replaced > 0 {
        layers_updated.vector = true;
    }

    // --- GRAPH LAYER: Update relationships based on parsed dependencies ---

    // Delete old relationships for this file
    let relationship_tables = ["depends_on", "calls"];
    for table in &relationship_tables {
        let query = format!(
            "DELETE FROM {} WHERE in IN (SELECT id FROM objects WHERE file_id = $file_id)",
            table
        );
        let _ = state.db.client
            .query(&query)
            .bind(("file_id", file_id.clone()))
            .await;
    }

    // Create new dependency relationships
    for dep in &deps {
        // Try to find the target file by dependency name
        let find_query = "SELECT VALUE id FROM objects WHERE type = 'FileLog' AND (file_path CONTAINS $dep OR key_symbols CONTAINS $dep) LIMIT 1";
        if let Ok(mut response) = state.db.client
            .query(find_query)
            .bind(("dep", dep.clone()))
            .await
        {
            let values = take_json_values(&mut response, 0);
            if let Some(target_id) = values.first().and_then(|v| v.as_str()) {
                let relate_query = format!(
                    "RELATE (SELECT id FROM objects WHERE type = 'FileLog' AND file_id = $file_id LIMIT 1)->depends_on->{} SET created_at = time::now()",
                    target_id
                );
                if state.db.client
                    .query(&relate_query)
                    .bind(("file_id", file_id.clone()))
                    .await
                    .is_ok()
                {
                    relationships_updated += 1;
                }
            }
        }
    }

    if relationships_updated > 0 {
        layers_updated.graph = true;
    }

    tracing::info!(
        "File sync complete: {} - temporal={}, vector={} ({} chunks), graph={} ({} rels)",
        request.path,
        layers_updated.temporal,
        layers_updated.vector,
        chunks_replaced,
        layers_updated.graph,
        relationships_updated
    );

    Ok(Json(FileSyncResponse {
        file_id,
        action,
        layers_updated,
        audit_entry_added: true,
        chunks_replaced,
        relationships_updated,
    }))
}
