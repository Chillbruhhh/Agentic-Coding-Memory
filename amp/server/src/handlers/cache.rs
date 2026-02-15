use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::cache::{CacheItem, CacheItemKind, CacheService};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GetPackRequest {
    pub scope_id: String,
    #[serde(default = "default_token_budget")]
    pub token_budget: usize,
    pub query: Option<String>,
    #[allow(dead_code)] // Reserved for delta pack feature
    pub since_version: Option<u64>,
}

fn default_token_budget() -> usize {
    600
}

#[derive(Debug, Serialize)]
pub struct GetPackResponse {
    pub scope_id: String,
    pub summary: String,
    pub facts: Vec<PackItem>,
    pub decisions: Vec<PackItem>,
    pub snippets: Vec<PackItem>,
    pub warnings: Vec<PackItem>,
    pub artifact_pointers: Vec<String>,
    pub token_count: usize,
    pub version: u64,
    pub is_fresh: bool,
}

#[derive(Debug, Serialize)]
pub struct PackItem {
    pub preview: String,
    pub facts: Vec<String>,
    pub importance: f32,
    pub artifact_id: Option<String>,
}

impl From<CacheItem> for PackItem {
    fn from(item: CacheItem) -> Self {
        Self {
            preview: item.preview,
            facts: item.facts,
            importance: item.importance,
            artifact_id: item.artifact_id,
        }
    }
}

pub async fn get_pack(
    State(state): State<AppState>,
    Json(request): Json<GetPackRequest>,
) -> Result<Json<GetPackResponse>, (StatusCode, String)> {
    // Get query embedding if query provided
    let query_embedding = if let Some(ref query) = request.query {
        if state.embedding_service.is_enabled() {
            state.embedding_service.generate_embedding(query).await.ok()
        } else {
            None
        }
    } else {
        None
    };

    let cache_service = CacheService::new(state.db.clone(), state.embedding_service.clone());

    let pack = cache_service
        .get_pack(
            &request.scope_id,
            request.token_budget,
            query_embedding.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to get cache pack: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(Json(GetPackResponse {
        scope_id: pack.scope_id,
        summary: pack.summary,
        facts: pack.facts.into_iter().map(PackItem::from).collect(),
        decisions: pack.decisions.into_iter().map(PackItem::from).collect(),
        snippets: pack.snippets.into_iter().map(PackItem::from).collect(),
        warnings: pack.warnings.into_iter().map(PackItem::from).collect(),
        artifact_pointers: pack.artifact_pointers,
        token_count: pack.token_count,
        version: pack.version,
        is_fresh: pack.is_fresh,
    }))
}

#[derive(Debug, Deserialize)]
pub struct WriteItemsRequest {
    pub scope_id: String,
    pub items: Vec<WriteItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct WriteItemInput {
    pub kind: String,
    pub preview: String,
    #[serde(default)]
    pub facts: Vec<String>,
    pub artifact_id: Option<String>,
    #[serde(default = "default_importance")]
    pub importance: f32,
}

fn default_importance() -> f32 {
    0.5
}

#[derive(Debug, Serialize)]
pub struct WriteItemsResponse {
    pub written: usize,
    pub merged: usize,
}

pub async fn write_items(
    State(state): State<AppState>,
    Json(request): Json<WriteItemsRequest>,
) -> Result<Json<WriteItemsResponse>, (StatusCode, String)> {
    let cache_service = CacheService::new(state.db.clone(), state.embedding_service.clone());

    let items: Vec<CacheItem> = request
        .items
        .into_iter()
        .map(|input| {
            let kind = match input.kind.to_lowercase().as_str() {
                "fact" => CacheItemKind::Fact,
                "decision" => CacheItemKind::Decision,
                "snippet" => CacheItemKind::Snippet,
                "warning" => CacheItemKind::Warning,
                _ => CacheItemKind::Fact, // Default to fact
            };

            CacheItem {
                id: None,
                scope_id: request.scope_id.clone(),
                artifact_id: input.artifact_id,
                kind,
                preview: input.preview,
                facts: input.facts,
                embedding: None,
                importance: input.importance,
                access_count: 0,
                provenance: Value::Object(Default::default()),
            }
        })
        .collect();

    let total = items.len();
    let written = cache_service
        .write_items(&request.scope_id, items)
        .await
        .map_err(|e| {
            tracing::error!("Failed to write cache items: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    // Items that weren't written were merged with existing
    let merged = total - written;

    Ok(Json(WriteItemsResponse { written, merged }))
}

pub async fn gc(State(state): State<AppState>) -> Result<Json<Value>, (StatusCode, String)> {
    let cache_service = CacheService::new(state.db.clone(), state.embedding_service.clone());

    cache_service.gc().await.map_err(|e| {
        tracing::error!("Failed to run cache GC: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Cache garbage collection completed"
    })))
}

// ============================================================================
// Block-Based Episodic Memory Cache
// Rolling window of ~20 blocks, each holding 1800-2000 tokens
// ============================================================================

use crate::surreal_json::take_json_values;

const MAX_BLOCKS: usize = 20;
const TOKEN_THRESHOLD: usize = 1800;

/// Escape a cache_block record ID for use in queries
/// SurrealDB requires backticks around IDs containing hyphens
/// Note: SurrealDB returns IDs with ⟨⟩ (Unicode angle brackets) but queries need backticks
fn escape_block_id(id: &str) -> String {
    // If it already has backticks, return as-is
    if id.contains('`') {
        return id.to_string();
    }
    // If it's just a UUID (no prefix), add the table prefix and backticks
    if !id.starts_with("cache_block:") {
        // Clean any angle brackets that might be present
        let clean = id.trim_matches('⟨').trim_matches('⟩');
        return format!("cache_block:`{}`", clean);
    }
    // Extract the UUID part and add backticks
    let uuid_part = id.strip_prefix("cache_block:").unwrap_or(id);
    // Remove Unicode angle brackets that SurrealDB adds to output
    let clean_uuid = uuid_part.trim_matches('⟨').trim_matches('⟩');
    format!("cache_block:`{}`", clean_uuid)
}

#[derive(Debug, Deserialize)]
pub struct BlockWriteRequest {
    pub scope_id: String,
    pub kind: String,
    pub content: String,
    #[serde(default = "default_importance")]
    pub importance: f32,
    pub file_ref: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockWriteResponse {
    pub block_id: String,
    pub block_status: String,
    pub token_count: usize,
    pub items_in_block: usize,
    pub new_block_id: Option<String>,
    pub evicted_block: Option<String>,
}

fn normalize_run_id(raw: &str) -> String {
    raw.trim()
        .trim_matches('⟨')
        .trim_matches('⟩')
        .trim_matches('`')
        .trim_start_matches("objects:")
        .to_string()
}

async fn fetch_active_run_ids_for_project(state: &AppState, project_id: &str) -> Vec<String> {
    let query = r#"
        SELECT VALUE run_id FROM agent_connections
        WHERE status = 'connected'
          AND expires_at > time::now()
          AND (project_id = $project_id OR project_id IS NONE)
          AND run_id IS NOT NONE
    "#;
    let mut response = match state.db.client
        .query(query)
        .bind(("project_id", project_id.to_string()))
        .await
    {
        Ok(response) => response,
        Err(_) => return Vec::new(),
    };
    take_json_values(&mut response, 0)
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

async fn write_block_for_scope(
    state: &AppState,
    scope_id: &str,
    request: &BlockWriteRequest,
) -> Result<BlockWriteResponse, (StatusCode, String)> {
    // Estimate tokens for this item
    let item_tokens = request.content.len() / 4;

    // Find or create open block for this scope
    let find_query = "SELECT <string>id AS id_str, scope_id, sequence, status, items, token_count FROM cache_block WHERE scope_id = $scope_id AND status = 'open' LIMIT 1";

    tracing::debug!("Looking for open cache_block with scope_id = '{}'", scope_id);

    let mut response = state.db.client
        .query(find_query)
        .bind(("scope_id", scope_id.to_string()))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let values = take_json_values(&mut response, 0);
    tracing::debug!("Found {} cache_block records", values.len());

    let (block_id, mut token_count, mut items, sequence) = if let Some(block) = values.first() {
        tracing::debug!("Found existing block: {:?}", block);
        let id = block.get("id_str").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tokens = block.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let items_arr = block.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let seq = block.get("sequence").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        tracing::debug!("Using existing block: id={}, tokens={}, items={}, seq={}", id, tokens, items_arr.len(), seq);
        (id, tokens, items_arr, seq)
    } else {
        // Create new open block - use backticks to escape UUID with hyphens
        tracing::debug!("No existing open block found, creating new one");
        let uuid = uuid::Uuid::new_v4();
        let new_id = format!("cache_block:`{}`", uuid);
        let create_query = format!(
            "CREATE {} SET scope_id = $scope_id, sequence = 1, status = 'open', items = [], token_count = 0, created_at = time::now()",
            new_id
        );
        tracing::debug!("Creating block with query: {}", create_query);
        state.db.client
            .query(&create_query)
            .bind(("scope_id", scope_id.to_string()))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        tracing::debug!("Created new block: {}", new_id);
        (new_id, 0, Vec::new(), 1)
    };

    // Check if adding this item would exceed threshold
    let mut new_block_id = None;
    let mut evicted_block = None;
    let mut final_block_id = block_id.clone();
    let mut final_status = "open".to_string();

    if token_count + item_tokens >= TOKEN_THRESHOLD {
        // Close current block
        let close_result = close_block(state, &block_id, scope_id).await;
        if let Err(e) = close_result {
            tracing::warn!("Failed to close block: {}", e);
        }

        // Check if we need to evict oldest block
        evicted_block = evict_oldest_if_needed(state, scope_id).await.ok().flatten();

        // Create new block - use backticks to escape UUID with hyphens
        let new_seq = sequence + 1;
        let uuid = uuid::Uuid::new_v4();
        let created_id = format!("cache_block:`{}`", uuid);
        let create_query = format!(
            "CREATE {} SET scope_id = $scope_id, sequence = $seq, status = 'open', items = [], token_count = 0, created_at = time::now()",
            created_id
        );
        state.db.client
            .query(&create_query)
            .bind(("scope_id", scope_id.to_string()))
            .bind(("seq", new_seq as i32))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        new_block_id = Some(created_id.clone());
        final_block_id = created_id;
        final_status = "closed".to_string(); // Previous block was closed
        token_count = 0;
        items = Vec::new();
    }

    // Add item to the block
    let new_item = serde_json::json!({
        "kind": request.kind,
        "content": request.content,
        "importance": request.importance,
        "file_ref": request.file_ref,
        "created_at": chrono::Utc::now().to_rfc3339()
    });
    items.push(new_item);
    token_count += item_tokens;

    // Update the block - escape ID for SurrealDB
    let escaped_id = escape_block_id(&final_block_id);
    let update_query = format!(
        "UPDATE {} SET items = $items, token_count = $tokens",
        escaped_id
    );
    state.db.client
        .query(&update_query)
        .bind(("items", items.clone()))
        .bind(("tokens", token_count as i32))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(BlockWriteResponse {
        block_id: final_block_id,
        block_status: final_status,
        token_count,
        items_in_block: items.len(),
        new_block_id,
        evicted_block,
    })
}

/// Write an item to the current open cache block
pub async fn block_write(
    State(state): State<AppState>,
    Json(request): Json<BlockWriteRequest>,
) -> Result<Json<BlockWriteResponse>, (StatusCode, String)> {
    let primary = write_block_for_scope(&state, &request.scope_id, &request).await?;

    if let Some(project_id) = request.scope_id.strip_prefix("project:") {
        let run_ids = fetch_active_run_ids_for_project(&state, project_id).await;
        for run_id in run_ids {
            let normalized_run = normalize_run_id(&run_id);
            if normalized_run.is_empty() {
                continue;
            }
            let run_scope = format!("run:{}", normalized_run);
            let session_scope = format!("session:{}", normalized_run);
            let _ = write_block_for_scope(&state, &run_scope, &request).await;
            let _ = write_block_for_scope(&state, &session_scope, &request).await;
        }
    }

    Ok(Json(primary))
}

#[derive(Debug, Deserialize)]
pub struct BlockCompactRequest {
    pub scope_id: String,
}

#[derive(Debug, Serialize)]
pub struct BlockCompactResponse {
    pub closed_block_id: Option<String>,
    pub new_block_id: String,
    pub summary_generated: bool,
}

/// Close current block and open a new one
pub async fn block_compact(
    State(state): State<AppState>,
    Json(request): Json<BlockCompactRequest>,
) -> Result<Json<BlockCompactResponse>, (StatusCode, String)> {
    // Find open block
    let find_query = "SELECT <string>id AS id_str, sequence FROM cache_block WHERE scope_id = $scope_id AND status = 'open' LIMIT 1";

    let mut response = state.db.client
        .query(find_query)
        .bind(("scope_id", request.scope_id.clone()))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let values = take_json_values(&mut response, 0);

    let (closed_id, sequence) = if let Some(block) = values.first() {
        let id = block.get("id_str").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let seq = block.get("sequence").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        // Close the block
        let _summary_generated = close_block(&state, &id, &request.scope_id).await.is_ok();

        // Evict if needed
        let _ = evict_oldest_if_needed(&state, &request.scope_id).await;

        (Some(id), seq)
    } else {
        (None, 0)
    };

    // Create new block - use backticks to escape UUID with hyphens
    let new_seq = sequence + 1;
    let uuid = uuid::Uuid::new_v4();
    let new_id = format!("cache_block:`{}`", uuid);
    let create_query = format!(
        "CREATE {} SET scope_id = $scope_id, sequence = $seq, status = 'open', items = [], token_count = 0, created_at = time::now()",
        new_id
    );
    state.db.client
        .query(&create_query)
        .bind(("scope_id", request.scope_id.clone()))
        .bind(("seq", new_seq as i32))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(BlockCompactResponse {
        closed_block_id: closed_id,
        new_block_id: new_id,
        summary_generated: true,
    }))
}

#[derive(Debug, Deserialize)]
pub struct BlockSearchRequest {
    pub scope_id: String,
    pub query: String,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
    /// Include the current open block in search results (default: false)
    #[serde(default)]
    pub include_open: bool,
}

fn default_search_limit() -> usize {
    5
}

#[derive(Debug, Serialize)]
pub struct BlockSearchResponse {
    pub matches: Vec<BlockMatch>,
}

#[derive(Debug, Serialize)]
pub struct BlockMatch {
    pub block_id: String,
    pub summary: String,
    pub relevance: f64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct BlockReadRequest {
    pub scope_id: String,
    #[serde(default)]
    pub list_all: Option<bool>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub include_content: Option<bool>,
    #[serde(default)]
    pub include_open: Option<bool>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub block_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlockReadQuery {
    pub scope_id: String,
    #[serde(default)]
    pub list_all: Option<bool>,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub include_content: Option<bool>,
    #[serde(default)]
    pub include_open: Option<bool>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub block_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockReadResponse {
    pub scope_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<BlockGetResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<BlockMatch>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocks: Option<Vec<BlockGetResponse>>,
}

pub async fn block_read_get(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<BlockReadQuery>,
) -> Result<Json<BlockReadResponse>, (StatusCode, String)> {
    let request = BlockReadRequest {
        scope_id: query.scope_id,
        list_all: query.list_all,
        query: query.query,
        include_content: query.include_content,
        include_open: query.include_open,
        limit: query.limit,
        block_id: query.block_id,
    };
    block_read_impl(&state, request).await
}

pub async fn block_read_post(
    State(state): State<AppState>,
    Json(request): Json<BlockReadRequest>,
) -> Result<Json<BlockReadResponse>, (StatusCode, String)> {
    block_read_impl(&state, request).await
}

pub async fn block_list_get(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<BlockReadQuery>,
) -> Result<Json<BlockReadResponse>, (StatusCode, String)> {
    let request = BlockReadRequest {
        scope_id: query.scope_id,
        list_all: Some(true),
        query: None,
        include_content: query.include_content,
        include_open: query.include_open,
        limit: query.limit,
        block_id: None,
    };
    block_read_impl(&state, request).await
}

pub async fn block_list_post(
    State(state): State<AppState>,
    Json(mut request): Json<BlockReadRequest>,
) -> Result<Json<BlockReadResponse>, (StatusCode, String)> {
    request.list_all = Some(true);
    request.query = None;
    request.block_id = None;
    block_read_impl(&state, request).await
}

async fn block_read_impl(
    state: &AppState,
    request: BlockReadRequest,
) -> Result<Json<BlockReadResponse>, (StatusCode, String)> {
    // Case 1: Get a specific block by ID
    if let Some(block_id) = request.block_id.as_deref() {
        let block = get_block_by_id(state, block_id).await?;
        return Ok(Json(BlockReadResponse {
            scope_id: request.scope_id,
            block: Some(block),
            matches: None,
            blocks: None,
        }));
    }

    // Case 2: list_all=true (newest first, include open by default)
    if request.list_all.unwrap_or(false) {
        let limit = request.limit.unwrap_or(5);
        let include_open = request.include_open.unwrap_or(true);
        let include_content = request.include_content.unwrap_or(false);

        let search_request = BlockSearchRequest {
            scope_id: request.scope_id.clone(),
            query: "*".to_string(),
            limit,
            include_open,
        };

        let Json(search_result) = block_search(State(state.clone()), Json(search_request)).await?;

        if include_content {
            let mut blocks = Vec::new();
            for m in &search_result.matches {
                blocks.push(get_block_by_id(state, &m.block_id).await?);
            }
            return Ok(Json(BlockReadResponse {
                scope_id: request.scope_id,
                block: None,
                matches: Some(search_result.matches),
                blocks: Some(blocks),
            }));
        }

        return Ok(Json(BlockReadResponse {
            scope_id: request.scope_id,
            block: None,
            matches: Some(search_result.matches),
            blocks: None,
        }));
    }

    // Case 3: search mode (query provided)
    if let Some(query) = request.query.clone() {
        let limit = request.limit.unwrap_or(5);
        let include_open = request.include_open.unwrap_or(false);
        let include_content = request.include_content.unwrap_or(false);

        let search_request = BlockSearchRequest {
            scope_id: request.scope_id.clone(),
            query,
            limit,
            include_open,
        };

        let Json(search_result) = block_search(State(state.clone()), Json(search_request)).await?;

        if include_content {
            let mut blocks = Vec::new();
            for m in &search_result.matches {
                blocks.push(get_block_by_id(state, &m.block_id).await?);
            }
            return Ok(Json(BlockReadResponse {
                scope_id: request.scope_id,
                block: None,
                matches: Some(search_result.matches),
                blocks: Some(blocks),
            }));
        }

        return Ok(Json(BlockReadResponse {
            scope_id: request.scope_id,
            block: None,
            matches: Some(search_result.matches),
            blocks: None,
        }));
    }

    // Case 4: default to current open block for scope
    let block = get_or_create_open_block(state, &request.scope_id).await?;
    Ok(Json(BlockReadResponse {
        scope_id: request.scope_id,
        block: Some(block),
        matches: None,
        blocks: None,
    }))
}

/// Search cache blocks by summary
pub async fn block_search(
    State(state): State<AppState>,
    Json(request): Json<BlockSearchRequest>,
) -> Result<Json<BlockSearchResponse>, (StatusCode, String)> {
    let mut matches: Vec<BlockMatch> = Vec::new();

    // If include_open is true, first add the current open block (if it exists and matches)
    if request.include_open {
        let open_query = "SELECT <string>id AS block_id, items, <string>created_at AS created_at FROM cache_block WHERE scope_id = $scope_id AND status = 'open' LIMIT 1";

        let mut open_response = state.db.client
            .query(open_query)
            .bind(("scope_id", request.scope_id.clone()))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let open_values = take_json_values(&mut open_response, 0);

        if let Some(open_block) = open_values.first() {
            // Generate summary from items for the open block
            let items = open_block.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let mut summary_parts: Vec<String> = Vec::new();
            let query_lower = request.query.to_lowercase();
            let mut content_matches = false;

            for item in &items {
                if let Some(content) = item.get("content").and_then(|c| c.as_str()) {
                    let kind = item.get("kind").and_then(|k| k.as_str()).unwrap_or("item");
                    let part = format!("[{}] {}", kind, content);
                    // Check if query matches content (case-insensitive)
                    if content.to_lowercase().contains(&query_lower) || query_lower == "*" {
                        content_matches = true;
                    }
                    if summary_parts.len() < 5 {
                        summary_parts.push(part);
                    }
                }
            }

            // Include open block if query matches content or is wildcard
            if content_matches || request.query == "*" {
                let summary = if summary_parts.is_empty() {
                    "[open block - no items yet]".to_string()
                } else {
                    summary_parts.join("; ")
                };

                matches.push(BlockMatch {
                    block_id: open_block.get("block_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    summary,
                    relevance: 1.0, // Open block gets highest relevance since it's current
                    created_at: open_block.get("created_at").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                });
            }
        }
    }

    // Generate embedding for query
    let query_embedding = if state.embedding_service.is_enabled() && request.query != "*" {
        state.embedding_service.generate_embedding(&request.query).await.ok()
    } else {
        None
    };

    let closed_matches: Vec<BlockMatch> = if let Some(embedding) = query_embedding {
        // Semantic search on summaries
        let vec_str = embedding.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
        let search_query = format!(
            "SELECT <string>id AS block_id, summary, vector::similarity::cosine(summary_embedding, [{}]) AS relevance, <string>created_at AS created_at FROM cache_block WHERE scope_id = $scope_id AND status = 'closed' AND summary_embedding IS NOT NONE ORDER BY relevance DESC LIMIT $limit",
            vec_str
        );

        let mut response = state.db.client
            .query(&search_query)
            .bind(("scope_id", request.scope_id.clone()))
            .bind(("limit", request.limit as i32))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let values = take_json_values(&mut response, 0);
        values.into_iter().filter_map(|v| {
            Some(BlockMatch {
                block_id: v.get("block_id")?.as_str()?.to_string(),
                summary: v.get("summary").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                relevance: v.get("relevance").and_then(|r| r.as_f64()).unwrap_or(0.0),
                created_at: v.get("created_at").and_then(|c| c.as_str()).unwrap_or("").to_string(),
            })
        }).collect()
    } else {
        // Fallback: text search (or wildcard)
        let search_query = if request.query == "*" {
            "SELECT <string>id AS block_id, summary, 0.5 AS relevance, <string>created_at AS created_at FROM cache_block WHERE scope_id = $scope_id AND status = 'closed' ORDER BY created_at DESC LIMIT $limit"
        } else {
            "SELECT <string>id AS block_id, summary, 0.5 AS relevance, <string>created_at AS created_at FROM cache_block WHERE scope_id = $scope_id AND status = 'closed' AND summary CONTAINS $query ORDER BY created_at DESC LIMIT $limit"
        };

        let mut response = state.db.client
            .query(search_query)
            .bind(("scope_id", request.scope_id.clone()))
            .bind(("query", request.query.clone()))
            .bind(("limit", request.limit as i32))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let values = take_json_values(&mut response, 0);
        values.into_iter().filter_map(|v| {
            Some(BlockMatch {
                block_id: v.get("block_id")?.as_str()?.to_string(),
                summary: v.get("summary").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                relevance: v.get("relevance").and_then(|r| r.as_f64()).unwrap_or(0.5),
                created_at: v.get("created_at").and_then(|c| c.as_str()).unwrap_or("").to_string(),
            })
        }).collect()
    };

    // Combine open block (if found) with closed block matches
    matches.extend(closed_matches);

    Ok(Json(BlockSearchResponse { matches }))
}

#[derive(Debug, Serialize)]
pub struct BlockGetResponse {
    pub block_id: String,
    pub status: String,
    pub summary: Option<String>,
    pub items: Vec<Value>,
    pub token_count: usize,
    pub created_at: String,
}

/// Get the current open block for a scope
pub async fn block_current(
    State(state): State<AppState>,
    axum::extract::Path(scope_id): axum::extract::Path<String>,
) -> Result<Json<BlockGetResponse>, (StatusCode, String)> {
    get_or_create_open_block(&state, &scope_id).await.map(Json)
}

/// Get a specific cache block by ID
pub async fn block_get(
    State(state): State<AppState>,
    axum::extract::Path(block_id): axum::extract::Path<String>,
) -> Result<Json<BlockGetResponse>, (StatusCode, String)> {
    // Escape the block ID for SurrealDB
    let escaped_id = escape_block_id(&block_id);

    let query = format!("SELECT <string>id AS id_str, status, summary, items, token_count, <string>created_at AS created_at FROM {}", escaped_id);

    let mut response = state.db.client
        .query(&query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let values = take_json_values(&mut response, 0);

    if let Some(block) = values.first() {
        Ok(Json(BlockGetResponse {
            block_id: block.get("id_str").and_then(|v| v.as_str()).unwrap_or(&block_id).to_string(),
            status: block.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            summary: block.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string()),
            items: block.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default(),
            token_count: block.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            created_at: block.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "Block not found".to_string()))
    }
}

async fn get_block_by_id(state: &AppState, block_id: &str) -> Result<BlockGetResponse, (StatusCode, String)> {
    // Escape the block ID for SurrealDB
    let escaped_id = escape_block_id(block_id);

    let query = format!("SELECT <string>id AS id_str, status, summary, items, token_count, <string>created_at AS created_at FROM {}", escaped_id);

    let mut response = state.db.client
        .query(&query)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let values = take_json_values(&mut response, 0);

    if let Some(block) = values.first() {
        Ok(BlockGetResponse {
            block_id: block.get("id_str").and_then(|v| v.as_str()).unwrap_or(block_id).to_string(),
            status: block.get("status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            summary: block.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string()),
            items: block.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default(),
            token_count: block.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            created_at: block.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    } else {
        Err((StatusCode::NOT_FOUND, "Block not found".to_string()))
    }
}

async fn get_or_create_open_block(
    state: &AppState,
    scope_id: &str,
) -> Result<BlockGetResponse, (StatusCode, String)> {
    let query = "SELECT <string>id AS id_str, status, summary, items, token_count, <string>created_at AS created_at FROM cache_block WHERE scope_id = $scope_id AND status = 'open' LIMIT 1";

    let mut response = state.db.client
        .query(query)
        .bind(("scope_id", scope_id.to_string()))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let values = take_json_values(&mut response, 0);

    if let Some(block) = values.first() {
        Ok(BlockGetResponse {
            block_id: block.get("id_str").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            status: block.get("status").and_then(|v| v.as_str()).unwrap_or("open").to_string(),
            summary: block.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string()),
            items: block.get("items").and_then(|v| v.as_array()).cloned().unwrap_or_default(),
            token_count: block.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            created_at: block.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    } else {
        // No open block exists - create a new empty block and return it
        let seq_query = "SELECT sequence FROM cache_block WHERE scope_id = $scope_id ORDER BY sequence DESC LIMIT 1";
        let mut seq_response = state.db.client
            .query(seq_query)
            .bind(("scope_id", scope_id.to_string()))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let seq_values = take_json_values(&mut seq_response, 0);
        let last_seq = seq_values
            .first()
            .and_then(|v| v.get("sequence"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let new_seq = last_seq + 1;
        let uuid = uuid::Uuid::new_v4();
        let new_id = format!("cache_block:`{}`", uuid);
        let created_at = chrono::Utc::now().to_rfc3339();
        let create_query = format!(
            "CREATE {} SET scope_id = $scope_id, sequence = $seq, status = 'open', items = [], token_count = 0, created_at = time::now()",
            new_id
        );

        state.db.client
            .query(&create_query)
            .bind(("scope_id", scope_id.to_string()))
            .bind(("seq", new_seq as i32))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(BlockGetResponse {
            block_id: new_id,
            status: "open".to_string(),
            summary: None,
            items: Vec::new(),
            token_count: 0,
            created_at,
        })
    }
}

/// Close a block and generate summary
async fn close_block(state: &AppState, block_id: &str, _scope_id: &str) -> Result<(), String> {
    // Escape the block ID for SurrealDB
    let escaped_id = escape_block_id(block_id);

    // Get items from block for summary generation
    let get_query = format!("SELECT items FROM {} LIMIT 1", escaped_id);
    let mut response = state.db.client
        .query(&get_query)
        .await
        .map_err(|e| e.to_string())?;

    let values = take_json_values(&mut response, 0);
    let items = values.first()
        .and_then(|v| v.get("items"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // Generate summary from items (combine content, max ~200 tokens)
    let mut summary_parts: Vec<String> = Vec::new();
    let mut summary_tokens = 0;
    for item in &items {
        if let Some(content) = item.get("content").and_then(|c| c.as_str()) {
            let kind = item.get("kind").and_then(|k| k.as_str()).unwrap_or("item");
            let part = format!("[{}] {}", kind, content);
            let part_tokens = part.len() / 4;
            if summary_tokens + part_tokens > 200 {
                break;
            }
            summary_parts.push(part);
            summary_tokens += part_tokens;
        }
    }
    let summary = summary_parts.join("; ");

    // Generate embedding for summary
    let summary_embedding = if state.embedding_service.is_enabled() && !summary.is_empty() {
        state.embedding_service.generate_embedding(&summary).await.ok()
    } else {
        None
    };

    let embedding_str = summary_embedding
        .as_ref()
        .map(|e| format!("[{}]", e.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ")))
        .unwrap_or_else(|| "NONE".to_string());

    // Update block to closed with summary
    let update_query = format!(
        "UPDATE {} SET status = 'closed', summary = $summary, summary_embedding = {}, closed_at = time::now()",
        escaped_id, embedding_str
    );
    state.db.client
        .query(&update_query)
        .bind(("summary", summary))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Evict oldest block if we have more than MAX_BLOCKS
async fn evict_oldest_if_needed(state: &AppState, scope_id: &str) -> Result<Option<String>, String> {
    let scope_id_owned = scope_id.to_string();

    // Count blocks for this scope
    let count_query = "SELECT VALUE count() FROM cache_block WHERE scope_id = $scope_id";
    let mut response = state.db.client
        .query(count_query)
        .bind(("scope_id", scope_id_owned.clone()))
        .await
        .map_err(|e| e.to_string())?;

    let values = take_json_values(&mut response, 0);
    let count = values.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    if count > MAX_BLOCKS {
        // Find and delete oldest block
        let find_query = "SELECT <string>id AS id_str FROM cache_block WHERE scope_id = $scope_id ORDER BY sequence ASC LIMIT 1";
        let mut response = state.db.client
            .query(find_query)
            .bind(("scope_id", scope_id_owned))
            .await
            .map_err(|e| e.to_string())?;

        let values = take_json_values(&mut response, 0);
        if let Some(oldest) = values.first() {
            if let Some(oldest_id) = oldest.get("id_str").and_then(|v| v.as_str()) {
                let escaped_id = escape_block_id(oldest_id);
                let delete_query = format!("DELETE {}", escaped_id);
                state.db.client
                    .query(&delete_query)
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok(Some(oldest_id.to_string()));
            }
        }
    }

    Ok(None)
}
