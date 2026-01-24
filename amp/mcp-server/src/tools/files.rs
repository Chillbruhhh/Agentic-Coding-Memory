use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpFilelogGetInput {
    pub path: String,
}

/// Action type for file sync operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileSyncAction {
    /// File was created
    Create,
    /// File was edited/modified
    Edit,
    /// File was deleted
    Delete,
}

/// Input for amp_file_sync - synchronizes file state across all memory layers
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpFileSyncInput {
    /// Path to the file
    pub path: String,
    /// Action performed on the file
    pub action: FileSyncAction,
    /// Concise summary of what changed and why (1-4 sentences)
    pub summary: String,
    /// Optional run ID for audit trail linkage
    pub run_id: Option<String>,
    /// Optional agent ID for audit trail
    pub agent_id: Option<String>,
}

// Keep legacy input for backward compatibility
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpFilelogUpdateInput {
    pub path: String,
    pub summary: String,
    pub linked_run: Option<String>,
    pub linked_changeset: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpFileContentGetInput {
    pub path: String,
    pub max_chars: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpFilePathResolveInput {
    pub path: String,
}

pub async fn handle_filelog_get(
    client: &crate::amp_client::AmpClient,
    input: AmpFilelogGetInput,
) -> Result<Vec<Content>> {
    let result = client.get_file_log(&input.path).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

pub async fn handle_filelog_update(
    client: &crate::amp_client::AmpClient,
    input: AmpFilelogUpdateInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "path": input.path,
        "summary": input.summary,
        "linked_run": input.linked_run,
        "linked_changeset": input.linked_changeset
    });

    let result = client.update_file_log(payload).await?;

    Ok(vec![Content::text(format!(
        "File log updated: {}",
        serde_json::to_string_pretty(&result)?
    ))])
}

pub async fn handle_file_content_get(
    client: &crate::amp_client::AmpClient,
    input: AmpFileContentGetInput,
) -> Result<Vec<Content>> {
    let normalized = normalize_request_path(&input.path);
    let mut result = client
        .get_file_content(&normalized, input.max_chars)
        .await?;
    if is_not_found(&result) {
        if let Some(alt) = alternate_path(&input.path, &normalized) {
            let retry = client.get_file_content(&alt, input.max_chars).await?;
            if !is_not_found(&retry) {
                result = retry;
            }
        }
    }
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

pub async fn handle_file_path_resolve(
    client: &crate::amp_client::AmpClient,
    input: AmpFilePathResolveInput,
) -> Result<Vec<Content>> {
    let normalized = normalize_request_path(&input.path);
    let mut tried = vec![normalized.clone()];

    let mut result = client.get_file_log(&normalized).await?;
    let mut resolved = extract_file_path(&result);

    if resolved.is_none() {
        if let Some(alt) = alternate_path(&input.path, &normalized) {
            tried.push(alt.clone());
            let retry = client.get_file_log(&alt).await?;
            if let Some(found) = extract_file_path(&retry) {
                resolved = Some(found);
                result = retry;
            }
        }
    }

    if resolved.is_none() && !Path::new(&input.path).is_absolute() {
        let project_roots = fetch_project_roots(client).await?;
        for root in project_roots {
            if root.trim().is_empty() || root == "." {
                continue;
            }
            let candidate = Path::new(&root).join(&input.path);
            let candidate_str = candidate.to_string_lossy().to_string();
            tried.push(candidate_str.clone());
            let attempt = client.get_file_log(&candidate_str).await?;
            if let Some(found) = extract_file_path(&attempt) {
                resolved = Some(found);
                result = attempt;
                break;
            }
        }
    }

    let response = serde_json::json!({
        "input_path": input.path,
        "normalized_path": normalized,
        "tried_paths": tried,
        "resolved_path": resolved,
        "error": result.get("error").cloned(),
    });

    Ok(vec![Content::text(serde_json::to_string_pretty(
        &response,
    )?)])
}

fn is_not_found(result: &Value) -> bool {
    result
        .get("error")
        .and_then(|value| value.as_str())
        .map(|value| value.eq_ignore_ascii_case("File content not found"))
        .unwrap_or(false)
}

fn normalize_request_path(path: &str) -> String {
    let trimmed = path.trim();
    if cfg!(windows) {
        trimmed.replace('/', "\\")
    } else {
        trimmed.replace('\\', "/")
    }
}

fn alternate_path(original: &str, normalized: &str) -> Option<String> {
    let trimmed = original.trim();
    if trimmed == normalized {
        return None;
    }
    let alt = if cfg!(windows) {
        trimmed.replace('\\', "/")
    } else {
        trimmed.replace('/', "\\")
    };
    if alt == normalized {
        None
    } else {
        Some(alt)
    }
}

fn extract_file_path(result: &Value) -> Option<String> {
    let file_log = result.get("file_log")?;
    if let Some(path) = file_log.get("file_path").and_then(|value| value.as_str()) {
        return Some(path.to_string());
    }
    if let Some(path) = file_log.get("path").and_then(|value| value.as_str()) {
        return Some(path.to_string());
    }
    None
}

async fn fetch_project_roots(client: &crate::amp_client::AmpClient) -> Result<Vec<String>> {
    let mut roots = Vec::new();
    let payload = serde_json::json!({
        "text": "Project root:",
        "filters": { "type": ["symbol"] },
        "limit": 50
    });

    let response = client.query(payload).await?;
    if let Some(results) = response.get("results").and_then(|v| v.as_array()) {
        for result in results {
            let obj = result.get("object").unwrap_or(result);
            let kind = obj.get("kind").and_then(|v| v.as_str()).unwrap_or_default();
            if kind == "project" {
                if let Some(path) = obj.get("path").and_then(|v| v.as_str()) {
                    roots.push(path.to_string());
                }
            }
        }
    }

    if roots.is_empty() {
        let fallback = serde_json::json!({
            "filters": { "type": ["symbol"] },
            "limit": 200
        });
        let response = client.query(fallback).await?;
        if let Some(results) = response.get("results").and_then(|v| v.as_array()) {
            for result in results {
                let obj = result.get("object").unwrap_or(result);
                let kind = obj.get("kind").and_then(|v| v.as_str()).unwrap_or_default();
                if kind == "project" {
                    if let Some(path) = obj.get("path").and_then(|v| v.as_str()) {
                        roots.push(path.to_string());
                    }
                }
            }
        }
    }

    Ok(roots)
}

/// Handle amp_file_sync - synchronizes file state across all memory layers
/// Updates: temporal (FileLog + audit trail), vector (embeddings), graph (relationships)
pub async fn handle_file_sync(
    client: &crate::amp_client::AmpClient,
    input: AmpFileSyncInput,
) -> Result<Vec<Content>> {
    let action_str = match input.action {
        FileSyncAction::Create => "create",
        FileSyncAction::Edit => "edit",
        FileSyncAction::Delete => "delete",
    };

    let payload = serde_json::json!({
        "path": input.path,
        "action": action_str,
        "summary": input.summary,
        "run_id": input.run_id,
        "agent_id": input.agent_id
    });

    let result = client.file_sync(payload).await?;

    // Check if result indicates ambiguous path - return that directly
    if result.get("status").and_then(|s| s.as_str()) == Some("ambiguous") {
        return Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)]);
    }

    // Format response based on what was synced
    let layers = result.get("layers_updated").cloned().unwrap_or_else(|| {
        serde_json::json!({
            "temporal": false,
            "vector": false,
            "graph": false
        })
    });

    let response = serde_json::json!({
        "status": "synced",
        "file_id": result.get("file_id"),
        "action": action_str,
        "layers_updated": layers,
        "audit_entry_added": result.get("audit_entry_added").unwrap_or(&serde_json::json!(true)),
        "chunks_replaced": result.get("chunks_replaced").unwrap_or(&serde_json::json!(0)),
        "relationships_updated": result.get("relationships_updated").unwrap_or(&serde_json::json!(0))
    });

    Ok(vec![Content::text(serde_json::to_string_pretty(&response)?)])
}
