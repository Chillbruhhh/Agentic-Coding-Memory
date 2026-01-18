use crate::amp_client::AmpClient;
use serde_json::Value;
use tauri::command;

#[command]
pub async fn get_amp_data() -> Result<Value, String> {
    let client = AmpClient::new("http://localhost:8105");
    
    match client.get_hierarchy().await {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Failed to fetch AMP data: {}", e)),
    }
}

#[command]
pub async fn query_amp_objects(query: Value) -> Result<Value, String> {
    let client = AmpClient::new("http://localhost:8105");
    
    match client.query_objects(query).await {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Failed to query AMP objects: {}", e)),
    }
}
