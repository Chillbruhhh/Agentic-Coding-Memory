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
        let encoded = urlencoding::encode(path);
        let url = format!("{}/v1/codebase/file-log-objects/{}", self.base_url, encoded);
        let response = self.client.get(&url).send().await?;

        let status = response.status();

        if status.is_success() {
            let data = response.json().await?;
            return Ok(data);
        }

        // Handle 409 Conflict (ambiguous path) as a successful response with file list
        if status.as_u16() == 409 {
            let error_data: Value = response.json().await?;
            // Transform the error into an informative success response
            return Ok(serde_json::json!({
                "status": "ambiguous",
                "message": error_data.get("error").and_then(|v| v.as_str()).unwrap_or("Multiple files match"),
                "input_path": error_data.get("input_path"),
                "matching_files": error_data.get("matching_files"),
                "hint": error_data.get("hint").and_then(|v| v.as_str()).unwrap_or("Please use a more specific path (e.g., include parent directory)")
            }));
        }

        // Only fall back for other errors (404, 500, etc.)
        let fallback_url = format!("{}/v1/codebase/file-logs/{}", self.base_url, encoded);
        let response = self.client.get(&fallback_url).send().await?;
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

    // Get stored file content from FileChunk objects
    pub async fn get_file_content(&self, path: &str, max_chars: Option<usize>) -> Result<Value> {
        let encoded = urlencoding::encode(path);
        let mut url = format!("{}/v1/codebase/file-contents/{}", self.base_url, encoded);
        if let Some(limit) = max_chars {
            url = format!("{}?max_chars={}", url, limit);
        }
        let response = self.client.get(&url).send().await?;
        let status = response.status();

        // Handle 409 Conflict (ambiguous path) as a successful response with file list
        if status.as_u16() == 409 {
            let error_data: Value = response.json().await?;
            return Ok(serde_json::json!({
                "status": "ambiguous",
                "message": error_data.get("error").and_then(|v| v.as_str()).unwrap_or("Multiple files match"),
                "input_path": error_data.get("input_path"),
                "matching_files": error_data.get("matching_files"),
                "hint": error_data.get("hint").and_then(|v| v.as_str()).unwrap_or("Please use a more specific path (e.g., include parent directory)")
            }));
        }

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

    // Write artifact
    pub async fn write_artifact(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/artifacts", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Cache get pack
    pub async fn cache_get_pack(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/cache/pack", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // Cache write items
    pub async fn cache_write_items(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/cache/write", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data = response.json().await?;
        Ok(data)
    }

    // File sync - synchronize file state across all memory layers
    pub async fn file_sync(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/codebase/sync", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let status = response.status();

        if status.is_success() {
            let data = response.json().await?;
            return Ok(data);
        }

        // Handle 409 Conflict (ambiguous path) as a successful response with file list
        if status.as_u16() == 409 {
            let error_data: Value = response.json().await?;
            return Ok(serde_json::json!({
                "status": "ambiguous",
                "message": error_data.get("error").and_then(|v| v.as_str()).unwrap_or("Multiple files match"),
                "input_path": error_data.get("input_path"),
                "matching_files": error_data.get("matching_files"),
                "hint": error_data.get("hint").and_then(|v| v.as_str()).unwrap_or("Please use a more specific path (e.g., include parent directory)")
            }));
        }

        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("file_sync failed ({}): {}", status, body);
    }

    // Cache block operations for episodic memory
    pub async fn cache_block_write(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/cache/block/write", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("cache_block_write failed ({}): {}", status, body);
        }
        let data = response.json().await?;
        Ok(data)
    }

    pub async fn cache_block_compact(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/cache/block/compact", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("cache_block_compact failed ({}): {}", status, body);
        }
        let data = response.json().await?;
        Ok(data)
    }

    pub async fn cache_block_search(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/cache/block/search", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("cache_block_search failed ({}): {}", status, body);
        }
        let data = response.json().await?;
        Ok(data)
    }

    pub async fn cache_block_get(&self, block_id: &str) -> Result<Value> {
        let url = format!("{}/v1/cache/block/{}", self.base_url, block_id);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("cache_block_get failed ({}): {}", status, body);
        }
        let data = response.json().await?;
        Ok(data)
    }
}
