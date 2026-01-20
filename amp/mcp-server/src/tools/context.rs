use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpContextInput {
    pub goal: String,
    pub scope: String,
    #[serde(default)]
    pub include_recent: bool,
    #[serde(default)]
    pub include_decisions: bool,
}

pub async fn handle_amp_context(
    client: &crate::amp_client::AmpClient,
    input: AmpContextInput,
) -> Result<Vec<Content>> {
    let mut query = serde_json::json!({
        "text": input.goal,
        "hybrid": true,
        "limit": 10
    });

    let mut filters = serde_json::Map::new();
    
    if input.include_decisions {
        filters.insert("type".to_string(), serde_json::json!(["decision"]));
    }

    if !filters.is_empty() {
        query["filters"] = serde_json::Value::Object(filters);
    }

    let result = client.query(query).await?;
    
    // Summarize context instead of returning raw JSON
    let summary = summarize_context_results(&result, &input)?;

    Ok(vec![Content::text(summary)])
}

fn summarize_context_results(result: &serde_json::Value, input: &AmpContextInput) -> Result<String> {
    let mut summary = format!("Context for: {}\nScope: {}\n\n", input.goal, input.scope);
    
    if let Some(results) = result.get("results").and_then(|r| r.as_array()) {
        summary.push_str(&format!("Found {} relevant items:\n\n", results.len()));
        
        let mut symbols = Vec::new();
        let mut decisions = Vec::new();
        let mut files = Vec::new();
        
        for item in results.iter().take(15) {
            // Query results are wrapped: { "object": {...}, "score": 0.0 }
            let obj = item.get("object").unwrap_or(item);

            if let Some(obj_type) = obj.get("type").and_then(|t| t.as_str()) {
                match obj_type {
                    "symbol" => symbols.push(obj),
                    "decision" => decisions.push(obj),
                    "FileChunk" | "filechunk" | "FileLog" | "filelog" => files.push(obj),
                    _ => {}
                }
            }
        }

        if !symbols.is_empty() {
            summary.push_str("Key Symbols:\n");
            for (i, symbol) in symbols.iter().take(5).enumerate() {
                let name = symbol.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                let kind = symbol.get("kind").and_then(|k| k.as_str()).unwrap_or("unknown");
                let path = symbol.get("path").and_then(|p| p.as_str()).unwrap_or("unknown");
                summary.push_str(&format!("  {}. {} ({}) in {}\n", i+1, name, kind, path));
            }
            summary.push('\n');
        }

        if !decisions.is_empty() {
            summary.push_str("Relevant Decisions:\n");
            for (i, decision) in decisions.iter().take(3).enumerate() {
                let title = decision.get("title").and_then(|t| t.as_str()).unwrap_or("unknown");
                let status = decision.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
                summary.push_str(&format!("  {}. {} ({})\n", i+1, title, status));
            }
            summary.push('\n');
        }

        if !files.is_empty() {
            summary.push_str("Related Files:\n");
            for (i, file) in files.iter().take(5).enumerate() {
                let path = file.get("file_path")
                    .or_else(|| file.get("path"))
                    .and_then(|p| p.as_str())
                    .unwrap_or("unknown");
                summary.push_str(&format!("  {}. {}\n", i+1, path));
            }
        }
        
    } else {
        summary.push_str("No relevant context found\n");
    }
    
    Ok(summary)
}
