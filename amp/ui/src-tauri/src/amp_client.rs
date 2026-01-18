use reqwest::Client;
use serde_json::Value;
use anyhow::Result;

#[derive(Clone)]
pub struct AmpClient {
    client: Client,
    base_url: String,
}

impl AmpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn query_objects(&self, query_request: Value) -> Result<Value> {
        let response = self.client
            .post(&format!("{}/v1/query", self.base_url))
            .json(&query_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to query objects: {}", response.status())
        }
    }

    pub async fn get_hierarchy(&self) -> Result<Value> {
        let query_request = serde_json::json!({
            "text": null,
            "vector": null,
            "filters": null,
            "graph": null,
            "limit": 1000,
            "hybrid": false
        });
        
        self.query_objects(query_request).await
    }
}
