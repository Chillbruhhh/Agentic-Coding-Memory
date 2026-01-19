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
        "limit": 20
    });

    let mut filters = serde_json::Map::new();
    
    if input.include_decisions {
        filters.insert("type".to_string(), serde_json::json!(["decision"]));
    }

    if !filters.is_empty() {
        query["filters"] = serde_json::Value::Object(filters);
    }

    let result = client.query(query).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}
