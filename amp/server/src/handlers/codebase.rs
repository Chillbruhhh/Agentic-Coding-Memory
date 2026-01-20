use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::services::codebase_parser::{CodebaseParser, FileLog};
use crate::services::index_llm::{AiFileLogInput, AiFileLogOutput, IndexLlmService};
use crate::{surreal_json::{normalize_object_ids, take_json_values}, AppState};
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
    State(state): State<AppState>,
    Json(request): Json<UpdateFileLogRequest>,
) -> Result<Json<FileLogResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("Updating file log for: {}", request.file_path);
    
    // First, re-parse the file to get current state
    let parser = CodebaseParser::new()
        .map_err(|e| {
            tracing::error!("Failed to create parser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create parser", "details": e.to_string()}))
            )
        })?;
    
    // Resolve the file path
    let file_path = match resolve_file_path(&request.file_path, &state) {
        Ok(path) => path,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "File not found", "path": request.file_path}))
            ));
        }
    };
    
    let language = detect_language(&file_path);
    
    let mut file_log = parser.parse_file(&file_path, &language)
        .map_err(|e| {
            tracing::error!("Failed to parse file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to parse file", "details": e.to_string()}))
            )
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
    let resolved_path = match resolve_file_path(&file_path, &state) {
        Ok(path) => path,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "File not found", "path": file_path}))
            ));
        }
    };
    
    tracing::debug!("Resolved path: {:?}", resolved_path);
    
    // TODO: Query the AMP database for the specific file log
    // For now, re-parse the file
    let parser = CodebaseParser::new()
        .map_err(|e| {
            tracing::error!("Failed to create parser: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create parser", "details": e.to_string()}))
            )
        })?;
    
    let language = detect_language(&resolved_path);
    
    let file_log = parser.parse_file(&resolved_path, &language)
        .map_err(|e| {
            tracing::error!("Failed to parse file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to parse file", "details": e.to_string()}))
            )
        })?;
    
    let markdown = parser.generate_file_log_markdown(&file_log);
    
    Ok(Json(FileLogResponse {
        file_log,
        markdown,
    }))
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
            .query("SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE id = type::thing('objects', $id)")
            .bind(("id", object_id))
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
    }

    let normalized = normalize_lookup_path(&file_path);
    let basename = extract_basename(&file_path);
    let query = "SELECT VALUE { id: string::concat(id), type: type, file_path: file_path, file_id: file_id, summary: summary, summary_markdown: summary_markdown, purpose: purpose, key_symbols: key_symbols, dependencies: dependencies, notes: notes, updated_at: updated_at, created_at: created_at, project_id: project_id, tenant_id: tenant_id } FROM objects WHERE type = 'FileLog' AND (file_path = $path OR file_path CONTAINS $path OR file_path = $norm OR file_path CONTAINS $norm OR file_path CONTAINS $basename) LIMIT 1";
    let mut values = match state
        .db
        .client
        .query(query)
        .bind(("path", file_path.clone()))
        .bind(("norm", normalized.clone()))
        .bind(("basename", basename.clone()))
        .await
    {
        Ok(mut response) => take_json_values(&mut response, 0),
        Err(err) => {
            tracing::warn!("File log query failed, falling back to scan: {}", err);
            fetch_file_log_fallback(&state, &file_path, &normalized, &basename).await?
        }
    };

    if values.is_empty() {
        let chunk_query = "SELECT VALUE file_id FROM objects WHERE type = 'FileChunk' AND (file_path = $path OR file_path CONTAINS $path OR file_path CONTAINS $norm OR file_path CONTAINS $basename) LIMIT 1";
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
                    Json(serde_json::json!({ "error": format!("Failed to query file chunks: {}", err) })),
                ));
            }
        };

        let chunk_values = take_json_values(&mut chunk_response, 0);
        let file_id = chunk_values
            .iter()
            .find_map(|value| value.as_str().map(|val| val.to_string()));

        if let Some(file_id) = file_id {
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
    if let Some(stripped) = normalized.strip_prefix(".\\") {
        normalized = stripped.to_string();
    }
    normalized.to_lowercase()
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

    let mut values = take_json_values(&mut response, 0);
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
    let query_str = "SELECT content, chunk_index FROM objects WHERE type = 'FileChunk' AND (file_path = $path OR file_path CONTAINS $path) ORDER BY chunk_index ASC";
    let mut response = match state
        .db
        .client
        .query(query_str)
        .bind(("path", file_path.clone()))
        .await
    {
        Ok(response) => response,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to query file content: {}", err) })),
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
fn resolve_file_path(file_path: &str, _state: &AppState) -> Result<PathBuf, StatusCode> {
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
    
    // Strategy 4: Try going up directories to find the file
    if let Ok(cwd) = std::env::current_dir() {
        let mut current = cwd.clone();
        for _ in 0..5 {  // Try up to 5 levels up
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
        let file_name = file_path.file_name()
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
