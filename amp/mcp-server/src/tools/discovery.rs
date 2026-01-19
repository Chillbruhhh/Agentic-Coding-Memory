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
        "text": "",
        "mode": "text",
        "limit": input.limit.unwrap_or(10)
    });

    if let Some(obj_type) = input.object_type {
        query["filters"] = serde_json::json!({
            "type": obj_type
        });
    }

    let result = client.query(query).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}
