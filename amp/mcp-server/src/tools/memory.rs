use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpWriteDecisionInput {
    pub title: String,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub alternatives: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpWriteChangesetInput {
    pub description: String,
    pub files_changed: Vec<String>,
    pub diff_summary: String,
    pub linked_decisions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpRunStartInput {
    pub goal: String,
    pub repo_id: String,
    pub agent_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpRunEndInput {
    pub run_id: String,
    pub status: String,
    pub outputs: Vec<String>,
    pub summary: String,
}

pub async fn handle_write_decision(
    client: &crate::amp_client::AmpClient,
    input: AmpWriteDecisionInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "type": "decision",
        "content": {
            "title": input.title,
            "context": input.context,
            "decision": input.decision,
            "consequences": input.consequences,
            "alternatives": input.alternatives.unwrap_or_default()
        }
    });

    let result = client.create_object(payload).await?;

    Ok(vec![Content::text(format!("Decision created: {}", serde_json::to_string_pretty(&result)?))])
}

pub async fn handle_write_changeset(
    client: &crate::amp_client::AmpClient,
    input: AmpWriteChangesetInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "type": "changeset",
        "content": {
            "description": input.description,
            "files_changed": input.files_changed,
            "diff_summary": input.diff_summary,
            "linked_decisions": input.linked_decisions.unwrap_or_default()
        }
    });

    let result = client.create_object(payload).await?;

    Ok(vec![Content::text(format!("ChangeSet created: {}", serde_json::to_string_pretty(&result)?))])
}

pub async fn handle_run_start(
    client: &crate::amp_client::AmpClient,
    input: AmpRunStartInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "type": "run",
        "content": {
            "goal": input.goal,
            "repo_id": input.repo_id,
            "agent_name": input.agent_name,
            "status": "running"
        }
    });

    let result = client.create_object(payload).await?;

    Ok(vec![Content::text(format!("Run started: {}", serde_json::to_string_pretty(&result)?))])
}

pub async fn handle_run_end(
    client: &crate::amp_client::AmpClient,
    input: AmpRunEndInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "status": input.status,
        "outputs": input.outputs,
        "summary": input.summary
    });

    let result = client.update_object(&input.run_id, payload).await?;

    Ok(vec![Content::text(format!("Run completed: {}", serde_json::to_string_pretty(&result)?))])
}
