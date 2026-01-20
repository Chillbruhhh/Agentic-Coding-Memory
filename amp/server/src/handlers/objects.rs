use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use uuid::Uuid;
use serde_json::Value;
use tokio::time::{timeout, Duration};
use crate::{
    models::AmpObject,
    surreal_json::{normalize_object_id, take_json_values},
    AppState,
};

fn extract_object_id(obj: &AmpObject) -> Uuid {
    match obj {
        AmpObject::Symbol(s) => s.base.id,
        AmpObject::Decision(d) => d.base.id,
        AmpObject::ChangeSet(c) => c.base.id,
        AmpObject::Run(r) => r.base.id,
        AmpObject::FileChunk(f) => f.base.id,
        AmpObject::FileLog(f) => f.base.id,
    }
}

fn payload_to_content_value(payload: &AmpObject) -> Result<Value, StatusCode> {
    // Serialize the specific object type, not the enum wrapper
    let mut value = match payload {
        AmpObject::Symbol(s) => serde_json::to_value(s),
        AmpObject::Decision(d) => serde_json::to_value(d),
        AmpObject::ChangeSet(c) => serde_json::to_value(c),
        AmpObject::Run(r) => serde_json::to_value(r),
        AmpObject::FileChunk(f) => serde_json::to_value(f),
        AmpObject::FileLog(f) => serde_json::to_value(f),
    }.map_err(|err| {
        tracing::error!("Failed to serialize payload: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Set timestamps if not provided
    if let Some(map) = value.as_object_mut() {
        let now = chrono::Utc::now().to_rfc3339();
        if !map.contains_key("created_at") || map.get("created_at").map(|v| v.is_null()).unwrap_or(true) {
            map.insert("created_at".to_string(), serde_json::Value::String(now.clone()));
        }
        if !map.contains_key("updated_at") || map.get("updated_at").map(|v| v.is_null()).unwrap_or(true) {
            map.insert("updated_at".to_string(), serde_json::Value::String(now));
        }
    }

    // Convert to JSON string and back to ensure all enums are plain strings
    // This prevents SurrealDB from storing them as its own enum types
    let json_str = serde_json::to_string(&value).map_err(|e| {
        tracing::error!("Failed to stringify JSON: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let plain_value = serde_json::from_str(&json_str).map_err(|e| {
        tracing::error!("Failed to parse JSON: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(plain_value)
}

fn set_embedding(mut obj: AmpObject, embedding: Option<Vec<f32>>) -> AmpObject {
    match &mut obj {
        AmpObject::Symbol(s) => s.base.embedding = embedding,
        AmpObject::Decision(d) => d.base.embedding = embedding,
        AmpObject::ChangeSet(c) => c.base.embedding = embedding,
        AmpObject::Run(r) => r.base.embedding = embedding,
        AmpObject::FileChunk(f) => f.base.embedding = embedding,
        AmpObject::FileLog(f) => f.base.embedding = embedding,
    }
    obj
}

fn extract_embedding_text(obj: &AmpObject) -> String {
    let mut parts = Vec::new();

    match obj {
        AmpObject::Symbol(symbol) => {
            parts.push(symbol.base.provenance.summary.clone());
            parts.push(symbol.name.clone());
            parts.push(format!("{:?}", symbol.kind));
            parts.push(symbol.path.clone());
            parts.push(symbol.language.clone());

            if let Some(signature) = &symbol.signature {
                parts.push(signature.clone());
            }
            if let Some(documentation) = &symbol.documentation {
                parts.push(documentation.clone());
            }
            if let Some(content_hash) = &symbol.content_hash {
                parts.push(content_hash.clone());
            }
        }
        AmpObject::Decision(decision) => {
            parts.push(decision.base.provenance.summary.clone());
            parts.push(decision.title.clone());
            parts.push(decision.problem.clone());
            parts.push(decision.rationale.clone());
            parts.push(decision.outcome.clone());

            if let Some(status) = &decision.status {
                parts.push(format!("{:?}", status));
            }

            if let Some(options) = &decision.options {
                for option in options {
                    parts.push(option.name.clone());
                    parts.push(option.description.clone());
                    if let Some(pros) = &option.pros {
                        parts.extend(pros.clone());
                    }
                    if let Some(cons) = &option.cons {
                        parts.extend(cons.clone());
                    }
                }
            }
        }
        AmpObject::ChangeSet(changeset) => {
            parts.push(changeset.base.provenance.summary.clone());
            parts.push(changeset.title.clone());

            if let Some(description) = &changeset.description {
                parts.push(description.clone());
            }
            if let Some(diff) = &changeset.diff {
                parts.push(diff.clone());
            }
            parts.extend(changeset.files_changed.clone());
            parts.push(format!("{:?}", changeset.status));

            if let Some(commit_hash) = &changeset.commit_hash {
                parts.push(commit_hash.clone());
            }
            if let Some(tests) = &changeset.tests {
                for test in tests {
                    parts.push(test.name.clone());
                    parts.push(format!("{:?}", test.status));
                    if let Some(output) = &test.output {
                        parts.push(output.clone());
                    }
                }
            }
        }
        AmpObject::Run(run) => {
            parts.push(run.base.provenance.summary.clone());
            parts.push(run.input_summary.clone());
            parts.push(format!("{:?}", run.status));

            if let Some(outputs) = &run.outputs {
                for output in outputs {
                    parts.push(format!("{:?}", output.output_type));
                    parts.push(output.content.clone());
                }
            }

            if let Some(errors) = &run.errors {
                for error in errors {
                    parts.push(error.message.clone());
                    if let Some(code) = &error.code {
                        parts.push(code.clone());
                    }
                }
            }
        }
        AmpObject::FileChunk(chunk) => {
            parts.push(chunk.file_path.clone());
            parts.push(chunk.content.clone());
            parts.push(chunk.language.clone());
        }
        AmpObject::FileLog(log) => {
            parts.push(log.file_path.clone());
            parts.push(log.summary.clone());
            if let Some(purpose) = &log.purpose {
                parts.push(purpose.clone());
            }
            parts.extend(log.key_symbols.clone());
            parts.extend(log.dependencies.clone());
        }
    }

    parts
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

async fn apply_embedding(state: &AppState, obj: AmpObject) -> AmpObject {
    if !state.embedding_service.is_enabled() {
        return obj;
    }

    let text = extract_embedding_text(&obj);
    if text.trim().is_empty() {
        return obj;
    }

    match state.embedding_service.generate_embedding(&text).await {
        Ok(embedding) => set_embedding(obj, Some(embedding)),
        Err(err) => {
            tracing::warn!("Failed to generate embedding: {}", err);
            obj
        }
    }
}



pub async fn create_object(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let object_id = payload.get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    tracing::info!("Creating object: {}", object_id);

    // Parse the payload into proper SurrealDB format
    let mut clean_payload = payload.clone();

    // Generate embedding if enabled (for hybrid search)
    if state.embedding_service.is_enabled() {
        if let Some(text) = extract_text_for_embedding(&clean_payload) {
            if !text.trim().is_empty() {
                match state.embedding_service.generate_embedding(&text).await {
                    Ok(embedding) => {
                        if let Some(map) = clean_payload.as_object_mut() {
                            map.insert("embedding".to_string(), serde_json::json!(embedding));
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate embedding for {}: {}", object_id, e);
                    }
                }
            }
        }
    }

    // Ensure proper field types for SurrealDB
    if let Some(obj) = clean_payload.as_object_mut() {
        // Remove id from content - CREATE objects:`id` CONTENT {...} sets the ID via the record path,
        // including id in content causes: "Found 'id' for the `id` field, but a specific record has been specified"
        obj.remove("id");

        // Remove timestamps - let DB set them
        obj.remove("created_at");
        obj.remove("updated_at");
    }

    // Create with explicit ID using backtick syntax - but use proper JSON structure
    let query = format!("CREATE objects:`{}` CONTENT $data", object_id);
    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client
            .query(query)
            .bind(("data", clean_payload)),
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            Ok((
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "id": object_id,
                    "created_at": chrono::Utc::now().to_rfc3339()
                })),
            ))
        }
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
    Json(payload): Json<Vec<Value>>,
) -> Result<(StatusCode, Json<BatchResponse>), StatusCode> {
    let mut results = Vec::new();
    let total = payload.len();
    let mut succeeded = 0;
    let mut failed = 0;

    for mut obj_value in payload {
        let object_id = obj_value.get("id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);

        // Generate embedding if enabled
        if state.embedding_service.is_enabled() {
            if let Some(text) = extract_text_for_embedding(&obj_value) {
                if !text.trim().is_empty() {
                    match state.embedding_service.generate_embedding(&text).await {
                        Ok(embedding) => {
                            if let Some(map) = obj_value.as_object_mut() {
                                map.insert("embedding".to_string(), serde_json::json!(embedding));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to generate embedding for {}: {}", object_id, e);
                        }
                    }
                }
            }
        }

        // Remove timestamps - let DB set them
        if let Some(map) = obj_value.as_object_mut() {
            map.remove("created_at");
            map.remove("updated_at");
        }

        let query = "INSERT INTO objects $data";
        let result: Result<Result<surrealdb::Response, _>, _> = timeout(
            Duration::from_secs(5),
            state.db.client
                .query(query)
                .bind(("data", obj_value)),
        )
        .await;

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

fn extract_text_for_embedding(obj: &Value) -> Option<String> {
    let obj_type = obj.get("type")?.as_str()?.to_lowercase();
    let mut parts = Vec::new();

    match obj_type.as_str() {
        "symbol" => {
            if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                parts.push(name.to_string());
            }
            if let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) {
                parts.push(kind.to_string());
            }
            if let Some(path) = obj.get("path").and_then(|v| v.as_str()) {
                parts.push(path.to_string());
            }
            if let Some(sig) = obj.get("signature").and_then(|v| v.as_str()) {
                parts.push(sig.to_string());
            }
            if let Some(doc) = obj.get("documentation").and_then(|v| v.as_str()) {
                parts.push(doc.to_string());
            }
        }
        "filechunk" => {
            if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
                parts.push(content.to_string());
            }
        }
        "filelog" => {
            if let Some(summary) = obj.get("summary").and_then(|v| v.as_str()) {
                parts.push(summary.to_string());
            }
            if let Some(purpose) = obj.get("purpose").and_then(|v| v.as_str()) {
                parts.push(purpose.to_string());
            }
        }
        _ => {}
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

pub async fn get_object(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    tracing::debug!("Get object: {}", id);
    
    let query = "SELECT * FROM objects WHERE id = $id";
    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client
            .query(query)
            .bind(("id", id)),
    )
    .await;

    match result {
        Ok(Ok(mut response)) => {
            let mut results = take_json_values(&mut response, 0);
            if results.is_empty() {
                tracing::warn!("Object not found: {}", id);
                return Err(StatusCode::NOT_FOUND);
            }
            let mut json_value = results.remove(0);
            normalize_object_id(&mut json_value);
            Ok(Json(json_value))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to retrieve object {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout retrieving object {}", id);
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn update_object(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Updating object: {}", id);

    // Support partial updates - remove RETURN to avoid serialization issues
    let query = format!("UPDATE objects:`{}` MERGE $data", id);

    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client
            .query(query)
            .bind(("data", payload)),
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            tracing::info!("Object updated: {}", id);
            Ok(Json(serde_json::json!({"success": true, "message": "Object updated"})))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to update object {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout updating object {}", id);
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn delete_object(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let query = "DELETE type::record('objects', $id)";
    
    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.query(query).bind(("id", id)),
    )
    .await;

    match result {
        Ok(Ok(_)) => Ok(StatusCode::NO_CONTENT),
        Ok(Err(e)) => {
            tracing::error!("Failed to delete object {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout deleting object {}", id);
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}
