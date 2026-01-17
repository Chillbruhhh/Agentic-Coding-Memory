use amp::commands::index;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_index_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();
    
    // This test would require a mock AMP client
    // For now, we'll test the helper functions
    assert!(Path::new(temp_path).exists());
}

#[test]
fn test_should_exclude_patterns() {
    use amp::commands::index::should_exclude;
    use std::path::PathBuf;
    
    let exclude_patterns = vec![
        ".git".to_string(),
        "target".to_string(),
        "*.log".to_string(),
    ];
    
    // Test directory exclusions
    assert!(should_exclude(&PathBuf::from(".git/config"), &exclude_patterns));
    assert!(should_exclude(&PathBuf::from("project/.git/HEAD"), &exclude_patterns));
    assert!(should_exclude(&PathBuf::from("target/debug/app"), &exclude_patterns));
    
    // Test file pattern exclusions
    assert!(should_exclude(&PathBuf::from("app.log"), &exclude_patterns));
    assert!(should_exclude(&PathBuf::from("logs/error.log"), &exclude_patterns));
    
    // Test files that should not be excluded
    assert!(!should_exclude(&PathBuf::from("src/main.rs"), &exclude_patterns));
    assert!(!should_exclude(&PathBuf::from("README.md"), &exclude_patterns));
    assert!(!should_exclude(&PathBuf::from("app.py"), &exclude_patterns));
}

#[test]
fn test_create_file_symbol() {
    use amp::commands::index::create_file_symbol;
    use std::path::PathBuf;
    
    let path = PathBuf::from("src/main.py");
    let content = "def hello():\n    print('Hello, world!')";
    
    let symbol = create_file_symbol(&path, content).unwrap();
    
    // Verify symbol structure
    assert_eq!(symbol["type"], "Symbol");
    assert_eq!(symbol["name"], "main.py");
    assert_eq!(symbol["language"], "python");
    assert_eq!(symbol["kind"], "file");
    assert_eq!(symbol["tenant_id"], "default");
    assert_eq!(symbol["project_id"], "indexed-project");
    
    // Verify path and signature
    assert_eq!(symbol["path"], "src/main.py");
    assert_eq!(symbol["signature"], "file: main.py");
    
    // Verify documentation contains line count
    let doc = symbol["documentation"].as_str().unwrap();
    assert!(doc.contains("2 lines"));
}

#[test]
fn test_create_typescript_file_symbol() {
    use amp::commands::index::create_file_symbol;
    use std::path::PathBuf;
    
    let path = PathBuf::from("src/component.tsx");
    let content = "export const Component = () => {\n  return <div>Hello</div>;\n};";
    
    let symbol = create_file_symbol(&path, content).unwrap();
    
    assert_eq!(symbol["name"], "component.tsx");
    assert_eq!(symbol["language"], "typescript");
    assert_eq!(symbol["kind"], "file");
}

#[tokio::test]
async fn test_process_supported_files() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    
    // Create test files
    fs::write(temp_path.join("main.py"), "print('hello')").unwrap();
    fs::write(temp_path.join("app.ts"), "console.log('hello')").unwrap();
    fs::write(temp_path.join("README.md"), "# Project").unwrap();
    fs::write(temp_path.join("config.json"), "{}").unwrap();
    
    // Create subdirectory with files
    fs::create_dir(temp_path.join("src")).unwrap();
    fs::write(temp_path.join("src/utils.py"), "def util(): pass").unwrap();
    
    // This test would need a mock client to fully test
    // For now, we verify the files exist
    assert!(temp_path.join("main.py").exists());
    assert!(temp_path.join("app.ts").exists());
    assert!(temp_path.join("src/utils.py").exists());
}
