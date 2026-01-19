use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct AmpClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl AmpClient {
    pub fn new(base_url: String, timeout_secs: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(10)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            timeout: Duration::from_secs(timeout_secs),
        })
    }

    // Health check
    pub async fn health(&self) -> Result<Value> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Analytics
    pub async fn analytics(&self) -> Result<Value> {
        let url = format!("{}/v1/analytics", self.base_url);
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Query endpoint
    pub async fn query(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/query", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Create object
    pub async fn create_object(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/objects", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Get object
    pub async fn get_object(&self, id: &str) -> Result<Value> {
        let url = format!("{}/v1/objects/{}", self.base_url, id);
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Update object
    pub async fn update_object(&self, id: &str, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/objects/{}", self.base_url, id);
        let response = self.client.put(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Get relationships
    pub async fn get_relationships(&self, params: Value) -> Result<Value> {
        let url = format!("{}/v1/relationships", self.base_url);
        let response = self.client.get(&url).query(&params).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Get file log
    pub async fn get_file_log(&self, path: &str) -> Result<Value> {
        let url = format!("{}/v1/codebase/file-logs/{}", self.base_url, urlencoding::encode(path));
        let response = self.client.get(&url).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Update file log
    pub async fn update_file_log(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/codebase/update-file-log", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Acquire lease
    pub async fn acquire_lease(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/leases/acquire", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Release lease
    pub async fn release_lease(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/leases/release", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }
}
