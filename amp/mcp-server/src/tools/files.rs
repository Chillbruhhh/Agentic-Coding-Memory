use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpFilelogGetInput {
    pub path: String,
}

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

    Ok(vec![Content::text(format!("File log updated: {}", serde_json::to_string_pretty(&result)?))])
}

pub async fn handle_file_content_get(
    client: &crate::amp_client::AmpClient,
    input: AmpFileContentGetInput,
) -> Result<Vec<Content>> {
    let normalized = normalize_request_path(&input.path);
    let mut result = client.get_file_content(&normalized, input.max_chars).await?;
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

    let response = serde_json::json!({
        "input_path": input.path,
        "normalized_path": normalized,
        "tried_paths": tried,
        "resolved_path": resolved,
        "error": result.get("error").cloned(),
    });

    Ok(vec![Content::text(serde_json::to_string_pretty(&response)?)])
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
