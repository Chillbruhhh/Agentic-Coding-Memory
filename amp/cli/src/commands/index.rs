use crate::client::AmpClient;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;
use walkdir::WalkDir;
use uuid::Uuid;
use chrono::Utc;

pub async fn run_index(path: &str, exclude: &[String], client: &AmpClient) -> Result<()> {
    println!("🔍 Indexing directory: {}", path);
    
    // Check if AMP server is available
    if !client.health_check().await? {
        anyhow::bail!("AMP server is not available. Please start the server first.");
    }
    
    let root_path = Path::new(path);
    if !root_path.exists() {
        anyhow::bail!("Directory does not exist: {}", path);
    }
    
    // Create project root node first
    let project_id = create_project_node(root_path, client).await?;
    println!("📁 Created project node: {}", project_id);
    
    let mut total_files = 0;
    let mut processed_files = 0;
    let mut created_symbols = 0;
    let mut created_directories = 0;
    let mut errors = Vec::new();
    
    // Default exclude patterns
    let mut exclude_patterns = vec![
        ".git".to_string(),
        ".venv".to_string(),
        "venv".to_string(),
        "env".to_string(),
        ".env".to_string(),
        "node_modules".to_string(),
        "target".to_string(),
        "dist".to_string(),
        "build".to_string(),
        "__pycache__".to_string(),
        ".pytest_cache".to_string(),
        ".mypy_cache".to_string(),
        ".tox".to_string(),
        "*.pyc".to_string(),
        "*.pyo".to_string(),
        "*.log".to_string(),
        "*.tmp".to_string(),
        ".DS_Store".to_string(),
        "Thumbs.db".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
        "*.egg-info".to_string(),
        ".coverage".to_string(),
        "htmlcov".to_string(),
    ];
    exclude_patterns.extend_from_slice(exclude);
    
    println!("📋 Exclude patterns: {:?}", exclude_patterns);
    
    // Track created directories to avoid duplicates
    let mut created_dirs = std::collections::HashSet::new();
    
    // Walk directory and collect supported files
    let supported_extensions = vec!["py", "ts", "tsx", "js", "jsx"];
    let mut files_to_process = Vec::new();
    
    for entry in WalkDir::new(root_path).follow_links(false) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                // Skip if matches exclude patterns
                if should_exclude(path, &exclude_patterns) {
                    continue;
                }
                
                // Create directory nodes for all directories in the path
                if let Some(parent) = path.parent() {
                    if parent != root_path {
                        let relative_parent = parent.strip_prefix(root_path).unwrap_or(parent);
                        let dir_key = relative_parent.to_string_lossy().to_string();
                        
                        if !created_dirs.contains(&dir_key) && !dir_key.is_empty() {
                            match create_directory_node(parent, &project_id, client).await {
                                Ok(dir_id) => {
                                    created_directories += 1;
                                    created_dirs.insert(dir_key);
                                    println!("📂 Created directory node: {}", parent.display());
                                }
                                Err(e) => {
                                    errors.push(format!("Failed to create directory node for {}: {}", parent.display(), e));
                                }
                            }
                        }
                    }
                }
                
                // Check if it's a supported file type
                if let Some(extension) = path.extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if supported_extensions.contains(&ext_str) {
                            files_to_process.push(path.to_path_buf());
                        }
                    }
                }
                total_files += 1;
            }
            Err(e) => {
                errors.push(format!("Error walking directory: {}", e));
            }
        }
    }
    
    println!("📊 Found {} supported files out of {} total files", files_to_process.len(), total_files);
    println!("📂 Created {} directory nodes", created_directories);
    
    // Process each file and create hierarchical structure
    for file_path in files_to_process {
        match process_file_hierarchical(&file_path, &project_id, client).await {
            Ok(symbols_count) => {
                processed_files += 1;
                created_symbols += symbols_count;
                println!("✅ Processed {}: {} symbols", file_path.display(), symbols_count);
            }
            Err(e) => {
                errors.push(format!("Error processing {}: {}", file_path.display(), e));
            }
        }
    }
    
    // Print summary
    println!("\n🎉 Indexing complete!");
    println!("📊 Summary:");
    println!("   Project: 1 node");
    println!("   Directories: {} nodes", created_directories);
    println!("   Files processed: {}", processed_files);
    println!("   Code symbols: {}", created_symbols);
    println!("   Total nodes: {}", 1 + created_directories + processed_files + created_symbols);
    
    if !errors.is_empty() {
        println!("⚠️  Errors encountered:");
        for error in &errors {
            println!("   - {}", error);
        }
    }
    
    Ok(())
}

pub fn should_exclude(path: &Path, exclude_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    
    for pattern in exclude_patterns {
        // Simple pattern matching - check if path contains the pattern
        if pattern.starts_with('*') {
            // Handle wildcard patterns like *.log
            let suffix = &pattern[1..];
            if path_str.ends_with(suffix) {
                return true;
            }
        } else if path_str.contains(pattern) {
            return true;
        }
    }
    
    false
}

async fn process_file(file_path: &Path, client: &AmpClient) -> Result<usize> {
    println!("🔍 Processing file: {}", file_path.display());
    
    // Read file content for fallback
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to read file: {}", e));
        }
    };
    
    // Create a basic Symbol object for the file
    let symbol = create_file_symbol(file_path, &content)?;
    
    // Send to AMP server
    match client.create_object(symbol).await {
        Ok(_) => Ok(1), // Created 1 symbol
        Err(e) => Err(anyhow::anyhow!("Failed to create symbol: {}", e)),
    }
}

async fn create_project_node(root_path: &Path, client: &AmpClient) -> Result<String> {
    let now = Utc::now();
    let project_name = root_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
    
    let project_id = Uuid::new_v4().to_string();
    
    let project_symbol = json!({
        "id": project_id.clone(),
        "type": "Symbol",
        "tenant_id": "default",
        "project_id": "indexed-project",
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": {
            "source": "amp-cli-index",
            "confidence": 1.0,
            "method": "directory-scan"
        },
        "links": [],
        "name": project_name,
        "kind": "project",
        "path": root_path.to_string_lossy(),
        "language": "multi",
        "content_hash": format!("{:x}", md5::compute(project_name.as_bytes())),
        "signature": format!("project: {}", project_name),
        "documentation": format!("Project root: {}", root_path.display())
    });
    
    client.create_object(project_symbol).await?;
    
    // Small delay to ensure object is fully created
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(project_id)
}

async fn create_directory_node(dir_path: &Path, project_id: &str, client: &AmpClient) -> Result<String> {
    let now = Utc::now();
    let dir_name = dir_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("directory");
    
    let dir_id = Uuid::new_v4().to_string();
    
    let dir_symbol = json!({
        "id": dir_id.clone(),
        "type": "Symbol",
        "tenant_id": "default",
        "project_id": "indexed-project",
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": {
            "source": "amp-cli-index",
            "confidence": 1.0,
            "method": "directory-scan"
        },
        "links": [],
        "name": dir_name,
        "kind": "directory",
        "path": dir_path.to_string_lossy(),
        "language": "directory",
        "content_hash": format!("{:x}", md5::compute(dir_name.as_bytes())),
        "signature": format!("directory: {}", dir_name),
        "documentation": format!("Directory: {}", dir_path.display())
    });
    
    client.create_object(dir_symbol).await?;
    
    // Small delay to ensure object is fully created
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // Create relationship: project contains directory
    match client.create_relationship_direct(project_id, &dir_id, "defined_in").await {
        Ok(_) => println!("✅ Created relationship: project contains {}", dir_name),
        Err(e) => println!("⚠️  Failed to create relationship: {}", e),
    }
    
    Ok(dir_id)
}

async fn process_file_hierarchical(file_path: &Path, project_id: &str, client: &AmpClient) -> Result<usize> {
    println!("🔍 Processing file: {}", file_path.display());
    
    // First create a file node
    let file_id = create_file_node(file_path, project_id, client).await?;
    
    // Then try to parse symbols within the file
    match use_codebase_parser_hierarchical(file_path, &file_id, client).await {
        Ok(count) => {
            println!("✅ Codebase parser created {} symbols", count);
            Ok(count + 1) // +1 for the file node itself
        },
        Err(e) => {
            println!("⚠️  Codebase parser failed ({}), file node created", e);
            Ok(1) // Just the file node
        }
    }
}

async fn create_file_node(file_path: &Path, project_id: &str, client: &AmpClient) -> Result<String> {
    let now = Utc::now();
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        _ => "unknown",
    };
    
    let file_id = Uuid::new_v4().to_string();
    
    let file_symbol = json!({
        "id": file_id.clone(),
        "type": "Symbol",
        "tenant_id": "default",
        "project_id": "indexed-project",
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": {
            "source": "amp-cli-index",
            "confidence": 0.9,
            "method": "file-scan"
        },
        "links": [],
        "name": file_name,
        "kind": "file",
        "path": file_path.to_string_lossy(),
        "language": language,
        "content_hash": format!("{:x}", md5::compute(file_name.as_bytes())),
        "signature": format!("file: {}", file_name),
        "documentation": format!("File: {}", file_path.display())
    });
    
    client.create_object(file_symbol).await?;
    
    // Small delay to ensure object is fully created
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // Create relationship: project contains file
    match client.create_relationship_direct(project_id, &file_id, "defined_in").await {
        Ok(_) => println!("✅ Created relationship: project contains {}", file_name),
        Err(e) => println!("⚠️  Failed to create relationship: {}", e),
    }
    
    Ok(file_id)
}

async fn use_codebase_parser_hierarchical(file_path: &Path, file_id: &str, client: &AmpClient) -> Result<usize> {
    // Convert to absolute path to avoid server working directory issues
    let absolute_path = file_path.canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to get absolute path: {}", e))?;
    
    // Use the codebase parser endpoint to get detailed symbols
    let parse_request = serde_json::json!({
        "file_path": absolute_path.to_string_lossy(),
        "project_id": "indexed-project",
        "tenant_id": "default"
    });
    
    println!("🔍 Sending absolute path to parser: {}", absolute_path.display());
    
    let response = client.parse_file(parse_request).await?;
    
    // The response contains a file_log with symbols
    if let Some(file_log) = response.get("file_log") {
        if let Some(symbols) = file_log.get("symbols") {
            if let Some(symbols_array) = symbols.as_array() {
                println!("🔍 Found {} symbols in {}", symbols_array.len(), file_path.display());
                
                // Create AMP Symbol objects for each parsed symbol with file relationship
                let mut created_count = 0;
                for symbol_data in symbols_array {
                    if let Ok(amp_symbol) = create_amp_symbol_from_parsed_hierarchical(symbol_data, file_path, file_id) {
                        match client.create_object(amp_symbol.clone()).await {
                            Ok(_) => {
                                created_count += 1;
                                // Create relationship: file defines symbol
                                if let Some(symbol_id) = amp_symbol.get("id").and_then(|v| v.as_str()) {
                                    let symbol_name = amp_symbol.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                                    match client.create_relationship_direct(file_id, symbol_id, "defined_in").await {
                                        Ok(_) => println!("✅ Created relationship: {} defines {}", file_path.file_name().unwrap_or_default().to_string_lossy(), symbol_name),
                                        Err(e) => println!("⚠️  Failed to create relationship: {}", e),
                                    }
                                }
                            },
                            Err(e) => println!("⚠️  Failed to create symbol: {}", e),
                        }
                    }
                }
                
                return Ok(created_count);
            }
        }
    }
    
    // If no symbols found, create at least a file-level symbol
    Ok(1)
}

fn create_amp_symbol_from_parsed_hierarchical(symbol_data: &serde_json::Value, file_path: &Path, file_id: &str) -> Result<serde_json::Value> {
    let now = chrono::Utc::now();
    
    let name = symbol_data.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    // Fix: Use "symbol_type" instead of "kind" to match the parser output
    let kind = symbol_data.get("symbol_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    let line_start = symbol_data.get("start_line")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    // Create a signature from the symbol data
    let signature = format!("{}: {}", kind, name);
    
    // Determine language from file extension
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript", 
        Some("js") | Some("jsx") => "javascript",
        _ => "unknown",
    };
    
    let symbol = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "Symbol",
        "tenant_id": "default",
        "project_id": "indexed-project",
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": {
            "source": "amp-cli-codebase-parser",
            "confidence": 0.95,
            "method": "tree-sitter-parsing"
        },
        "links": [],
        "name": name,
        "kind": kind,
        "path": file_path.to_string_lossy(),
        "language": language,
        "content_hash": format!("{:x}", md5::compute(signature.as_bytes())),
        "signature": signature,
        "documentation": format!("{} {} at line {} in {}", kind, name, line_start + 1, file_path.display())
    });
    
    Ok(symbol)
}

async fn create_simple_file_symbol(file_path: &Path, client: &AmpClient) -> Result<usize> {
    // Read file content
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to read file: {}", e));
        }
    };
    
    // Create a basic Symbol object for the file
    let symbol = create_file_symbol(file_path, &content)?;
    
    // Send to AMP server
    match client.create_object(symbol).await {
        Ok(_) => Ok(1), // Created 1 symbol
        Err(e) => Err(anyhow::anyhow!("Failed to create symbol: {}", e)),
    }
}

pub fn create_file_symbol(file_path: &Path, content: &str) -> Result<Value> {
    let now = Utc::now();
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    // Determine language from extension
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        _ => "unknown",
    };
    
    // Create a simple content hash
    let content_hash = format!("{:x}", md5::compute(content.as_bytes()));
    
    let symbol = json!({
        "id": Uuid::new_v4().to_string(),
        "type": "Symbol",
        "tenant_id": "default",
        "project_id": "indexed-project",
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": {
            "source": "amp-cli-index",
            "confidence": 0.9,
            "method": "file-scan"
        },
        "links": [],
        "name": file_name,
        "kind": "file",
        "path": file_path.to_string_lossy(),
        "language": language,
        "content_hash": content_hash,
        "signature": format!("file: {}", file_name),
        "documentation": format!("File: {} ({} lines)", file_path.display(), content.lines().count())
    });
    
    Ok(symbol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_should_exclude() {
        let exclude_patterns = vec![
            ".git".to_string(),
            "target".to_string(),
            "*.log".to_string(),
        ];
        
        assert!(should_exclude(&PathBuf::from(".git/config"), &exclude_patterns));
        assert!(should_exclude(&PathBuf::from("target/debug/app"), &exclude_patterns));
        assert!(should_exclude(&PathBuf::from("app.log"), &exclude_patterns));
        assert!(!should_exclude(&PathBuf::from("src/main.rs"), &exclude_patterns));
    }
    
    #[test]
    fn test_create_file_symbol() {
        let path = PathBuf::from("src/main.py");
        let content = "def hello():\n    print('Hello, world!')";
        
        let symbol = create_file_symbol(&path, content).unwrap();
        
        assert_eq!(symbol["type"], "Symbol");
        assert_eq!(symbol["name"], "main.py");
        assert_eq!(symbol["language"], "python");
        assert_eq!(symbol["kind"], "file");
    }
}
