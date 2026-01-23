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
pub struct AmpWriteArtifactInput {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub title: String,
    pub project_id: Option<String>,
    pub agent_id: Option<String>,
    pub run_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub context: Option<String>,
    pub decision: Option<String>,
    pub consequences: Option<String>,
    pub alternatives: Option<Vec<String>>,
    pub status: Option<String>,
    pub file_path: Option<String>,
    pub summary: Option<String>,
    pub symbols: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub diff_summary: Option<String>,
    pub files_changed: Option<Vec<String>>,
    pub linked_objects: Option<Vec<String>>,
    pub linked_decisions: Option<Vec<String>>,
    pub linked_files: Option<Vec<String>>,
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
        "tenant_id": "default",
        "project_id": input.repo_id,
        "provenance": {
            "agent": input.agent_name,
            "summary": input.goal,
            "model": null,
            "tools": null
        },
        "input_summary": input.goal,
        "status": "running"
    });

    let result = client.create_object(payload).await?;

    Ok(vec![Content::text(format!("Run started: {}", serde_json::to_string_pretty(&result)?))])
}

pub async fn handle_run_end(
    client: &crate::amp_client::AmpClient,
    input: AmpRunEndInput,
) -> Result<Vec<Content>> {
    let mut outputs = Vec::new();
    if !input.summary.trim().is_empty() {
        outputs.push(serde_json::json!({
            "type": "response",
            "content": input.summary,
            "metadata": { "kind": "summary" }
        }));
    }
    outputs.extend(input.outputs.into_iter().map(|output| {
        serde_json::json!({
            "type": "response",
            "content": output
        })
    }));

    let payload = serde_json::json!({
        "status": input.status,
        "outputs": outputs
    });

    let result = client.update_object(&input.run_id, payload).await?;

    Ok(vec![Content::text(format!("Run completed: {}", serde_json::to_string_pretty(&result)?))])
}

pub async fn handle_write_artifact(
    client: &crate::amp_client::AmpClient,
    input: AmpWriteArtifactInput,
) -> Result<Vec<Content>> {
    let mut payload = serde_json::Map::new();
    payload.insert("type".to_string(), serde_json::Value::String(input.artifact_type));
    payload.insert("title".to_string(), serde_json::Value::String(input.title));

    let mut insert_optional = |key: &str, value: Option<serde_json::Value>| {
        if let Some(value) = value {
            payload.insert(key.to_string(), value);
        }
    };

    insert_optional("project_id", input.project_id.map(serde_json::Value::String));
    insert_optional("agent_id", input.agent_id.map(serde_json::Value::String));
    insert_optional("run_id", input.run_id.map(serde_json::Value::String));
    insert_optional("tags", input.tags.map(|value| serde_json::json!(value)));
    insert_optional("context", input.context.map(serde_json::Value::String));
    insert_optional("decision", input.decision.map(serde_json::Value::String));
    insert_optional("consequences", input.consequences.map(serde_json::Value::String));
    insert_optional("alternatives", input.alternatives.map(|value| serde_json::json!(value)));
    insert_optional("status", input.status.map(serde_json::Value::String));
    insert_optional("file_path", input.file_path.map(serde_json::Value::String));
    insert_optional("summary", input.summary.map(serde_json::Value::String));
    insert_optional("symbols", input.symbols.map(|value| serde_json::json!(value)));
    insert_optional("dependencies", input.dependencies.map(|value| serde_json::json!(value)));
    insert_optional("content", input.content.map(serde_json::Value::String));
    insert_optional("category", input.category.map(serde_json::Value::String));
    insert_optional("description", input.description.map(serde_json::Value::String));
    insert_optional("diff_summary", input.diff_summary.map(serde_json::Value::String));
    insert_optional("files_changed", input.files_changed.map(|value| serde_json::json!(value)));
    insert_optional("linked_objects", input.linked_objects.map(|value| serde_json::json!(value)));
    insert_optional("linked_decisions", input.linked_decisions.map(|value| serde_json::json!(value)));
    insert_optional("linked_files", input.linked_files.map(|value| serde_json::json!(value)));

    let result = client.write_artifact(serde_json::Value::Object(payload)).await?;

    Ok(vec![Content::text(format!(
        "Artifact created: {}",
        serde_json::to_string_pretty(&result)?
    ))])
}
