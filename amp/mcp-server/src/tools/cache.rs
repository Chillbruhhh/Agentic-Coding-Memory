use anyhow::Result;
use rmcp::model::Content;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

// ============================================================================
// Block-Based Episodic Memory Cache
// Rolling window of ~20 blocks, each holding 1800-2000 tokens
// ============================================================================

/// Item kind for cache entries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheItemKind {
    Fact,
    Decision,
    Snippet,
    Warning,
}

/// Input for writing to the cache (appends to current open block)
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpCacheWriteInput {
    /// Scope ID (e.g., "project:amp", "workspace:default")
    #[serde(default)]
    pub scope_id: Option<String>,
    /// Type of item: fact, decision, snippet, warning
    pub kind: CacheItemKind,
    /// Content of the cache item
    pub content: String,
    /// Importance score 0.0-1.0 (default: 0.5)
    #[serde(default)]
    pub importance: Option<f32>,
    /// Optional file reference (for snippets)
    #[serde(default)]
    pub file_ref: Option<String>,
}

/// Input for manually compacting/closing the current block
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpCacheCompactInput {
    /// Scope ID to compact
    #[serde(default)]
    pub scope_id: Option<String>,
}

/// Input for searching cache blocks by summary
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpCacheSearchInput {
    /// Scope ID to search within
    pub scope_id: String,
    /// Search query
    pub query: String,
    /// Maximum number of block summaries to return (default: 5)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Include the current open block in search results (default: false)
    #[serde(default)]
    pub include_open: Option<bool>,
}

/// Input for getting a specific block or current open block
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpCacheGetInput {
    /// Scope ID
    pub scope_id: String,
    /// Optional specific block ID to retrieve (omit for current open block)
    #[serde(default)]
    pub block_id: Option<String>,
    /// Token budget for response (legacy support)
    #[serde(default)]
    pub token_budget: Option<usize>,
    /// Optional query for semantic relevance (legacy support)
    #[serde(default)]
    pub query: Option<String>,
    /// Version for delta pack (legacy support)
    #[serde(default)]
    pub since_version: Option<u64>,
}

// ============================================================================
// Unified Cache Read Tool (replaces amp_cache_search + amp_cache_get)
// ============================================================================

/// Unified input for reading from cache - search, get specific block, list all, or get current block
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AmpCacheReadInput {
    /// Scope ID (e.g., "project:amp", "workspace:default")
    pub scope_id: String,

    // List all mode parameters
    /// List all blocks for the scope (returns newest first, defaults: limit=5, include_open=true)
    #[serde(default)]
    pub list_all: Option<bool>,

    // Search mode parameters
    /// Search query - if provided, searches closed blocks by summary
    #[serde(default)]
    pub query: Option<String>,
    /// Maximum blocks to return when searching or listing (default: 5)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Return full block content instead of just summaries (default: false)
    #[serde(default)]
    pub include_content: Option<bool>,
    /// Include the current open block in results (default: false for search, true for list_all)
    #[serde(default)]
    pub include_open: Option<bool>,

    // Get mode parameters
    /// Specific block ID to retrieve (returns full content)
    #[serde(default)]
    pub block_id: Option<String>,
}

// ============================================================================
// Legacy input types (kept for backward compatibility)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CacheItemInput {
    /// Type: fact, decision, snippet, warning
    pub kind: String,
    /// Short preview text
    pub preview: String,
    /// Atomic facts extracted
    #[serde(default)]
    pub facts: Vec<String>,
    /// Source artifact ID
    #[serde(default)]
    pub artifact_id: Option<String>,
    /// Importance score 0.0-1.0
    #[serde(default)]
    pub importance: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LegacyCacheWriteInput {
    /// Scope ID to write to
    pub scope_id: Option<String>,
    /// Items to write
    #[schemars(schema_with = "cache_item_list_schema")]
    pub items: Vec<CacheItemInput>,
}

fn cache_item_list_schema(gen: &mut SchemaGenerator) -> Schema {
    let item_schema = CacheItemInput::json_schema(gen);
    let mut map = Map::new();
    map.insert("type".to_string(), Value::String("array".to_string()));
    map.insert("items".to_string(), Value::from(item_schema));
    Schema::from(map)
}

// ============================================================================
// Handler implementations
// ============================================================================

/// Write an item to the current open cache block
/// Automatically closes block and opens new one if token threshold reached
pub async fn handle_cache_write(
    client: &crate::amp_client::AmpClient,
    run_id: Option<&str>,
    input: AmpCacheWriteInput,
) -> Result<Vec<Content>> {
    let kind_str = match input.kind {
        CacheItemKind::Fact => "fact",
        CacheItemKind::Decision => "decision",
        CacheItemKind::Snippet => "snippet",
        CacheItemKind::Warning => "warning",
    };

    let scope_id = input
        .scope_id
        .filter(|scope| !scope.trim().is_empty())
        .or_else(|| run_id.map(|id| format!("run:{}", id)))
        .unwrap_or_else(|| "project:amp".to_string());

    let payload = serde_json::json!({
        "scope_id": scope_id,
        "kind": kind_str,
        "content": input.content,
        "importance": input.importance.unwrap_or(0.5),
        "file_ref": input.file_ref,
    });

    let result = client.cache_block_write(payload).await?;

    let block_id = result.get("block_id").and_then(|v| v.as_str()).unwrap_or("unknown");
    let block_status = result.get("block_status").and_then(|v| v.as_str()).unwrap_or("open");
    let token_count = result.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let items_in_block = result.get("items_in_block").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut response = format!(
        "Cache write complete:\n  Block: {}\n  Status: {}\n  Tokens: {}/1800\n  Items: {}",
        block_id, block_status, token_count, items_in_block
    );

    // If block was closed, include that info
    if block_status == "closed" {
        if let Some(new_block) = result.get("new_block_id").and_then(|v| v.as_str()) {
            response.push_str(&format!("\n  New block opened: {}", new_block));
        }
        if let Some(evicted) = result.get("evicted_block").and_then(|v| v.as_str()) {
            response.push_str(&format!("\n  Evicted oldest block: {}", evicted));
        }
    }

    Ok(vec![Content::text(response)])
}

/// Manually close the current block (e.g., on conversation compact)
pub async fn handle_cache_compact(
    client: &crate::amp_client::AmpClient,
    run_id: Option<&str>,
    input: AmpCacheCompactInput,
) -> Result<Vec<Content>> {
    let scope_id = input
        .scope_id
        .filter(|scope| !scope.trim().is_empty())
        .or_else(|| run_id.map(|id| format!("run:{}", id)))
        .unwrap_or_else(|| "project:amp".to_string());

    let payload = serde_json::json!({
        "scope_id": scope_id,
    });

    let result = client.cache_block_compact(payload).await?;

    let closed_id = result.get("closed_block_id").and_then(|v| v.as_str()).unwrap_or("none");
    let new_id = result.get("new_block_id").and_then(|v| v.as_str()).unwrap_or("none");
    let summary_generated = result.get("summary_generated").and_then(|v| v.as_bool()).unwrap_or(false);

    let response = format!(
        "Cache compact complete:\n  Closed block: {}\n  New block: {}\n  Summary generated: {}",
        closed_id, new_id, summary_generated
    );

    Ok(vec![Content::text(response)])
}

/// Search cache blocks by summary (two-phase retrieval)
pub async fn handle_cache_search(
    client: &crate::amp_client::AmpClient,
    input: AmpCacheSearchInput,
) -> Result<Vec<Content>> {
    let limit = input.limit.unwrap_or(5);
    let include_open = input.include_open.unwrap_or(false);

    let payload = serde_json::json!({
        "scope_id": input.scope_id,
        "query": input.query,
        "limit": limit,
        "include_open": include_open,
    });

    let result = client.cache_block_search(payload).await?;

    let mut output = format!("Cache search results for: \"{}\"\n", input.query);
    output.push_str(&"-".repeat(50));
    output.push('\n');

    if let Some(matches) = result.get("matches").and_then(|v| v.as_array()) {
        if matches.is_empty() {
            output.push_str("No matching blocks found.\n");
        } else {
            for (idx, m) in matches.iter().enumerate() {
                let block_id = m.get("block_id").and_then(|v| v.as_str()).unwrap_or("?");
                let summary = m.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                let relevance = m.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let created = m.get("created_at").and_then(|v| v.as_str()).unwrap_or("");

                output.push_str(&format!(
                    "\n{}. Block: {} (relevance: {:.2})\n   Created: {}\n   Summary: {}\n",
                    idx + 1, block_id, relevance, created,
                    if summary.len() > 200 { &summary[..200] } else { summary }
                ));
            }
            output.push_str(&format!("\nUse amp_cache_get with block_id to retrieve full content.\n"));
        }
    }

    Ok(vec![Content::text(output)])
}

/// Get a specific block or the current open block
pub async fn handle_cache_get(
    client: &crate::amp_client::AmpClient,
    input: AmpCacheGetInput,
) -> Result<Vec<Content>> {
    // If block_id specified, get that specific block
    if let Some(block_id) = &input.block_id {
        let result = client.cache_block_get(block_id).await?;
        return Ok(vec![Content::text(format_block(&result)?)]);
    }

    // Legacy behavior: get memory pack for scope
    let token_budget = input.token_budget.unwrap_or(600);

    let mut request = serde_json::json!({
        "scope_id": input.scope_id,
        "token_budget": token_budget,
    });

    if let Some(query) = &input.query {
        request["query"] = serde_json::Value::String(query.clone());
    }

    if let Some(version) = input.since_version {
        request["since_version"] = serde_json::Value::Number(version.into());
    }

    let result = client.cache_get_pack(request).await?;
    let summary = format_memory_pack(&result, &input)?;
    Ok(vec![Content::text(summary)])
}

// ============================================================================
// Unified Cache Read Handler
// ============================================================================

/// Unified cache read - handles list all, search, get specific block, or get current block
///
/// Behavior:
/// - list_all=true → list newest blocks (default: 5, include_open=true)
/// - query + include_content=false → search (return summaries)
/// - query + include_content=true → search + return full content
/// - block_id → get specific block (full content)
/// - neither → get current open block
pub async fn handle_cache_read(
    client: &crate::amp_client::AmpClient,
    input: AmpCacheReadInput,
) -> Result<Vec<Content>> {
    // Case 1: Get specific block by ID
    if let Some(block_id) = &input.block_id {
        let result = client.cache_block_get(block_id).await?;
        return Ok(vec![Content::text(format_block(&result)?)]);
    }

    // Case 2: List all blocks mode (newest first, includes open block by default)
    if input.list_all.unwrap_or(false) {
        let limit = input.limit.unwrap_or(5);
        let include_content = input.include_content.unwrap_or(false);
        // Default to true for list_all - open block is current work, should be visible
        let include_open = input.include_open.unwrap_or(true);

        let payload = serde_json::json!({
            "scope_id": input.scope_id,
            "query": "*",  // Wildcard = no filtering, returns all
            "limit": limit,
            "include_open": include_open,
        });

        let result = client.cache_block_search(payload).await?;

        if include_content {
            return format_list_with_content(client, &result, &input.scope_id).await;
        } else {
            return Ok(vec![Content::text(format_list_summaries(&result, &input.scope_id)?)]);
        }
    }

    // Case 3: Search mode (query provided)
    if let Some(query) = &input.query {
        let limit = input.limit.unwrap_or(5);
        let include_content = input.include_content.unwrap_or(false);
        let include_open = input.include_open.unwrap_or(false);

        let payload = serde_json::json!({
            "scope_id": input.scope_id,
            "query": query,
            "limit": limit,
            "include_open": include_open,
        });

        let result = client.cache_block_search(payload).await?;

        if include_content {
            // Fetch full content for matching blocks
            return format_search_with_content(client, &result, query).await;
        } else {
            // Return summaries only
            return Ok(vec![Content::text(format_search_summaries(&result, query)?)]);
        }
    }

    // Case 4: Get current open block (no query, no block_id, no list_all)
    match client.cache_block_current(&input.scope_id).await? {
        Some(block) => Ok(vec![Content::text(format_block(&block)?)]),
        None => Ok(vec![Content::text(format!(
            "No open cache block found for scope: {}",
            input.scope_id
        ))]),
    }
}

// ============================================================================
// List All Blocks Formatting
// ============================================================================

/// Format list results as summaries (default for list_all)
/// Open blocks show item count, closed blocks show ~200 token summary
fn format_list_summaries(result: &serde_json::Value, scope_id: &str) -> Result<String> {
    let mut output = format!("Cache Blocks for scope: {}\n", scope_id);
    output.push_str(&"=".repeat(50));
    output.push('\n');

    if let Some(matches) = result.get("matches").and_then(|v| v.as_array()) {
        if matches.is_empty() {
            output.push_str("\nNo blocks found in this scope.\n");
            output.push_str("\nTip: Use amp_cache_write to create cache entries.\n");
        } else {
            output.push_str(&format!("Showing {} most recent blocks:\n\n", matches.len()));

            for (idx, m) in matches.iter().enumerate() {
                let block_id = m.get("block_id").and_then(|v| v.as_str()).unwrap_or("?");
                let summary = m.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                let relevance = m.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let created = m.get("created_at").and_then(|v| v.as_str()).unwrap_or("");

                // Determine status: open blocks have relevance=1.0 and dynamic summary
                let is_open = relevance == 1.0 && (summary.starts_with("[") || summary.contains("[open block"));
                let status = if is_open { "open" } else { "closed" };

                output.push_str(&format!("{}. [{}] {}\n", idx + 1, status, block_id));

                if !created.is_empty() {
                    output.push_str(&format!("   Created: {}\n", created));
                }

                if !summary.is_empty() {
                    // Truncate long summaries to ~200 chars for readability
                    let display_summary = if summary.len() > 200 {
                        format!("{}...", &summary[..200])
                    } else {
                        summary.to_string()
                    };
                    output.push_str(&format!("   Summary: {}\n", display_summary));
                }
                output.push('\n');
            }

            output.push_str("Tip: Use include_content=true for full block content, or block_id to get a specific block.\n");
        }
    } else {
        output.push_str("\nNo blocks found in this scope.\n");
    }

    Ok(output)
}

/// Format list results with full block content
async fn format_list_with_content(
    client: &crate::amp_client::AmpClient,
    result: &serde_json::Value,
    scope_id: &str,
) -> Result<Vec<Content>> {
    let mut output = format!("Cache Blocks for scope: {} (with content)\n", scope_id);
    output.push_str(&"=".repeat(50));
    output.push('\n');

    if let Some(matches) = result.get("matches").and_then(|v| v.as_array()) {
        if matches.is_empty() {
            output.push_str("\nNo blocks found in this scope.\n");
            output.push_str("\nTip: Use amp_cache_write to create cache entries.\n");
        } else {
            output.push_str(&format!("Showing {} most recent blocks:\n", matches.len()));

            for (idx, m) in matches.iter().enumerate() {
                let block_id = m.get("block_id").and_then(|v| v.as_str()).unwrap_or("?");
                let relevance = m.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);

                // Determine status from relevance (open blocks get 1.0)
                let is_open = relevance == 1.0;
                let status = if is_open { "open" } else { "closed" };

                output.push_str(&format!(
                    "\n[{}/{}] {} [{}]\n",
                    idx + 1, matches.len(), block_id, status
                ));
                output.push_str(&"-".repeat(40));
                output.push('\n');

                // Fetch full block content
                match client.cache_block_get(block_id).await {
                    Ok(block) => {
                        let token_count = block.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0);
                        output.push_str(&format!("Tokens: {}/1800\n", token_count));

                        if let Some(summary) = block.get("summary").and_then(|v| v.as_str()) {
                            if !summary.is_empty() && !is_open {
                                output.push_str(&format!("Summary: {}\n", summary));
                            }
                        }

                        if let Some(items) = block.get("items").and_then(|v| v.as_array()) {
                            output.push_str(&format!("Items ({}):\n", items.len()));
                            for item in items {
                                let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                                let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                let importance = item.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                let icon = match kind {
                                    "fact" => "-",
                                    "decision" => "*",
                                    "snippet" => ">",
                                    "warning" => "!",
                                    _ => "?",
                                };
                                output.push_str(&format!("  {} [{}] {}\n", icon, kind, content));
                                if importance > 0.7 {
                                    output.push_str(&format!("    (importance: {:.1})\n", importance));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        output.push_str(&format!("  (Error fetching block: {})\n", e));
                    }
                }
            }
        }
    }

    Ok(vec![Content::text(output)])
}

// ============================================================================
// Search Formatting
// ============================================================================

/// Format search results as summaries only
fn format_search_summaries(result: &serde_json::Value, query: &str) -> Result<String> {
    let mut output = format!("Cache search results for: \"{}\"\n", query);
    output.push_str(&"-".repeat(50));
    output.push('\n');

    if let Some(matches) = result.get("matches").and_then(|v| v.as_array()) {
        if matches.is_empty() {
            output.push_str("No matching blocks found.\n");
        } else {
            for (idx, m) in matches.iter().enumerate() {
                let block_id = m.get("block_id").and_then(|v| v.as_str()).unwrap_or("?");
                let summary = m.get("summary").and_then(|v| v.as_str()).unwrap_or("");
                let relevance = m.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let created = m.get("created_at").and_then(|v| v.as_str()).unwrap_or("");

                output.push_str(&format!(
                    "\n{}. Block: {} (relevance: {:.2})\n   Created: {}\n   Summary: {}\n",
                    idx + 1, block_id, relevance, created,
                    if summary.len() > 200 { &summary[..200] } else { summary }
                ));
            }
            output.push_str(&format!(
                "\nTip: Use include_content=true to fetch full block content, or block_id to get a specific block.\n"
            ));
        }
    }

    Ok(output)
}

/// Format search results with full block content
async fn format_search_with_content(
    client: &crate::amp_client::AmpClient,
    result: &serde_json::Value,
    query: &str,
) -> Result<Vec<Content>> {
    let mut output = format!("Cache search results for: \"{}\" (with content)\n", query);
    output.push_str(&"=".repeat(50));
    output.push('\n');

    if let Some(matches) = result.get("matches").and_then(|v| v.as_array()) {
        if matches.is_empty() {
            output.push_str("No matching blocks found.\n");
        } else {
            for (idx, m) in matches.iter().enumerate() {
                let block_id = m.get("block_id").and_then(|v| v.as_str()).unwrap_or("?");
                let relevance = m.get("relevance").and_then(|v| v.as_f64()).unwrap_or(0.0);

                output.push_str(&format!(
                    "\n[{}/{}] Block: {} (relevance: {:.2})\n",
                    idx + 1, matches.len(), block_id, relevance
                ));
                output.push_str(&"-".repeat(40));
                output.push('\n');

                // Fetch full block content
                match client.cache_block_get(block_id).await {
                    Ok(block) => {
                        if let Some(summary) = block.get("summary").and_then(|v| v.as_str()) {
                            if !summary.is_empty() {
                                output.push_str(&format!("Summary: {}\n\n", summary));
                            }
                        }

                        if let Some(items) = block.get("items").and_then(|v| v.as_array()) {
                            for item in items {
                                let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                                let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
                                let icon = match kind {
                                    "fact" => "-",
                                    "decision" => "*",
                                    "snippet" => ">",
                                    "warning" => "!",
                                    _ => "?",
                                };
                                output.push_str(&format!("  {} [{}] {}\n", icon, kind, content));
                            }
                        }
                    }
                    Err(e) => {
                        output.push_str(&format!("  (Error fetching block: {})\n", e));
                    }
                }
            }
        }
    }

    Ok(vec![Content::text(output)])
}

fn format_block(result: &serde_json::Value) -> Result<String> {
    let block_id = result.get("block_id").and_then(|v| v.as_str()).unwrap_or("?");
    let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("?");
    let token_count = result.get("token_count").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut output = format!("Cache Block: {}\n", block_id);
    output.push_str(&format!("Status: {} | Tokens: {}\n", status, token_count));
    output.push_str(&"-".repeat(50));
    output.push('\n');

    if let Some(summary) = result.get("summary").and_then(|v| v.as_str()) {
        if !summary.is_empty() {
            output.push_str(&format!("Summary: {}\n\n", summary));
        }
    }

    if let Some(items) = result.get("items").and_then(|v| v.as_array()) {
        for item in items {
            let kind = item.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
            let content = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let importance = item.get("importance").and_then(|v| v.as_f64()).unwrap_or(0.5);

            let icon = match kind {
                "fact" => "-",
                "decision" => "*",
                "snippet" => ">",
                "warning" => "!",
                _ => "?",
            };

            output.push_str(&format!("  {} [{}] {}\n", icon, kind, content));
            if importance > 0.7 {
                output.push_str(&format!("    (importance: {:.1})\n", importance));
            }
        }
    }

    Ok(output)
}

fn format_memory_pack(result: &serde_json::Value, input: &AmpCacheGetInput) -> Result<String> {
    let mut output = format!("Memory Pack for scope: {}\n", input.scope_id);
    output.push_str(&"-".repeat(50));
    output.push('\n');

    if let Some(summary) = result.get("summary").and_then(|v| v.as_str()) {
        if !summary.is_empty() {
            output.push_str(&format!("Summary: {}\n\n", summary));
        }
    }

    // Facts
    if let Some(facts) = result.get("facts").and_then(|v| v.as_array()) {
        if !facts.is_empty() {
            output.push_str("Facts:\n");
            for fact in facts.iter().take(10) {
                if let Some(preview) = fact.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  - {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Decisions
    if let Some(decisions) = result.get("decisions").and_then(|v| v.as_array()) {
        if !decisions.is_empty() {
            output.push_str("Decisions:\n");
            for decision in decisions.iter().take(5) {
                if let Some(preview) = decision.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  * {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Snippets
    if let Some(snippets) = result.get("snippets").and_then(|v| v.as_array()) {
        if !snippets.is_empty() {
            output.push_str("Snippets:\n");
            for snippet in snippets.iter().take(3) {
                if let Some(preview) = snippet.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  > {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Warnings
    if let Some(warnings) = result.get("warnings").and_then(|v| v.as_array()) {
        if !warnings.is_empty() {
            output.push_str("Warnings:\n");
            for warning in warnings.iter().take(3) {
                if let Some(preview) = warning.get("preview").and_then(|v| v.as_str()) {
                    output.push_str(&format!("  ! {}\n", preview));
                }
            }
            output.push('\n');
        }
    }

    // Metadata
    if let Some(token_count) = result.get("token_count").and_then(|v| v.as_u64()) {
        output.push_str(&format!(
            "Token count: {} / {}\n",
            token_count,
            input.token_budget.unwrap_or(600)
        ));
    }

    if let Some(version) = result.get("version").and_then(|v| v.as_u64()) {
        output.push_str(&format!("Version: {}\n", version));
    }

    if let Some(is_fresh) = result.get("is_fresh").and_then(|v| v.as_bool()) {
        output.push_str(&format!("Fresh: {}\n", if is_fresh { "yes" } else { "no" }));
    }

    Ok(output)
}
