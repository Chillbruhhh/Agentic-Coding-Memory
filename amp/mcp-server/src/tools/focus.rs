use anyhow::{anyhow, Result};
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FocusAction {
    List,
    Get,
    Set,
    Complete,
    End,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AmpFocusInput {
    /// Action to perform: list | get | set | complete | end
    pub action: FocusAction,
    /// Optional run/session ID (defaults to current connection run)
    #[serde(default)]
    pub run_id: Option<String>,
    /// Focus title / task summary
    #[serde(default)]
    pub title: Option<String>,
    /// Optional plan steps for the focus
    #[serde(default)]
    pub plan: Option<Vec<String>>,
    /// Completion summary (used with complete)
    #[serde(default)]
    pub summary: Option<String>,
    /// Files touched or created during the focus
    #[serde(default)]
    pub files_changed: Option<Vec<String>>,
    /// Optional project ID filter (list)
    #[serde(default)]
    pub project_id: Option<String>,
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn extract_array(value: &Value) -> Vec<Value> {
    if let Some(results) = value.get("results").and_then(|v| v.as_array()) {
        return results
            .iter()
            .map(|entry| entry.get("object").cloned().unwrap_or_else(|| entry.clone()))
            .collect();
    }
    if let Some(arr) = value.as_array() {
        return arr.clone();
    }
    Vec::new()
}

fn get_outputs(run: &Value) -> Vec<Value> {
    run.get("outputs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

pub async fn handle_focus(
    client: &crate::amp_client::AmpClient,
    current_run_id: Option<&str>,
    input: AmpFocusInput,
) -> Result<Vec<Content>> {
    match input.action {
        FocusAction::List => {
            let connections = client.list_connections().await?;
            let mut output = String::from("Active sessions:\n");
            let list = extract_array(&connections);

            for connection in &list {
                if let Some(filter_project) = &input.project_id {
                    let connection_project = connection.get("project_id").and_then(|v| v.as_str());
                    if connection_project != Some(filter_project.as_str()) {
                        continue;
                    }
                }
                let agent_name = connection
                    .get("agent_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let run_id = connection.get("run_id").and_then(|v| v.as_str());
                let status = connection
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let mut focus_summary = None;
                if let Some(run_id) = run_id {
                    if let Ok(run) = client.get_object(run_id).await {
                        focus_summary = run
                            .get("input_summary")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }

                output.push_str(&format!(
                    "- {} | status: {} | run: {} | focus: {}\n",
                    agent_name,
                    status,
                    run_id.unwrap_or("none"),
                    focus_summary.unwrap_or_else(|| "unknown".to_string())
                ));
            }

            if list.is_empty() {
                output.push_str("No active connections found.\n");
            }

            return Ok(vec![Content::text(output)]);
        }
        FocusAction::Get => {
            let run_id = input
                .run_id
                .as_deref()
                .or(current_run_id)
                .ok_or_else(|| anyhow!("run_id required for get"))?;
            let run = client.get_object(run_id).await?;
            let focus = run.get("focus").cloned().unwrap_or(Value::Null);
            let outputs = run.get("outputs").cloned().unwrap_or(Value::Null);

            let output = serde_json::json!({
                "run_id": run_id,
                "input_summary": run.get("input_summary"),
                "focus": focus,
                "outputs": outputs,
                "status": run.get("status"),
                "cache_scope_id": format!("run:{}", run_id),
            });

            return Ok(vec![Content::text(serde_json::to_string_pretty(&output)?)]); 
        }
        FocusAction::Set => {
            let run_id = input
                .run_id
                .as_deref()
                .or(current_run_id)
                .ok_or_else(|| anyhow!("run_id required for set"))?;
            let title = input
                .title
                .clone()
                .ok_or_else(|| anyhow!("title required for set"))?;

            let focus = serde_json::json!({
                "title": title,
                "plan": input.plan,
                "status": "active",
                "started_at": now_rfc3339()
            });

            let mut payload = serde_json::json!({
                "input_summary": focus.get("title"),
                "focus": focus,
                "status": "running"
            });

            if let Some(project_id) = &input.project_id {
                payload["project_id"] = serde_json::Value::String(project_id.clone());
            }

            client.update_object(run_id, payload).await?;
            return Ok(vec![Content::text(format!(
                "Focus set for run {}",
                run_id
            ))]);
        }
        FocusAction::Complete => {
            let run_id = input
                .run_id
                .as_deref()
                .or(current_run_id)
                .ok_or_else(|| anyhow!("run_id required for complete"))?;

            let run = client.get_object(run_id).await?;
            let mut outputs = get_outputs(&run);

            let title = input.title.clone().or_else(|| {
                run.get("focus")
                    .and_then(|v| v.get("title"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });

            let summary = input
                .summary
                .clone()
                .or_else(|| title.clone())
                .unwrap_or_else(|| "Focus completed".to_string());

            let output_entry = serde_json::json!({
                "type": "focus",
                "content": summary,
                "metadata": {
                    "kind": "focus",
                    "status": "completed",
                    "title": title,
                    "plan": input.plan,
                    "files_changed": input.files_changed,
                    "completed_at": now_rfc3339()
                }
            });

            outputs.push(output_entry);

            let focus = serde_json::json!({
                "title": title,
                "plan": input.plan,
                "status": "completed",
                "completed_at": now_rfc3339()
            });

            let payload = serde_json::json!({
                "input_summary": "No active focus",
                "focus": focus,
                "outputs": outputs
            });

            client.update_object(run_id, payload).await?;
            return Ok(vec![Content::text(format!(
                "Focus marked complete for run {}",
                run_id
            ))]);
        }
        FocusAction::End => {
            let run_id = input
                .run_id
                .as_deref()
                .or(current_run_id)
                .ok_or_else(|| anyhow!("run_id required for end"))?;
            let payload = serde_json::json!({
                "status": "completed"
            });
            client.update_object(run_id, payload).await?;
            return Ok(vec![Content::text(format!(
                "Session ended for run {}",
                run_id
            ))]);
        }
    }
}
