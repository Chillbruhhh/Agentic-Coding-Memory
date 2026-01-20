use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpStatusInput {}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpListInput {
    #[serde(rename = "type")]
    pub object_type: Option<String>,
    pub limit: Option<i32>,
    pub sort: Option<String>,
}

pub async fn handle_amp_status(client: &crate::amp_client::AmpClient) -> Result<Vec<Content>> {
    let health = client.health().await?;
    let analytics = client.analytics().await?;

    let result = serde_json::json!({
        "health": health,
        "analytics": analytics
    });

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

pub async fn handle_amp_list(
    client: &crate::amp_client::AmpClient,
    input: AmpListInput,
) -> Result<Vec<Content>> {
    let mut query = serde_json::json!({
        "limit": input.limit.unwrap_or(10).min(20)  // Cap at 20 items
    });

    if let Some(obj_type) = &input.object_type {
        query["filters"] = serde_json::json!({
            "type": [obj_type]
        });
    }

    let result = client.query(query).await?;
    
    // Summarize list instead of returning raw JSON
    let summary = summarize_list_results(&result, &input)?;

    Ok(vec![Content::text(summary)])
}

fn summarize_list_results(result: &serde_json::Value, input: &AmpListInput) -> Result<String> {
    let type_filter = input.object_type.as_deref().unwrap_or("all");
    let mut summary = format!("List of {} objects:\n\n", type_filter);
    
    if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
        if results.is_empty() {
            summary.push_str("No objects found\n");
            return Ok(summary);
        }
        
        summary.push_str(&format!("Found {} objects:\n\n", results.len()));

        for (i, item) in results.iter().enumerate() {
            // Query results are wrapped: { "object": {...}, "score": 0.0 }
            // Extract the actual object from the wrapper
            let obj = item.get("object").unwrap_or(item);

            let obj_type = obj.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
            let id = obj.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
            let short_id = &id[..8.min(id.len())];

            match obj_type {
                "symbol" => {
                    let name = obj.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    let kind = obj.get("kind").and_then(|k| k.as_str()).unwrap_or("unknown");
                    let path = obj.get("path").and_then(|p| p.as_str()).unwrap_or("");
                    summary.push_str(&format!("{}. Symbol: {} ({}) in {}\n", i+1, name, kind, path));
                },
                "decision" => {
                    let title = obj.get("title").and_then(|t| t.as_str()).unwrap_or("unknown");
                    summary.push_str(&format!("{}. Decision: {}\n", i+1, title));
                },
                "changeset" => {
                    let desc = obj.get("description").and_then(|d| d.as_str()).unwrap_or("unknown");
                    let desc_short = if desc.len() > 50 { &desc[..50] } else { desc };
                    summary.push_str(&format!("{}. Changeset: {}\n", i+1, desc_short));
                },
                "FileChunk" | "filechunk" => {
                    let path = obj.get("file_path").and_then(|p| p.as_str()).unwrap_or("unknown");
                    let lines = format!("{}-{}",
                        obj.get("start_line").and_then(|l| l.as_u64()).unwrap_or(0),
                        obj.get("end_line").and_then(|l| l.as_u64()).unwrap_or(0)
                    );
                    summary.push_str(&format!("{}. FileChunk: {} (lines {})\n", i+1, path, lines));
                },
                "FileLog" | "filelog" => {
                    let path = obj.get("file_path").and_then(|p| p.as_str()).unwrap_or("unknown");
                    let purpose = obj.get("purpose").and_then(|p| p.as_str()).unwrap_or("");
                    summary.push_str(&format!("{}. FileLog: {} - {}\n", i+1, path, purpose));
                },
                _ => {
                    summary.push_str(&format!("{}. {}: {}\n", i+1, obj_type, id));
                }
            }
        }
    } else {
        summary.push_str("No results found\n");
    }
    
    Ok(summary)
}
