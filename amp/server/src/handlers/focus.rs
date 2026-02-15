use axum::{extract::State, http::StatusCode, response::Json};
use serde::Deserialize;
use serde_json::Value;
use tokio::time::{timeout, Duration};

use crate::surreal_json::take_json_values;
use crate::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FocusAction {
    List,
    Get,
    Set,
    Complete,
    End,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FocusRequest {
    pub action: FocusAction,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub plan: Option<Vec<String>>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub files_changed: Option<Vec<String>>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub include_expired: Option<bool>,
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn normalize_run_id(id: &str) -> String {
    id.trim()
        .trim_start_matches("objects:")
        .trim_start_matches("run:")
        .to_string()
}

async fn get_run_object(state: &AppState, run_id: &str) -> Result<Value, (StatusCode, String)> {
    let query = "SELECT VALUE { id: string::concat(id), input_summary: input_summary, focus: focus, outputs: outputs, status: status, project_id: project_id } FROM objects WHERE id = type::thing('objects', $id)";

    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.query(query).bind(("id", run_id.to_string())),
    )
    .await;

    match result {
        Ok(Ok(mut response)) => {
            let mut values = take_json_values(&mut response, 0);
            if values.is_empty() {
                return Err((StatusCode::NOT_FOUND, "Run not found".to_string()));
            }
            Ok(values.remove(0))
        }
        Ok(Err(e)) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        Err(_) => Err((StatusCode::GATEWAY_TIMEOUT, "Timeout retrieving run".to_string())),
    }
}

async fn update_run_object(
    state: &AppState,
    run_id: &str,
    payload: Value,
) -> Result<(), (StatusCode, String)> {
    let query = "UPDATE type::thing('objects', $id) MERGE $data";

    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state
            .db
            .client
            .query(query)
            .bind(("id", run_id.to_string()))
            .bind(("data", payload)),
    )
    .await;

    match result {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
        Err(_) => Err((StatusCode::GATEWAY_TIMEOUT, "Timeout updating run".to_string())),
    }
}

pub async fn handle_focus(
    State(state): State<AppState>,
    Json(input): Json<FocusRequest>,
) -> Result<Json<Value>, (StatusCode, String)> {
    match input.action {
        FocusAction::List => {
            let include_expired = input.include_expired.unwrap_or(false);

            let mut conditions: Vec<&str> = Vec::new();
            if !include_expired {
                conditions.push("expires_at > time::now()");
            }
            if input.project_id.is_some() {
                conditions.push("project_id = $project_id");
            }

            let where_clause = if conditions.is_empty() {
                String::new()
            } else {
                format!(" WHERE {}", conditions.join(" AND "))
            };

            // SurrealDB can reject ORDER BY when using `SELECT VALUE { ... }` unless ordering is
            // performed inside a subselect that explicitly selects the ordering idiom.
            let query = format!(
                "SELECT VALUE {{ agent_name: agent_name, run_id: run_id, project_id: project_id, status: status, last_heartbeat: last_heartbeat }} FROM (SELECT agent_name, run_id, project_id, status, last_heartbeat, expires_at FROM agent_connections{} ORDER BY last_heartbeat DESC)",
                where_clause
            );

            let mut q = state.db.client.query(&query);
            if let Some(project_id) = &input.project_id {
                q = q.bind(("project_id", project_id.clone()));
            }

            let result: Result<Result<surrealdb::Response, _>, _> =
                timeout(Duration::from_secs(5), q).await;

            let mut sessions: Vec<Value> = match result {
                Ok(Ok(mut response)) => take_json_values(&mut response, 0),
                Ok(Err(e)) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
                Err(_) => {
                    return Err((
                        StatusCode::GATEWAY_TIMEOUT,
                        "Timeout listing sessions".to_string(),
                    ))
                }
            };

            // Attach focus summaries when possible (best-effort).
            for session in &mut sessions {
                let run_id = session.get("run_id").and_then(|v| v.as_str());
                if let Some(run_id) = run_id {
                    let run_id = normalize_run_id(run_id);
                    if let Ok(run) = get_run_object(&state, &run_id).await {
                        session["input_summary"] =
                            run.get("input_summary").cloned().unwrap_or(Value::Null);
                        session["focus"] = run.get("focus").cloned().unwrap_or(Value::Null);
                    }
                }
            }

            Ok(Json(serde_json::json!({ "sessions": sessions })))
        }
        FocusAction::Get => {
            let run_id = input
                .run_id
                .as_deref()
                .ok_or((StatusCode::BAD_REQUEST, "run_id required for get".to_string()))?;
            let run_id = normalize_run_id(run_id);

            let run = get_run_object(&state, &run_id).await?;
            Ok(Json(serde_json::json!({
                "run_id": run_id,
                "input_summary": run.get("input_summary"),
                "focus": run.get("focus"),
                "outputs": run.get("outputs"),
                "status": run.get("status"),
                "cache_scope_id": format!("run:{}", run_id),
            })))
        }
        FocusAction::Set => {
            let run_id = input
                .run_id
                .as_deref()
                .ok_or((StatusCode::BAD_REQUEST, "run_id required for set".to_string()))?;
            let run_id = normalize_run_id(run_id);
            let title = input
                .title
                .clone()
                .ok_or((StatusCode::BAD_REQUEST, "title required for set".to_string()))?;

            let focus = serde_json::json!({
                "title": title,
                "plan": input.plan,
                "status": "active",
                "started_at": now_rfc3339()
            });

            let mut payload = serde_json::json!({
                "input_summary": focus.get("title"),
                "focus": focus,
                "status": "running",
                "updated_at": now_rfc3339(),
            });

            if let Some(project_id) = &input.project_id {
                payload["project_id"] = Value::String(project_id.clone());
            }

            update_run_object(&state, &run_id, payload).await?;
            Ok(Json(serde_json::json!({ "ok": true, "message": "Focus set", "run_id": run_id })))
        }
        FocusAction::Complete => {
            let run_id = input
                .run_id
                .as_deref()
                .ok_or((StatusCode::BAD_REQUEST, "run_id required for complete".to_string()))?;
            let run_id = normalize_run_id(run_id);

            let run = get_run_object(&state, &run_id).await?;
            let mut outputs = run
                .get("outputs")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

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
                "outputs": outputs,
                "updated_at": now_rfc3339(),
            });

            update_run_object(&state, &run_id, payload).await?;
            Ok(Json(serde_json::json!({ "ok": true, "message": "Focus marked complete", "run_id": run_id })))
        }
        FocusAction::End => {
            let run_id = input
                .run_id
                .as_deref()
                .ok_or((StatusCode::BAD_REQUEST, "run_id required for end".to_string()))?;
            let run_id = normalize_run_id(run_id);

            let payload = serde_json::json!({
                "status": "completed",
                "updated_at": now_rfc3339(),
            });
            update_run_object(&state, &run_id, payload).await?;
            Ok(Json(serde_json::json!({ "ok": true, "message": "Session ended", "run_id": run_id })))
        }
    }
}
