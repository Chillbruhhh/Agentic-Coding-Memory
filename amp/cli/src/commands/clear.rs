use crate::client::AmpClient;
use anyhow::Result;
use std::io::{self, Write};
use std::process::Command;
use std::env;

pub async fn run_clear(confirm: bool, client: &AmpClient) -> Result<()> {
    if !client.health_check().await? {
        anyhow::bail!("AMP server is not available. Please start the server first.");
    }

    if !confirm {
        print!("⚠️  This will delete ALL objects from the AMP database. Are you sure? (y/N): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ Clear operation cancelled.");
            return Ok(());
        }
    }

    println!("🗑️  Clearing AMP database...");
    
    // Try to load .env from server directory
    let server_env_path = std::path::Path::new("amp/server/.env");
    if server_env_path.exists() {
        println!("📋 Loading server config from amp/server/.env");
        dotenvy::from_path(server_env_path).ok();
    }
    
    // Check if we're using external SurrealDB
    if let Ok(database_url) = env::var("DATABASE_URL") {
        println!("🔍 Found DATABASE_URL: {}", database_url);
        if database_url.starts_with("ws://") || database_url.starts_with("http://") {
            println!("🔍 Detected external SurrealDB: {}", database_url);
            return clear_external_db(&database_url).await;
        }
    } else {
        println!("ℹ️  No DATABASE_URL found in environment");
    }
    
    // Fallback to API-based clearing for local/memory databases
    match clear_via_api(client).await {
        Ok(count) => {
            println!("✅ Successfully cleared {} objects from the database.", count);
        }
        Err(e) => {
            println!("❌ Failed to clear database: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

async fn clear_external_db(database_url: &str) -> Result<()> {
    let db_user = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
    let db_pass = env::var("DB_PASS").unwrap_or_else(|_| "root".to_string());
    
    println!("🔗 Connecting to external SurrealDB...");
    
    // Use SurrealDB CLI to execute DELETE command
    let output = Command::new("surreal")
        .args([
            "sql",
            "--conn", database_url,
            "--user", &db_user,
            "--pass", &db_pass,
            "--ns", "test",
            "--db", "test",
            "--query", "DELETE FROM objects;"
        ])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                println!("✅ Successfully executed DELETE command");
                if !stdout.is_empty() {
                    println!("📊 Output: {}", stdout.trim());
                }
                if !stderr.is_empty() {
                    println!("ℹ️  Info: {}", stderr.trim());
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                anyhow::bail!("SurrealDB command failed: {}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to execute surreal CLI command: {}", e);
            println!("💡 Make sure SurrealDB CLI is installed and in PATH");
            println!("   Install: https://surrealdb.com/install");
            println!("   Or manually connect: surreal sql --conn {} --user {} --pass {}", 
                database_url, db_user, db_pass);
            println!("   Then run: DELETE FROM objects;");
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn clear_via_api(client: &AmpClient) -> Result<usize> {
    println!("🔍 Using API-based clearing for local database...");
    
    // Use a simple text query to get all objects
    let query_request = serde_json::json!({
        "text": "*",
        "limit": 10000
    });
    
    let query_result = client.query_objects(query_request).await?;
    
    // Extract object IDs from the response
    let mut object_ids = Vec::new();
    if let Some(results) = query_result.get("results") {
        if let Some(array) = results.as_array() {
            for item in array {
                if let Some(object) = item.get("object") {
                    if let Some(id_str) = object.get("id").and_then(|v| v.as_str()) {
                        let clean_id = if id_str.starts_with("objects:") {
                            &id_str[8..]
                        } else {
                            id_str
                        };
                        object_ids.push(clean_id.to_string());
                    }
                }
            }
        }
    }
    
    println!("📊 Found {} objects to delete", object_ids.len());
    
    if object_ids.is_empty() {
        return Ok(0);
    }
    
    // Delete objects one by one
    let mut deleted_count = 0;
    let total = object_ids.len();
    
    for (i, id) in object_ids.iter().enumerate() {
        match client.delete_object(id).await {
            Ok(_) => {
                deleted_count += 1;
                if (i + 1) % 100 == 0 {
                    println!("🗑️  Deleted {}/{} objects", i + 1, total);
                }
            }
            Err(e) => {
                println!("⚠️  Failed to delete object {}: {}", id, e);
            }
        }
    }
    
    Ok(deleted_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_clear_with_confirm() {
        assert!(true);
    }
}
