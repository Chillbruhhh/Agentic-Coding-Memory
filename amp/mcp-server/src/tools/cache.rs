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
    pub scope_id: String,
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
    pub scope_id: String,
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
    pub scope_id: String,
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
    input: AmpCacheWriteInput,
) -> Result<Vec<Content>> {
    let kind_str = match input.kind {
        CacheItemKind::Fact => "fact",
        CacheItemKind::Decision => "decision",
        CacheItemKind::Snippet => "snippet",
        CacheItemKind::Warning => "warning",
    };

    let payload = serde_json::json!({
        "scope_id": input.scope_id,
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
    input: AmpCacheCompactInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "scope_id": input.scope_id,
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

    let payload = serde_json::json!({
        "scope_id": input.scope_id,
        "query": input.query,
        "limit": limit,
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
