use reqwest::Client;
use serde_json::Value;
use anyhow::Result;
use uuid::Uuid;

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

    pub async fn create_object(&self, object: Value) -> Result<Value> {
        let response = self.client
            .post(&format!("{}/v1/objects", self.base_url))
            .json(&object)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to create object: {}", response.status())
        }
    }

    pub async fn query(&self, query: &str) -> Result<Value> {
        let request_body = serde_json::json!({
            "query": query,
            "variables": {}
        });
        
        let response = self.client
            .post(&format!("{}/v1/query", self.base_url))
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to query: {}", response.status())
        }
    }

    pub async fn query_objects(&self, query_request: serde_json::Value) -> Result<Value> {
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

    pub async fn parse_file(&self, parse_request: serde_json::Value) -> Result<Value> {
        let response = self.client
            .post(&format!("{}/v1/codebase/parse-file", self.base_url))
            .json(&parse_request)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to parse file: {}", response.status())
        }
    }

    pub async fn create_relationship(&self, from_id: &str, to_id: &str, relation_type: &str) -> Result<Value> {
        // The IDs should already be UUIDs, but let's make sure they're clean
        let source_uuid = if from_id.starts_with("objects:") {
            uuid::Uuid::parse_str(&from_id[8..])
                .map_err(|e| anyhow::anyhow!("Invalid source UUID: {}", e))?
        } else {
            uuid::Uuid::parse_str(from_id)
                .map_err(|e| anyhow::anyhow!("Invalid source UUID: {}", e))?
        };
        
        let target_uuid = if to_id.starts_with("objects:") {
            uuid::Uuid::parse_str(&to_id[8..])
                .map_err(|e| anyhow::anyhow!("Invalid target UUID: {}", e))?
        } else {
            uuid::Uuid::parse_str(to_id)
                .map_err(|e| anyhow::anyhow!("Invalid target UUID: {}", e))?
        };
        
        // Map relation type to proper enum
        let relation_enum = match relation_type {
            "contains" => "defined_in", // Use defined_in for contains relationship
            "defines" => "defined_in",
            "depends_on" => "depends_on",
            "calls" => "calls",
            _ => "defined_in", // Default fallback
        };
        
        println!("🔗 Creating relationship: {} -> {} ({})", source_uuid, target_uuid, relation_enum);
        
        let relationship = serde_json::json!({
            "type": relation_enum,
            "source_id": source_uuid,
            "target_id": target_uuid,
            "metadata": null
        });
        
        let response = self.client
            .post(&format!("{}/v1/relationships", self.base_url))
            .json(&relationship)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to create relationship ({}): {}", status, error_text)
        }
    }
    pub async fn create_relationship_direct(&self, from_id: &str, to_id: &str, relation_type: &str) -> Result<Value> {
        // Map relation_type string to proper enum value
        let relation_enum = match relation_type {
            "defined_in" => "defined_in",
            "depends_on" => "depends_on", 
            "calls" => "calls",
            "justified_by" => "justified_by",
            "modifies" => "modifies",
            "implements" => "implements",
            "produced" => "produced",
            _ => "defined_in", // default fallback
        };
        
        let relationship_data = serde_json::json!({
            "type": relation_enum,
            "source_id": from_id,
            "target_id": to_id
        });
        
        println!("🔗 Creating relationship: {} -> {} -> {}", from_id, relation_enum, to_id);
        
        let response = self.client
            .post(&format!("{}/v1/relationships", self.base_url))
            .json(&relationship_data)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result = response.json().await?;
            println!("🔗 Relationship created successfully");
            Ok(result)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to create relationship ({}): {}", status, error_text)
        }
    }
    pub async fn delete_object(&self, id: &str) -> Result<()> {
        let response = self.client
            .delete(&format!("{}/v1/objects/{}", self.base_url, id))
            .send()
            .await?;
        
        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            anyhow::bail!("Failed to delete object {}: {}", id, response.status())
        }
    }

    pub async fn acquire_lease(&self, lease_id: Uuid, owner_pid: u32) -> Result<Value> {
        let request_body = serde_json::json!({
            "resource": format!("session-{}", lease_id),
            "holder": format!("amp-cli-{}", owner_pid),
            "ttl_seconds": 60
        });
        
        let response = self.client
            .post(&format!("{}/v1/leases/acquire", self.base_url))
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to acquire lease: {}", response.status())
        }
    }

    pub async fn renew_lease(&self, lease_id: Uuid) -> Result<Value> {
        let request_body = serde_json::json!({
            "lease_id": lease_id,
            "ttl_seconds": 60
        });
        
        let response = self.client
            .post(&format!("{}/v1/leases/renew", self.base_url))
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to renew lease: {}", response.status())
        }
    }

    pub async fn release_lease(&self, lease_id: Uuid) -> Result<Value> {
        let request_body = serde_json::json!({
            "lease_id": lease_id
        });
        
        let response = self.client
            .post(&format!("{}/v1/leases/release", self.base_url))
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            anyhow::bail!("Failed to release lease: {}", response.status())
        }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AmpClient::new("http://localhost:8105");
        assert_eq!(client.base_url, "http://localhost:8105");
    }
}
