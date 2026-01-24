use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::AppState;

/// Artifact types supported by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactType {
    Decision,
    FileLog,
    Note,
    ChangeSet,
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactType::Decision => write!(f, "decision"),
            ArtifactType::FileLog => write!(f, "filelog"),
            ArtifactType::Note => write!(f, "note"),
            ArtifactType::ChangeSet => write!(f, "changeset"),
        }
    }
}

/// Request to write an artifact - unified interface for all artifact types
#[derive(Debug, Deserialize)]
pub struct WriteArtifactRequest {
    /// Type of artifact
    #[serde(rename = "type")]
    pub artifact_type: ArtifactType,

    /// Title of the artifact
    pub title: String,

    /// Project ID this artifact belongs to
    pub project_id: Option<String>,

    /// Agent ID that created this artifact
    pub agent_id: Option<String>,

    /// Run ID if created during a specific run
    pub run_id: Option<String>,

    /// Tags for categorization
    pub tags: Option<Vec<String>>,

    // === Decision-specific fields ===
    /// Context/background for the decision
    pub context: Option<String>,
    /// The actual decision made
    pub decision: Option<String>,
    /// Consequences/implications of the decision
    pub consequences: Option<String>,
    /// Alternatives that were considered
    pub alternatives: Option<Vec<String>>,
    /// Status of the decision
    pub status: Option<String>,

    // === FileLog-specific fields ===
    /// Path to the file
    pub file_path: Option<String>,
    /// Summary of the file
    pub summary: Option<String>,
    /// Key symbols in the file
    pub symbols: Option<Vec<String>>,
    /// File dependencies
    pub dependencies: Option<Vec<String>>,

    // === Note-specific fields ===
    /// Content of the note (markdown)
    pub content: Option<String>,
    /// Category of the note
    pub category: Option<String>,

    // === ChangeSet-specific fields ===
    /// Description of the changes
    pub description: Option<String>,
    /// Diff summary
    pub diff_summary: Option<String>,
    /// Files that were changed
    pub files_changed: Option<Vec<String>>,

    // === Relationship fields (for graph layer) ===
    /// IDs of objects this artifact relates to
    pub linked_objects: Option<Vec<String>>,
    /// IDs of decisions that justify this artifact
    pub linked_decisions: Option<Vec<String>>,
    /// IDs of files this artifact modifies or references
    pub linked_files: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct WriteArtifactResponse {
    pub id: String,
    pub artifact_type: String,
    pub created_at: String,
    /// Which memory layers this artifact was written to
    pub memory_layers: MemoryLayersWritten,
    /// Relationships created in graph layer
    pub relationships_created: usize,
}

#[derive(Debug, Serialize)]
pub struct MemoryLayersWritten {
    pub graph: bool,
    pub vector: bool,
    pub temporal: bool,
}

/// Extract text for embedding generation based on artifact type
fn extract_embedding_text(request: &WriteArtifactRequest) -> String {
    let mut parts = Vec::new();

    // Always include title
    parts.push(request.title.clone());

    // Add tags
    if let Some(tags) = &request.tags {
        parts.extend(tags.clone());
    }

    // Type-specific text
    match request.artifact_type {
        ArtifactType::Decision => {
            if let Some(context) = &request.context {
                parts.push(context.clone());
            }
            if let Some(decision) = &request.decision {
                parts.push(decision.clone());
            }
            if let Some(consequences) = &request.consequences {
                parts.push(consequences.clone());
            }
            if let Some(alternatives) = &request.alternatives {
                parts.extend(alternatives.clone());
            }
        }
        ArtifactType::FileLog => {
            if let Some(file_path) = &request.file_path {
                parts.push(file_path.clone());
            }
            if let Some(summary) = &request.summary {
                parts.push(summary.clone());
            }
            if let Some(symbols) = &request.symbols {
                parts.extend(symbols.clone());
            }
        }
        ArtifactType::Note => {
            if let Some(content) = &request.content {
                parts.push(content.clone());
            }
            if let Some(category) = &request.category {
                parts.push(category.clone());
            }
        }
        ArtifactType::ChangeSet => {
            if let Some(description) = &request.description {
                parts.push(description.clone());
            }
            if let Some(diff_summary) = &request.diff_summary {
                parts.push(diff_summary.clone());
            }
            if let Some(files_changed) = &request.files_changed {
                parts.extend(files_changed.clone());
            }
        }
    }

    parts
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build the object payload for storage
fn build_artifact_object(request: &WriteArtifactRequest, _object_id: &str) -> Value {
    let now = chrono::Utc::now().to_rfc3339();

    let mut obj = serde_json::json!({
        "type": request.artifact_type.to_string(),
        "title": request.title,
        "created_at": now,
        "updated_at": now,
        "memory_layers": {
            "graph": true,
            "vector": true,
            "temporal": true
        }
    });

    let map = obj.as_object_mut().unwrap();

    // Add optional base fields
    if let Some(project_id) = &request.project_id {
        map.insert("project_id".to_string(), Value::String(project_id.clone()));
    }
    if let Some(agent_id) = &request.agent_id {
        map.insert("agent_id".to_string(), Value::String(agent_id.clone()));
        // Also add to provenance for consistency
        map.insert(
            "provenance".to_string(),
            serde_json::json!({
                "agent": agent_id,
                "summary": format!("Created {} artifact", request.artifact_type)
            }),
        );
    }
    if let Some(run_id) = &request.run_id {
        map.insert("run_id".to_string(), Value::String(run_id.clone()));
    }
    if let Some(tags) = &request.tags {
        map.insert("tags".to_string(), serde_json::json!(tags));
    }

    // Add type-specific fields
    match request.artifact_type {
        ArtifactType::Decision => {
            if let Some(context) = &request.context {
                map.insert("context".to_string(), Value::String(context.clone()));
            }
            if let Some(decision) = &request.decision {
                map.insert("decision".to_string(), Value::String(decision.clone()));
            }
            if let Some(consequences) = &request.consequences {
                map.insert(
                    "consequences".to_string(),
                    Value::String(consequences.clone()),
                );
            }
            if let Some(alternatives) = &request.alternatives {
                map.insert("alternatives".to_string(), serde_json::json!(alternatives));
            }
            if let Some(status) = &request.status {
                map.insert("status".to_string(), Value::String(status.clone()));
            }
            if let Some(linked_files) = &request.linked_files {
                map.insert("linked_files".to_string(), serde_json::json!(linked_files));
            }
        }
        ArtifactType::FileLog => {
            if let Some(file_path) = &request.file_path {
                map.insert("file_path".to_string(), Value::String(file_path.clone()));
            }
            if let Some(summary) = &request.summary {
                map.insert("summary".to_string(), Value::String(summary.clone()));
            }
            if let Some(symbols) = &request.symbols {
                map.insert("symbols".to_string(), serde_json::json!(symbols));
                map.insert("key_symbols".to_string(), serde_json::json!(symbols));
                // For compatibility
            }
            if let Some(dependencies) = &request.dependencies {
                map.insert("dependencies".to_string(), serde_json::json!(dependencies));
            }
        }
        ArtifactType::Note => {
            if let Some(content) = &request.content {
                map.insert("content".to_string(), Value::String(content.clone()));
            }
            if let Some(category) = &request.category {
                map.insert("category".to_string(), Value::String(category.clone()));
            }
            if let Some(linked_objects) = &request.linked_objects {
                map.insert(
                    "linked_objects".to_string(),
                    serde_json::json!(linked_objects),
                );
            }
        }
        ArtifactType::ChangeSet => {
            if let Some(description) = &request.description {
                map.insert(
                    "description".to_string(),
                    Value::String(description.clone()),
                );
            }
            if let Some(diff_summary) = &request.diff_summary {
                map.insert(
                    "diff_summary".to_string(),
                    Value::String(diff_summary.clone()),
                );
            }
            if let Some(files_changed) = &request.files_changed {
                map.insert(
                    "files_changed".to_string(),
                    serde_json::json!(files_changed),
                );
            }
            if let Some(linked_decisions) = &request.linked_decisions {
                map.insert(
                    "linked_decisions".to_string(),
                    serde_json::json!(linked_decisions),
                );
            }
        }
    }

    obj
}

/// Write an artifact to all 3 memory layers
///
/// This is the unified endpoint for agents to create artifacts.
/// It writes to:
/// 1. Temporal layer (objects table with timestamps)
/// 2. Vector layer (generates embeddings for semantic search)
/// 3. Graph layer (creates relationships to linked objects)
pub async fn write_artifact(
    State(state): State<AppState>,
    Json(request): Json<WriteArtifactRequest>,
) -> Result<(StatusCode, Json<WriteArtifactResponse>), StatusCode> {
    let object_id = Uuid::new_v4().to_string();
    let artifact_type_str = request.artifact_type.to_string();

    tracing::info!(
        "Writing {} artifact: {} (id: {})",
        artifact_type_str,
        request.title,
        object_id
    );

    // Build the artifact object
    let mut artifact_obj = build_artifact_object(&request, &object_id);

    // === LAYER 2: Vector Layer - Generate embedding ===
    let mut vector_written = false;
    if state.embedding_service.is_enabled() {
        let text = extract_embedding_text(&request);
        if !text.trim().is_empty() {
            match state.embedding_service.generate_embedding(&text).await {
                Ok(embedding) => {
                    if let Some(map) = artifact_obj.as_object_mut() {
                        map.insert("embedding".to_string(), serde_json::json!(embedding));
                        vector_written = true;
                        tracing::debug!("Generated embedding for artifact {}", object_id);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to generate embedding for artifact {}: {}",
                        object_id,
                        e
                    );
                }
            }
        }
    }

    // Update memory_layers to reflect actual state
    if let Some(map) = artifact_obj.as_object_mut() {
        map.insert(
            "memory_layers".to_string(),
            serde_json::json!({
                "graph": true,
                "vector": vector_written,
                "temporal": true
            }),
        );
    }

    // === LAYER 1: Temporal Layer - Write to objects table ===
    let query = format!("CREATE objects:`{}` CONTENT $data", object_id);
    let result = timeout(
        Duration::from_secs(5),
        state.db.client.query(query).bind(("data", artifact_obj)),
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            tracing::info!("Created artifact in temporal layer: {}", object_id);
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to create artifact {}: {}", object_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Err(_) => {
            tracing::error!("Timeout creating artifact {}", object_id);
            return Err(StatusCode::GATEWAY_TIMEOUT);
        }
    }

    // === LAYER 3: Graph Layer - Create relationships ===
    let mut relationships_created = 0;

    // Helper to create a relationship
    async fn create_relationship(
        state: &AppState,
        source_id: &str,
        relation_type: &str,
        target_id: &str,
    ) -> bool {
        let query = format!(
            "RELATE objects:`{}`->{}->objects:`{}` SET created_at = time::now()",
            source_id, relation_type, target_id
        );

        match timeout(Duration::from_secs(2), state.db.client.query(query)).await {
            Ok(Ok(_)) => {
                tracing::debug!(
                    "Created relationship: {} -> {} -> {}",
                    source_id,
                    relation_type,
                    target_id
                );
                true
            }
            Ok(Err(e)) => {
                tracing::warn!("Failed to create relationship: {}", e);
                false
            }
            Err(_) => {
                tracing::warn!("Timeout creating relationship");
                false
            }
        }
    }

    fn normalize_surreal_id(value: &str) -> String {
        value
            .trim()
            .trim_start_matches("objects:")
            .trim_matches('`')
            .trim_matches('\u{27E8}')
            .trim_matches('\u{27E9}')
            .replace("âŸ¨", "")
            .replace("âŸ©", "")
    }

    async fn find_or_create_artifact_core(state: &AppState) -> Option<String> {
        let query = "SELECT VALUE { id: string::concat(id) } FROM objects WHERE type = 'artifact_core' LIMIT 1".to_string();
        let name = "Artifact Core".to_string();

        if let Ok(Ok(mut response)) =
            timeout(Duration::from_secs(2), state.db.client.query(query)).await
        {
            let results: Vec<Value> = crate::surreal_json::take_json_values(&mut response, 0);
            if let Some(core_obj) = results.first() {
                if let Some(core_id) = core_obj.get("id").and_then(|v| v.as_str()) {
                    return Some(normalize_surreal_id(core_id));
                }
            }
        }

        let core_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let core_obj = serde_json::json!({
            "type": "artifact_core",
            "kind": "artifact_core",
            "name": name,
            "created_at": now,
            "updated_at": now
        });
        let query = format!("CREATE objects:`{}` CONTENT $data", core_id);
        match timeout(
            Duration::from_secs(3),
            state.db.client.query(query).bind(("data", core_obj)),
        )
        .await
        {
            Ok(Ok(_)) => Some(core_id),
            _ => None,
        }
    }

    // Helper to find a file node by exact path (prefer file symbols, fallback to FileLog)
    async fn find_file_node_id(state: &AppState, file_path: &str) -> Option<String> {
        let trimmed = file_path.trim().to_string();
        if trimmed.is_empty() {
            return None;
        }

        // Normalize path for matching
        let normalized = trimmed.replace('/', "\\");
        let basename = trimmed
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(&trimmed)
            .to_string();

        tracing::info!("find_file_node_id: looking for '{}' (norm='{}', basename='{}')", trimmed, normalized, basename);

        // Try file nodes first so graph links attach to file nodes.
        // Use string::concat(id) to convert SurrealDB Thing to JSON-serializable string.
        let symbol_query = "SELECT VALUE { id: string::concat(id) } FROM objects WHERE ((type = 'file') OR (type = 'symbol' AND kind = 'file')) AND ((path = $path OR path CONTAINS $path OR path CONTAINS $norm OR path CONTAINS $basename) OR (file_path = $path OR file_path CONTAINS $path OR file_path CONTAINS $norm OR file_path CONTAINS $basename)) LIMIT 1";
        if let Ok(Ok(mut response)) = timeout(
            Duration::from_secs(2),
            state.db.client
                .query(symbol_query)
                .bind(("path", trimmed.clone()))
                .bind(("norm", normalized.clone()))
                .bind(("basename", basename.clone())),
        )
        .await
        {
            let results: Vec<Value> = crate::surreal_json::take_json_values(&mut response, 0);
            tracing::info!("find_file_node_id: file symbol query returned {} results: {:?}", results.len(), results);
            if let Some(file_obj) = results.first() {
                if let Some(file_id) = file_obj.get("id").and_then(|v| v.as_str()) {
                    tracing::info!("find_file_node_id: found file symbol id={}", file_id);
                    return Some(normalize_surreal_id(file_id));
                }
            }
        } else {
            tracing::warn!("find_file_node_id: file symbol query failed or timed out");
        }

        // Fallback: try FileLog with CONTAINS matching (handles relative paths)
        let filelog_query = "SELECT VALUE { id: string::concat(id) } FROM objects WHERE type = 'FileLog' AND (file_path = $path OR file_path CONTAINS $path OR file_path CONTAINS $norm OR file_path CONTAINS $basename) LIMIT 1";
        if let Ok(Ok(mut response)) = timeout(
            Duration::from_secs(2),
            state.db.client
                .query(filelog_query)
                .bind(("path", trimmed))
                .bind(("norm", normalized))
                .bind(("basename", basename)),
        )
        .await
        {
            let results: Vec<Value> = crate::surreal_json::take_json_values(&mut response, 0);
            if let Some(file_obj) = results.first() {
                if let Some(file_id) = file_obj.get("id").and_then(|v| v.as_str()) {
                    tracing::info!("find_file_node_id: found FileLog id={}", file_id);
                    return Some(normalize_surreal_id(file_id));
                }
            }
        }

        None
    }

    // Create relationships based on artifact type and linked objects

    // Link to project if specified and exists as an object
    if let Some(project_id) = &request.project_id {
        let project_query = format!(
            "SELECT id FROM objects WHERE (type = 'symbol' OR type = 'project') AND kind = 'project' AND project_id = '{}' LIMIT 1",
            project_id.replace("'", "\\'")
        );
        if let Ok(Ok(mut response)) =
            timeout(Duration::from_secs(2), state.db.client.query(project_query)).await
        {
            let results: Vec<Value> = crate::surreal_json::take_json_values(&mut response, 0);
            if let Some(project_obj) = results.first() {
                if let Some(proj_id) = project_obj.get("id").and_then(|v| v.as_str()) {
                    let clean_id = normalize_surreal_id(proj_id);
                    if create_relationship(&state, &object_id, "defined_in", &clean_id).await {
                        relationships_created += 1;
                    }
                }
            }
        }
    }

    // Link to run if specified (artifact was "produced" by run)
    if let Some(run_id) = &request.run_id {
        if create_relationship(&state, run_id, "produced", &object_id).await {
            relationships_created += 1;
        }
    }

    // Link to generic linked objects
    if let Some(linked_objects) = &request.linked_objects {
        for linked_id in linked_objects {
            // Artifact depends on these objects for context
            if create_relationship(&state, &object_id, "depends_on", linked_id).await {
                relationships_created += 1;
            }
        }
    }

    // Link decisions (for changesets that are justified by decisions)
    if let Some(linked_decisions) = &request.linked_decisions {
        for decision_id in linked_decisions {
            if create_relationship(&state, &object_id, "justified_by", decision_id).await {
                relationships_created += 1;
            }
        }
    }

    let mut linked_to_file = false;

    // Link files (for decisions/notes that modify or reference files)
    if let Some(linked_files) = &request.linked_files {
        for file_ref in linked_files {
            let mut target_id = None;
            if Uuid::parse_str(file_ref).is_ok() {
                target_id = Some(file_ref.clone());
            } else if let Some(file_id) = find_file_node_id(&state, file_ref).await {
                target_id = Some(file_id);
            }

            if let Some(file_id) = target_id {
                if create_relationship(&state, &object_id, "modifies", &file_id).await {
                    relationships_created += 1;
                    linked_to_file = true;
                }
            }
        }
    }

    // Conservative auto-link: if file_path is provided and no linked_files were given,
    // attempt an exact path match to a file object and link it.
    let has_linked_files = request
        .linked_files
        .as_ref()
        .map(|files| !files.is_empty())
        .unwrap_or(false);
    if !has_linked_files {
        if let Some(file_path) = &request.file_path {
            if let Some(file_id) = find_file_node_id(&state, file_path).await {
                if create_relationship(&state, &object_id, "modifies", &file_id).await {
                    relationships_created += 1;
                    linked_to_file = true;
                }
            }
        }
    }

    // For FileLog artifacts, link to the file itself
    if matches!(request.artifact_type, ArtifactType::FileLog) {
        if let Some(file_path) = &request.file_path {
            if let Some(file_id) = find_file_node_id(&state, file_path).await {
                if create_relationship(&state, &object_id, "defined_in", &file_id).await {
                    relationships_created += 1;
                    linked_to_file = true;
                }
            }
        }
    }

    // Link to a single global artifact core only when not tied to a file.
    if !linked_to_file {
        if let Some(core_id) = find_or_create_artifact_core(&state).await {
            if create_relationship(&state, &object_id, "defined_in", &core_id).await {
                relationships_created += 1;
            }
        }
    }

    let now = chrono::Utc::now().to_rfc3339();

    Ok((
        StatusCode::CREATED,
        Json(WriteArtifactResponse {
            id: object_id,
            artifact_type: artifact_type_str,
            created_at: now,
            memory_layers: MemoryLayersWritten {
                graph: relationships_created > 0,
                vector: vector_written,
                temporal: true,
            },
            relationships_created,
        }),
    ))
}

/// List artifacts with optional filtering
#[derive(Debug, Deserialize)]
pub struct ListArtifactsQuery {
    #[serde(rename = "type")]
    pub artifact_type: Option<String>,
    pub project_id: Option<String>,
    pub agent_id: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_artifacts(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListArtifactsQuery>,
) -> Result<Json<Vec<Value>>, StatusCode> {
    let limit = query.limit.unwrap_or(100);

    let mut conditions = vec!["type IN ['decision', 'filelog', 'note', 'changeset']".to_string()];

    if let Some(artifact_type) = &query.artifact_type {
        conditions.push(format!("type = '{}'", artifact_type.to_lowercase()));
    }
    if let Some(project_id) = &query.project_id {
        conditions.push(format!("project_id = '{}'", project_id));
    }
    if let Some(agent_id) = &query.agent_id {
        conditions.push(format!("agent_id = '{}'", agent_id));
    }

    let query_str = format!(
        "SELECT * FROM objects WHERE {} ORDER BY created_at DESC LIMIT {}",
        conditions.join(" AND "),
        limit
    );

    tracing::debug!("List artifacts query: {}", query_str);

    let result = timeout(Duration::from_secs(5), state.db.client.query(query_str)).await;

    match result {
        Ok(Ok(mut response)) => {
            let artifacts: Vec<Value> = crate::surreal_json::take_json_values(&mut response, 0);
            tracing::debug!("Found {} artifacts", artifacts.len());
            Ok(Json(artifacts))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to list artifacts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout listing artifacts");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn delete_artifact(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let raw_id = id.trim().trim_start_matches("objects:").to_string();

    let delete_rels_query = "DELETE FROM [depends_on, defined_in, calls, justified_by, modifies, implements, produced] WHERE in = type::thing('objects', $id) OR out = type::thing('objects', $id)";
    let rels_result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state
            .db
            .client
            .query(delete_rels_query)
            .bind(("id", raw_id.clone())),
    )
    .await;

    if let Ok(Err(e)) = rels_result {
        tracing::warn!(
            "Failed to delete relationships for artifact {}: {}",
            raw_id,
            e
        );
    }

    let delete_obj_query = "DELETE type::record('objects', $id)";
    let obj_result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.query(delete_obj_query).bind(("id", raw_id)),
    )
    .await;

    match obj_result {
        Ok(Ok(_)) => Ok(StatusCode::NO_CONTENT),
        Ok(Err(e)) => {
            tracing::error!("Failed to delete artifact {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout deleting artifact {}", id);
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}
