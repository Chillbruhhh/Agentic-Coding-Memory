#![allow(dead_code)]
use anyhow::Result;
use rmcp::model::Content;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpLeaseAcquireInput {
    pub resource: String,
    pub duration: i32,
    pub agent_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AmpLeaseReleaseInput {
    pub lease_id: String,
}

pub async fn handle_lease_acquire(
    client: &crate::amp_client::AmpClient,
    input: AmpLeaseAcquireInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "resource": input.resource,
        "duration": input.duration,
        "agent_id": input.agent_id
    });

    let result = client.acquire_lease(payload).await?;

    Ok(vec![Content::text(format!(
        "Lease acquired: {}",
        serde_json::to_string_pretty(&result)?
    ))])
}

pub async fn handle_lease_release(
    client: &crate::amp_client::AmpClient,
    input: AmpLeaseReleaseInput,
) -> Result<Vec<Content>> {
    let payload = serde_json::json!({
        "lease_id": input.lease_id
    });

    let result = client.release_lease(payload).await?;

    Ok(vec![Content::text(format!(
        "Lease released: {}",
        serde_json::to_string_pretty(&result)?
    ))])
}
