use crate::client::AmpClient;
use anyhow::Result;
use serde_json::json;

pub async fn run_query(text: Option<&str>, relationships: bool, client: &AmpClient) -> Result<()> {
    if !client.health_check().await? {
        anyhow::bail!("AMP server is not available. Please start the server first.");
    }

    if relationships {
        println!("🔍 Checking relationships in database...");
        
        // Check for relationship tables
        let queries = vec![
            "SELECT * FROM contains LIMIT 5",
            "SELECT * FROM defines LIMIT 5", 
            "INFO FOR DB",
        ];
        
        for query in queries {
            println!("\n📋 Query: {}", query);
            match client.query(query).await {
                Ok(result) => {
                    println!("✅ Result: {}", serde_json::to_string_pretty(&result)?);
                }
                Err(e) => {
                    println!("⚠️  Error: {}", e);
                }
            }
        }
    } else if let Some(search_text) = text {
        println!("🔍 Searching for: {}", search_text);
        
        let query_request = json!({
            "text": search_text,
            "limit": 10
        });
        
        match client.query_objects(query_request).await {
            Ok(result) => {
                if let Some(results) = result.get("results") {
                    if let Some(array) = results.as_array() {
                        println!("📊 Found {} results:", array.len());
                        for (i, item) in array.iter().enumerate() {
                            if let Some(object) = item.get("object") {
                                let name = object.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let kind = object.get("kind").and_then(|v| v.as_str()).unwrap_or("unknown");
                                println!("  {}. {} ({})", i + 1, name, kind);
                            }
                        }
                    }
                } else {
                    println!("📊 No results found");
                }
            }
            Err(e) => {
                println!("⚠️  Query failed: {}", e);
            }
        }
    } else {
        println!("🔍 Showing database overview...");
        
        let query_request = json!({
            "limit": 10
        });
        
        match client.query_objects(query_request).await {
            Ok(result) => {
                if let Some(results) = result.get("results") {
                    if let Some(array) = results.as_array() {
                        println!("📊 Sample objects ({} shown):", array.len());
                        for (i, item) in array.iter().enumerate() {
                            if let Some(object) = item.get("object") {
                                let name = object.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                let kind = object.get("kind").and_then(|v| v.as_str()).unwrap_or("unknown");
                                println!("  {}. {} ({})", i + 1, name, kind);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Query failed: {}", e);
            }
        }
    }

    Ok(())
}
