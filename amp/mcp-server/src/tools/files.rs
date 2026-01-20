use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    let result = client.get_file_content(&input.path, input.max_chars).await?;
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}
