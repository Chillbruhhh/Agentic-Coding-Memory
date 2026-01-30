use crate::client::AmpClient;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ignore::WalkBuilder;
use uuid::Uuid;
use chrono::Utc;
use toml;
use sha2::{Digest, Sha256};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use std::sync::Mutex;
use std::io::IsTerminal;

use crate::commands::index_ui::{start_index_ui, IndexUiHandle, IndexUiState};

static INDEX_QUIET: AtomicBool = AtomicBool::new(false);
const MAX_AI_LOG_CONTENT_CHARS: usize = 20000;
const AI_LOG_CONTENT_HEAD_CHARS: usize = 12000;
const AI_LOG_CONTENT_TAIL_CHARS: usize = 6000;

macro_rules! index_log {
    ($($arg:tt)*) => {
        if !INDEX_QUIET.load(Ordering::Relaxed) {
            println!($($arg)*);
        }
    };
}

fn with_ui_state(state: &Arc<Mutex<IndexUiState>>, use_tui: bool, f: impl FnOnce(&mut IndexUiState)) {
    if !use_tui {
        return;
    }
    if let Ok(mut guard) = state.lock() {
        f(&mut guard);
    }
}

fn check_cancel(cancel_flag: &AtomicBool) -> Result<()> {
    if cancel_flag.load(Ordering::Relaxed) {
        anyhow::bail!("Indexing cancelled by user.");
    }
    Ok(())
}

struct UiGuard {
    handle: Option<IndexUiHandle>,
}

impl Drop for UiGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.stop();
        }
    }
}

pub async fn run_index(path: &str, exclude: &[String], init_root: bool, client: &AmpClient) -> Result<()> {
    let use_tui = std::io::stdout().is_terminal();
    let cancel_flag = Arc::new(AtomicBool::new(false));
    if use_tui {
        INDEX_QUIET.store(true, Ordering::Relaxed);
        client.set_quiet(true);
    }
    let ui_state = Arc::new(Mutex::new(IndexUiState::default()));
    let mut ui_guard = UiGuard { handle: None };

    if use_tui {
        with_ui_state(&ui_state, use_tui, |state| {
            state.phase = "Startup".to_string();
            state.status_message = "Initializing AMP index".to_string();
        });
        ui_guard.handle = Some(start_index_ui(Arc::clone(&ui_state), Arc::clone(&cancel_flag))?);
    } else {
        index_log!("Indexing directory: {}", path);
    }
    
    // Check if AMP server is available
    with_ui_state(&ui_state, use_tui, |state| {
        state.phase = "Health check".to_string();
        state.status_message = "Checking AMP server".to_string();
    });
    if !client.health_check().await? {
        anyhow::bail!("AMP server is not available. Please start the server first.");
    }
    check_cancel(&cancel_flag)?;
    
    let root_path_input = Path::new(path);
    let root_path = root_path_input
        .canonicalize()
        .unwrap_or_else(|_| root_path_input.to_path_buf());
    if !root_path.exists() {
        anyhow::bail!("Directory does not exist: {}", path);
    }

    if init_root {
        maybe_init_amp_root(&root_path)?;
    }
    
    // Create project root node first
    let (project_object_id, project_id) = create_project_node(&root_path, client).await?;
    if !use_tui {
        index_log!("Created project node: {} (id: {})", project_id, project_object_id);
    }
    with_ui_state(&ui_state, use_tui, |state| {
        state.phase = "Scanning".to_string();
        state.status_message = format!("Project: {}", project_id);
    });

    let mut warnings: Vec<String> = Vec::new();

    let (worker_count, index_ai_enabled, index_respect_gitignore) = match get_index_settings(client).await {
        Ok(settings) => (settings.worker_count, settings.ai_enabled, settings.respect_gitignore),
        Err(e) => {
            warnings.push(format!("Failed to load index settings: {}", e));
            with_ui_state(&ui_state, use_tui, |state| state.warnings += 1);
            (4, true, true)
        }
    };
    let worker_count = worker_count.clamp(1, 32);
    if !use_tui {
        index_log!("Index workers: {}", worker_count);
    }

    if index_ai_enabled {
        with_ui_state(&ui_state, use_tui, |state| {
            state.phase = "Project log".to_string();
            state.status_message = "Generating project AI log".to_string();
        });
        if let Err(e) = create_project_ai_log_and_link(&root_path, &project_object_id, &project_id, client).await {
            warnings.push(format!("Project AI log failed: {}", e));
            with_ui_state(&ui_state, use_tui, |state| state.warnings += 1);
        }
    }
    
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
        "lib".to_string(),
        "Lib".to_string(),
        "libs".to_string(),
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
        "amp-core".to_string(),
        "*.egg-info".to_string(),
        ".coverage".to_string(),
        "htmlcov".to_string(),
    ];
    exclude_patterns.extend_from_slice(exclude);
    
    if !use_tui {
        index_log!("Exclude patterns: {:?}", exclude_patterns);
    }
    
    // Track created directories to avoid duplicates
    let mut created_dir_keys: HashSet<String> = HashSet::new();
    let mut dir_index: HashMap<String, String> = HashMap::new();
    let mut created_dir_nodes: Vec<(PathBuf, String)> = Vec::new();
    
    // Walk directory and collect supported files
    let mut files_to_process = Vec::new();
    let mut skipped_files = Vec::new();
    
    let mut walker = WalkBuilder::new(&root_path);
    walker.follow_links(false).hidden(false);
    if index_respect_gitignore {
        walker.git_ignore(true).git_exclude(false).git_global(false);
    } else {
        walker.git_ignore(false).git_exclude(false).git_global(false);
    }

    for entry in walker.build() {
        check_cancel(&cancel_flag)?;
        match entry {
            Ok(entry) => {
                let path = entry.path();
                
                // Skip if matches exclude patterns
                if should_exclude(path, &exclude_patterns) {
                    skipped_files.push(format!("Excluded: {}", path.display()));
                    continue;
                }
                
                // Ensure directory chain exists for this entry
                if let Some(dir_path) = if path.is_dir() { Some(path) } else { path.parent() } {
                    if dir_path != root_path {
                        if let Err(e) = ensure_directory_chain(
                            dir_path,
                            &root_path,
                            &project_object_id,
                            &project_id,
                            client,
                            &mut created_dir_keys,
                            &mut dir_index,
                            &mut created_dir_nodes,
                            &mut created_directories,
                            use_tui,
                        )
                        .await
                        {
                            errors.push(format!("Failed to create directory nodes for {}: {}", dir_path.display(), e));
                            with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
                        }
                    }
                }
                
                // Check if it's a file and if it's a text file
                if path.is_file() {
                    // Only process text files, skip binary files
                    if is_text_file(path) {
                        files_to_process.push(path.to_path_buf());
                    } else {
                        skipped_files.push(format!("Binary file: {}", path.display()));
                    }
                }
                total_files += 1;
                with_ui_state(&ui_state, use_tui, |state| {
                    state.total_files = total_files;
                    state.supported_files = files_to_process.len();
                    state.created_directories = created_directories;
                    state.current_path = path.display().to_string();
                });
            }
            Err(e) => {
                errors.push(format!("Error walking directory: {}", e));
                with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
            }
        }
    }
    
    with_ui_state(&ui_state, use_tui, |state| {
        state.phase = "Indexing".to_string();
        state.supported_files = files_to_process.len();
        state.total_files = total_files;
        state.created_directories = created_directories;
        state.status_message = "Preparing file nodes".to_string();
    });
    check_cancel(&cancel_flag)?;
    if !use_tui {
        index_log!("\nFound {} supported files out of {} total files", files_to_process.len(), total_files);
        index_log!("Created {} directory nodes", created_directories);
    }
    
    // Show first 10 skipped files for debugging
    if !skipped_files.is_empty() && !use_tui {
        index_log!("\nSkipped {} files (showing first 10):", skipped_files.len());
        for skip in skipped_files.iter().take(10) {
            index_log!("   {}", skip);
        }
        if index_respect_gitignore {
            index_log!("Note: .gitignore entries are filtered before walking and are not counted above.");
        }
    }

    if index_ai_enabled && !created_dir_nodes.is_empty() {
        with_ui_state(&ui_state, use_tui, |state| {
            state.phase = "Directory logs".to_string();
            state.status_message = format!("Generating {} directory logs", created_dir_nodes.len());
        });
        if !use_tui {
            index_log!("Generating directory AI logs ({} entries)...", created_dir_nodes.len());
        }
        let semaphore = Arc::new(Semaphore::new(worker_count));
        let mut join_set = JoinSet::new();
        for (dir_path, dir_id) in created_dir_nodes {
            check_cancel(&cancel_flag)?;
            let permit = semaphore.clone().acquire_owned().await?;
            let client = client.clone();
            let project_id = project_id.clone();
            join_set.spawn(async move {
                let _permit = permit;
                create_directory_ai_log_and_link(&dir_path, &dir_id, &project_id, &client).await?;
                Ok::<(), anyhow::Error>(())
            });
        }
        while let Some(result) = join_set.join_next().await {
            if cancel_flag.load(Ordering::Relaxed) {
                join_set.abort_all();
                anyhow::bail!("Indexing cancelled by user.");
            }
            if let Ok(Err(e)) = result {
                warnings.push(format!("Directory AI log failed: {}", e));
                with_ui_state(&ui_state, use_tui, |state| state.warnings += 1);
            }
        }
    }


    // Create file nodes first so dependency edges can resolve reliably.
    with_ui_state(&ui_state, use_tui, |state| {
        state.phase = "File nodes".to_string();
        state.status_message = format!("Creating {} file nodes", files_to_process.len());
    });
    let semaphore = Arc::new(Semaphore::new(worker_count));
    let mut join_set = JoinSet::new();
    let dir_index = Arc::new(dir_index);
    for file_path in &files_to_process {
        check_cancel(&cancel_flag)?;
        let permit = semaphore.clone().acquire_owned().await?;
        let client = client.clone();
        let project_object_id = project_object_id.clone();
        let project_id = project_id.clone();
        let file_path = file_path.clone();
        let dir_index = Arc::clone(&dir_index);
        join_set.spawn(async move {
            let _permit = permit;
            let parent_dir_id = file_path
                .parent()
                .and_then(path_key)
                .and_then(|key| dir_index.get(&key).cloned());
            let file_id = create_file_node(&file_path, &project_object_id, &project_id, parent_dir_id.as_deref(), &client).await?;
            Ok::<(PathBuf, String), anyhow::Error>((file_path, file_id))
        });
    }

    let mut file_index: HashMap<String, String> = HashMap::new();
    while let Some(result) = join_set.join_next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            join_set.abort_all();
            anyhow::bail!("Indexing cancelled by user.");
        }
        match result {
            Ok(Ok((file_path, file_id))) => {
                if let Some(key) = path_key(&file_path) {
                    file_index.insert(key, file_id);
                }
            }
            Ok(Err(e)) => {
                errors.push(format!("Failed to create file node: {}", e));
                with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
            }
            Err(e) => {
                errors.push(format!("Failed to join file creation task: {}", e));
                with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
            }
        }
    }

    let file_index = Arc::new(file_index);
    let mut join_set = JoinSet::new();
    with_ui_state(&ui_state, use_tui, |state| {
        state.phase = "Parsing".to_string();
        state.status_message = "Processing files".to_string();
    });
    for file_path in files_to_process {
        check_cancel(&cancel_flag)?;
        let key = match path_key(&file_path) {
            Some(key) => key,
            None => {
                errors.push(format!("Failed to normalize path: {}", file_path.display()));
                with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
                continue;
            }
        };
        let file_id = match file_index.get(&key) {
            Some(id) => id.clone(),
            None => {
                errors.push(format!("Missing file node for {}", file_path.display()));
                with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
                continue;
            }
        };

        let permit = semaphore.clone().acquire_owned().await?;
        let client = client.clone();
        let project_id = project_id.clone();
        let root_path = root_path.to_path_buf();
        let file_index = Arc::clone(&file_index);
        join_set.spawn(async move {
            let _permit = permit;
            let symbols_count = process_file_hierarchical_with_id(
                &file_path,
                &file_id,
                &project_id,
                &root_path,
                file_index.as_ref(),
                index_ai_enabled,
                &client,
            )
            .await?;
            Ok::<(PathBuf, usize), anyhow::Error>((file_path, symbols_count))
        });
    }

    while let Some(result) = join_set.join_next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            join_set.abort_all();
            anyhow::bail!("Indexing cancelled by user.");
        }
        match result {
            Ok(Ok((file_path, symbols_count))) => {
                processed_files += 1;
                created_symbols += symbols_count;
                if !use_tui {
                    index_log!("Processed {}: {} symbols", file_path.display(), symbols_count);
                }
                with_ui_state(&ui_state, use_tui, |state| {
                    state.processed_files = processed_files;
                    state.created_symbols = created_symbols;
                    state.current_path = file_path.display().to_string();
                    state.status_message = "Processing files".to_string();
                });
            }
            Ok(Err(e)) => {
                errors.push(format!("Error processing file: {}", e));
                with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
            }
            Err(e) => {
                errors.push(format!("Failed to join file processing task: {}", e));
                with_ui_state(&ui_state, use_tui, |state| state.errors += 1);
            }
        }
    }
    
    with_ui_state(&ui_state, use_tui, |state| {
        state.phase = "Complete".to_string();
        state.status_message = "Indexing complete".to_string();
        state.processed_files = processed_files;
        state.created_symbols = created_symbols;
        state.created_directories = created_directories;
        state.done = true;
    });

    if !use_tui {
        // Print summary
        index_log!("\nIndexing complete!");
        index_log!("Summary:");
        index_log!("   Project: 1 node");
        index_log!("   Directories: {} nodes", created_directories);
        index_log!("   Files processed: {}", processed_files);
        index_log!("   Code symbols: {}", created_symbols);
        index_log!("   Total nodes: {}", 1 + created_directories + processed_files + created_symbols);

        // Show project name detection info
        index_log!("\nProject Name Detection:");
        if let Some(detected_name) = detect_project_name(&root_path) {
            index_log!("   Detected from config file: {}", detected_name);
        } else {
            let fallback_name = root_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project");
            index_log!("   No config file found, using directory name: {}", fallback_name);
        }
    }
    
    // Show skipped files for debugging
    if !skipped_files.is_empty() && !use_tui {
        index_log!("\nSkipped {} files:", skipped_files.len());
        for skip in skipped_files.iter() {
            index_log!("   {}", skip);
        }
        if index_respect_gitignore {
            index_log!("Note: .gitignore entries are filtered before walking and are not counted above.");
        }
    }
    
    if !errors.is_empty() && !use_tui {
        index_log!("\nErrors encountered ({}):", errors.len());
        for error in &errors {
            index_log!("   - {}", error);
        }
    }

    if !warnings.is_empty() && !use_tui {
        index_log!("\nWarnings ({}):", warnings.len());
        for warning in &warnings {
            index_log!("   - {}", warning);
        }
    }

    if use_tui {
        if let Some(handle) = ui_guard.handle.take() {
            handle.wait_for_exit()?;
        }
    }
    
    Ok(())
}

async fn ensure_directory_chain(
    dir_path: &Path,
    root_path: &Path,
    project_object_id: &str,
    project_id: &str,
    client: &AmpClient,
    created_dir_keys: &mut HashSet<String>,
    dir_index: &mut HashMap<String, String>,
    created_dir_nodes: &mut Vec<(PathBuf, String)>,
    created_directories: &mut usize,
    use_tui: bool,
) -> Result<()> {
    let relative = dir_path.strip_prefix(root_path).unwrap_or(dir_path);
    if relative.as_os_str().is_empty() {
        return Ok(());
    }

    let mut current = root_path.to_path_buf();
    let mut parent_id: Option<String> = None;

    for component in relative.components() {
        current.push(component.as_os_str());
        let key = match path_key(&current) {
            Some(key) => key,
            None => continue,
        };

        if !created_dir_keys.contains(&key) {
            let dir_id = create_directory_node(&current, project_object_id, project_id, client).await?;
            *created_directories += 1;
            created_dir_keys.insert(key.clone());
            dir_index.insert(key.clone(), dir_id.clone());
            created_dir_nodes.push((current.clone(), dir_id.clone()));
            if !use_tui {
                index_log!("Created directory node: {}", current.display());
            }

            if let Some(parent) = parent_id.as_ref() {
                let _ = client.create_relationship_direct(parent, &dir_id, "defined_in").await;
                let _ = client.create_relationship_direct(&dir_id, parent, "defined_in").await;
            }
        }

        parent_id = dir_index.get(&key).cloned();
    }

    Ok(())
}

struct IndexSettings {
    worker_count: usize,
    ai_enabled: bool,
    respect_gitignore: bool,
}

async fn get_index_settings(client: &AmpClient) -> Result<IndexSettings> {
    let settings = client.get_settings().await?;
    let workers = settings
        .get("indexWorkers")
        .and_then(|v| v.as_u64())
        .unwrap_or(4) as usize;
    let ai_enabled = settings
        .get("indexProvider")
        .and_then(|v| v.as_str())
        .map(|value| value != "none")
        .unwrap_or(true);
    let respect_gitignore = settings
        .get("indexRespectGitignore")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Ok(IndexSettings {
        worker_count: workers,
        ai_enabled,
        respect_gitignore,
    })
}


pub fn should_exclude(path: &Path, exclude_patterns: &[String]) -> bool {
    for pattern in exclude_patterns {
        // Handle wildcard patterns like *.log or *.egg-info
        if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            // Check if any path component ends with this suffix
            for component in path.components() {
                if let Some(comp_str) = component.as_os_str().to_str() {
                    if comp_str.ends_with(suffix) {
                        return true;
                    }
                }
            }
        } else {
            // Check if pattern matches any path component exactly
            for component in path.components() {
                if let Some(comp_str) = component.as_os_str().to_str() {
                    if comp_str == pattern {
                        return true;
                    }
                }
            }
        }
    }
    
    false
}

fn is_text_file(path: &Path) -> bool {
    // Check by extension first
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let text_extensions = [
            "txt", "md", "json", "yaml", "yml", "toml", "xml", "html", "css", "scss",
            "js", "jsx", "ts", "tsx", "py", "rs", "go", "java", "c", "cpp", "h", "hpp",
            "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd",
            "sql", "graphql", "proto", "thrift",
            "env", "gitignore", "dockerignore", "editorconfig",
            "lock", "sum", "mod",
        ];
        
        if text_extensions.contains(&ext.to_lowercase().as_str()) {
            return true;
        }
        
        // Skip known binary extensions
        let binary_extensions = [
            "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp",
            "mp3", "mp4", "avi", "mov", "wmv", "flv", "webm",
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
            "zip", "tar", "gz", "bz2", "7z", "rar",
            "exe", "dll", "so", "dylib", "bin",
            "wasm", "class", "jar", "war",
            "ttf", "otf", "woff", "woff2", "eot",
        ];
        
        if binary_extensions.contains(&ext.to_lowercase().as_str()) {
            return false;
        }
    }
    
    // For files without extension or unknown extensions, try reading first bytes
    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::Read;
        let mut buffer = [0u8; 512];
        if let Ok(n) = file.read(&mut buffer) {
            // Check for null bytes (strong indicator of binary)
            if buffer[..n].contains(&0) {
                return false;
            }
            // Check if valid UTF-8
            return std::str::from_utf8(&buffer[..n]).is_ok();
        }
    }
    
    false
}

#[allow(dead_code)]
async fn process_file(file_path: &Path, client: &AmpClient) -> Result<usize> {
    index_log!(" Processing file: {}", file_path.display());
    
    // Read file content for fallback
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to read file: {}", e));
        }
    };
    
    // Create a basic Symbol object for the file
    let symbol = create_file_symbol(file_path, &content, "default-project")?;
    
    // Send to AMP server
    match client.create_object(symbol).await {
        Ok(_) => Ok(1), // Created 1 symbol
        Err(e) => Err(anyhow::anyhow!("Failed to create symbol: {}", e)),
    }
}

fn detect_project_name(root_path: &Path) -> Option<String> {
    // Configuration files to check in priority order
    let config_files: Vec<(&str, fn(&Path) -> Option<String>)> = vec![
        ("package.json", extract_name_from_package_json as fn(&Path) -> Option<String>),
        ("Cargo.toml", extract_name_from_cargo_toml as fn(&Path) -> Option<String>),
        ("pyproject.toml", extract_name_from_pyproject_toml as fn(&Path) -> Option<String>),
        ("composer.json", extract_name_from_composer_json as fn(&Path) -> Option<String>),
    ];
    
    for (filename, extractor) in config_files {
        let config_path = root_path.join(filename);
        if config_path.exists() {
            if let Some(name) = extractor(&config_path) {
                return Some(name);
            }
        }
    }
    
    None
}

fn extract_name_from_package_json(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let json_value: serde_json::Value = serde_json::from_str(&content).ok()?;
    json_value.get("name")?.as_str().map(|s| s.to_string())
}

fn extract_name_from_cargo_toml(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let toml_value: toml::Value = toml::from_str(&content).ok()?;
    toml_value.get("package")?.get("name")?.as_str().map(|s| s.to_string())
}

fn extract_name_from_pyproject_toml(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let toml_value: toml::Value = toml::from_str(&content).ok()?;
    toml_value.get("project")?.get("name")?.as_str().map(|s| s.to_string())
}

fn extract_name_from_composer_json(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let json_value: serde_json::Value = serde_json::from_str(&content).ok()?;
    json_value.get("name")?.as_str().map(|s| s.to_string())
}

async fn create_project_node(root_path: &Path, client: &AmpClient) -> Result<(String, String)> {
    let now = Utc::now();
    let project_name = detect_project_name(root_path)
        .unwrap_or_else(|| {
            root_path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "project".to_string())
        });
    
    if detect_project_name(root_path).is_some() {
        index_log!(" Using project name from configuration: {}", project_name);
    } else {
        index_log!(" Using directory name as project name: {}", project_name);
    }
    
    // Use project name as the project_id (sanitized)
    let project_id = project_name.to_lowercase().replace(" ", "-");
    let object_id = Uuid::new_v4().to_string();
    
    let project_symbol = json!({
        "id": object_id.clone(),
        "type": "symbol",
        "tenant_id": "default",
        "project_id": project_id.clone(),
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
    
    Ok((object_id, project_id))
}

async fn create_project_ai_log_and_link(
    root_path: &Path,
    project_object_id: &str,
    project_id: &str,
    client: &AmpClient,
) -> Result<()> {
    let project_log = create_directory_log_ai(root_path, project_object_id, project_id, client).await?;
    let log_id = project_log.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
    client.create_object(project_log).await?;

    if let Some(log_id) = log_id {
        let _ = client.create_relationship_direct(project_object_id, &log_id, "defined_in").await;
        let _ = client.create_relationship_direct(&log_id, project_object_id, "defined_in").await;
    }

    Ok(())
}

fn maybe_init_amp_root(root_path: &Path) -> Result<()> {
    let git_dir = root_path.join(".git");
    let amp_root = root_path.join(".amp-root");
    if amp_root.exists() || git_dir.exists() {
        return Ok(());
    }
    std::fs::write(&amp_root, b"")?;
    index_log!("Created .amp-root in {}", root_path.display());
    Ok(())
}


async fn create_directory_node(dir_path: &Path, project_object_id: &str, project_id: &str, client: &AmpClient) -> Result<String> {
    let now = Utc::now();
    let dir_name = dir_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("directory");
    
    let dir_id = Uuid::new_v4().to_string();
    
    let dir_symbol = json!({
        "id": dir_id.clone(),
        "type": "symbol",
        "tenant_id": "default",
        "project_id": project_id,
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
    match client.create_relationship_direct(project_object_id, &dir_id, "defined_in").await {
        Ok(_) => index_log!(" Created relationship: project contains {}", dir_name),
        Err(e) => index_log!("  Failed to create relationship: {}", e),
    }

    // Symmetric relationship for traversal convenience
    match client.create_relationship_direct(&dir_id, project_object_id, "defined_in").await {
        Ok(_) => {},
        Err(e) => index_log!("  Failed to create reverse relationship: {}", e),
    }
    Ok(dir_id)
}

async fn create_directory_ai_log_and_link(
    dir_path: &Path,
    dir_id: &str,
    project_id: &str,
    client: &AmpClient,
) -> Result<()> {
    let dir_log = create_directory_log_ai(dir_path, dir_id, project_id, client).await?;
    let log_id = dir_log.get("id").and_then(|v| v.as_str()).map(|v| v.to_string());
    if let Err(e) = client.create_object(dir_log).await {
        anyhow::bail!("Failed to create directory file log: {}", e);
    }
    if let Some(log_id) = log_id {
        let _ = client.create_relationship_direct(dir_id, &log_id, "defined_in").await;
        let _ = client.create_relationship_direct(&log_id, dir_id, "defined_in").await;
    }
    Ok(())
}


#[allow(dead_code)]
async fn process_file_hierarchical(
    file_path: &Path,
    project_object_id: &str,
    project_id: &str,
    root_path: &Path,
    file_index: &mut HashMap<String, String>,
    index_ai_enabled: bool,
    client: &AmpClient
) -> Result<usize> {
    index_log!("Processing file: {}", file_path.display());
    
    // Create file node first
    let file_id = create_file_node(file_path, project_object_id, project_id, None, client).await?;
    if let Some(key) = path_key(file_path) {
        file_index.insert(key, file_id.clone());
    }

    process_file_hierarchical_with_id(
        file_path,
        &file_id,
        project_id,
        root_path,
        file_index,
        index_ai_enabled,
        client,
    )
    .await
}

async fn process_file_hierarchical_with_id(
    file_path: &Path,
    file_id: &str,
    project_id: &str,
    root_path: &Path,
    file_index: &HashMap<String, String>,
    index_ai_enabled: bool,
    client: &AmpClient,
) -> Result<usize> {
    // Parse and create symbols with relationships
    let (symbol_count, dependency_paths, symbol_names) = match use_codebase_parser_hierarchical(file_path, file_id, project_id, client).await {
        Ok((count, deps, names)) => {
            index_log!("Codebase parser created {} symbols", count);
            (count, deps, names)
        }
        Err(e) => {
            index_log!("Codebase parser failed: {}", e);
            (0, Vec::new(), Vec::new())
        }
    };

    // Create FileChunks and FileLog in batch (for embeddings)
    let mut batch = Vec::new();
    let chunks = create_file_chunks_objects(file_path, file_id, project_id)?;
    if chunks.len() > 1 {
        index_log!("Created {} chunks", chunks.len());
    }
    batch.extend(chunks);
    
    let file_log = if index_ai_enabled {
        create_file_log_object_ai(file_path, file_id, project_id, &symbol_names, &dependency_paths, client).await?
    } else {
        create_file_log_object(file_path, file_id, project_id, &[])?
    };
    batch.push(file_log);
    
    let mut file_artifact_ids: Vec<String> = Vec::new();
    for obj in &batch {
        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
            file_artifact_ids.push(id.to_string());
        }
    }

    if !batch.is_empty() {
        match client.batch_create_objects(batch).await {
            Ok(response) => {
                if let Some(summary) = response.get("summary") {
                    let succeeded = summary.get("succeeded").and_then(|v| v.as_u64()).unwrap_or(0);
                    index_log!("Batch created {} chunks/logs", succeeded);
                }
            },
            Err(e) => index_log!("Batch create failed: {}", e),
        }
    }

    // Link file to its chunks/log for graph traversal
    for artifact_id in file_artifact_ids {
        match client.create_relationship_direct(file_id, &artifact_id, "defined_in").await {
            Ok(_) => {}
            Err(e) => index_log!("Failed to link file artifact: {}", e),
        }
        match client.create_relationship_direct(&artifact_id, file_id, "defined_in").await {
            Ok(_) => {}
            Err(e) => index_log!("Failed to link file artifact (reverse): {}", e),
        }
    }
    
    // Create dependency edges from parsed file log dependencies
    if !dependency_paths.is_empty() {
        for dep_path in dependency_paths {
            if let Some(dep_id) = resolve_dependency_id(&dep_path, file_path, root_path, file_index) {
                match client.create_relationship_direct(file_id, &dep_id, "depends_on").await {
                    Ok(_) => {}
                    Err(e) => index_log!("Failed to create dependency relationship: {}", e),
                }
            }
        }
    }

    Ok(symbol_count + 1)
}

fn path_key(path: &Path) -> Option<String> {
    let canonical = path.canonicalize().ok().unwrap_or_else(|| path.to_path_buf());
    let mut key = canonical.to_string_lossy().to_string();
    key = key.replace('/', "\\").to_lowercase();
    Some(key)
}

fn resolve_dependency_id(
    dep: &str,
    file_path: &Path,
    root_path: &Path,
    file_index: &HashMap<String, String>,
) -> Option<String> {
    let resolved = resolve_dependency_path(dep, file_path, root_path)?;
    let key = path_key(&resolved)?;
    file_index.get(&key).cloned()
}

fn resolve_dependency_path(dep: &str, file_path: &Path, root_path: &Path) -> Option<PathBuf> {
    let dep_path = Path::new(dep);
    if dep_path.is_absolute() && dep_path.exists() {
        return dep_path.canonicalize().ok().or_else(|| Some(dep_path.to_path_buf()));
    }

    let extensions = ["py", "ts", "tsx", "js", "jsx", "rs", "json", "toml", "yaml", "yml"];
    let looks_like_path = dep.contains('\\') || dep.contains('/') || dep_path.extension().is_some();
    if !looks_like_path {
        return None;
    }

    let candidates = [
        file_path.parent().map(|p| p.join(dep)),
        Some(root_path.join(dep)),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return candidate.canonicalize().ok().or_else(|| Some(candidate));
        }
        if candidate.extension().is_none() {
            for ext in extensions {
                let with_ext = candidate.with_extension(ext);
                if with_ext.exists() {
                    return with_ext.canonicalize().ok().or_else(|| Some(with_ext));
                }
            }
        }
    }

    None
}

async fn create_file_node(
    file_path: &Path,
    project_object_id: &str,
    project_id: &str,
    parent_dir_id: Option<&str>,
    client: &AmpClient,
) -> Result<String> {
    let file_id = Uuid::new_v4().to_string();
    let file_symbol = create_file_node_object(file_path, &file_id, project_id)?;
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    client.create_object(file_symbol).await?;
    
    // Small delay to ensure object is fully created
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // Create relationship: project contains file
    match client.create_relationship_direct(project_object_id, &file_id, "defined_in").await {
        Ok(_) => index_log!(" Created relationship: project contains {}", file_name),
        Err(e) => index_log!("  Failed to create relationship: {}", e),
    }

    // Symmetric relationship for traversal convenience
    match client.create_relationship_direct(&file_id, project_object_id, "defined_in").await {
        Ok(_) => {}
        Err(e) => index_log!("  Failed to create reverse relationship: {}", e),
    }

    if let Some(parent_id) = parent_dir_id {
        let _ = client.create_relationship_direct(parent_id, &file_id, "defined_in").await;
        let _ = client.create_relationship_direct(&file_id, parent_id, "defined_in").await;
    }
    
    Ok(file_id)
}

fn create_amp_symbol_from_parsed_hierarchical(symbol_data: &serde_json::Value, file_path: &Path, _file_id: &str, project_id: &str) -> Result<serde_json::Value> {
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
        "type": "symbol",
        "tenant_id": "default",
        "project_id": project_id,
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

#[allow(dead_code)]
async fn create_simple_file_symbol(file_path: &Path, project_id: &str, client: &AmpClient) -> Result<usize> {
    // Read file content
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(anyhow::anyhow!("Failed to read file: {}", e));
        }
    };
    
    // Create a basic Symbol object for the file
    let symbol = create_file_symbol(file_path, &content, project_id)?;
    
    // Send to AMP server
    match client.create_object(symbol).await {
        Ok(_) => Ok(1), // Created 1 symbol
        Err(e) => Err(anyhow::anyhow!("Failed to create symbol: {}", e)),
    }
}

pub fn create_file_symbol(file_path: &Path, content: &str, project_id: &str) -> Result<Value> {
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
        "type": "symbol",
        "tenant_id": "default",
        "project_id": project_id,
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


#[allow(dead_code)]
async fn create_file_chunks(file_path: &Path, file_id: &str, project_id: &str, client: &AmpClient) -> Result<usize> {
    let content = std::fs::read_to_string(file_path)?;
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("rs") => "rust",
        _ => "text",
    };

    let words: Vec<&str> = content.split_whitespace().collect();
    let chunk_size = 500;
    let overlap = 50;
    
    if words.len() <= chunk_size {
        let chunk = create_chunk_object(file_path, file_id, project_id, &content, 0, 1, content.lines().count() as u32, language);
        client.create_object(chunk).await?;
        return Ok(1);
    }

    let mut created = 0;
    let mut start_idx = 0;
    let mut chunk_idx = 0;

    while start_idx < words.len() {
        let end_idx = (start_idx + chunk_size).min(words.len());
        let chunk_words = &words[start_idx..end_idx];
        let chunk_content = chunk_words.join(" ");
        
        let lines = content.lines().count();
        let start_line = ((start_idx as f32 / words.len() as f32) * lines as f32) as u32 + 1;
        let end_line = ((end_idx as f32 / words.len() as f32) * lines as f32) as u32 + 1;

        let chunk = create_chunk_object(file_path, file_id, project_id, &chunk_content, chunk_idx, start_line, end_line, language);

        match client.create_object(chunk).await {
            Ok(_) => created += 1,
            Err(e) => index_log!("  Failed to create chunk {}: {}", chunk_idx, e),
        }

        chunk_idx += 1;
        start_idx = if end_idx < words.len() { end_idx - overlap } else { break };
    }

    Ok(created)
}

fn create_chunk_object(file_path: &Path, file_id: &str, project_id: &str, content: &str, chunk_index: u32, start_line: u32, end_line: u32, language: &str) -> serde_json::Value {
    let now = chrono::Utc::now();
    let content_hash = format!("{:x}", md5::compute(content.as_bytes()));
    let token_count = content.split_whitespace().count() as u32;

    serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "FileChunk",
        "tenant_id": "default",
        "project_id": project_id,
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": { "source": "amp-cli-chunking", "confidence": 1.0, "method": "word-based-chunking" },
        "links": [],
        "file_path": file_path.to_string_lossy(),
        "file_id": file_id,
        "chunk_index": chunk_index,
        "start_line": start_line,
        "end_line": end_line,
        "token_count": token_count,
        "content": content,
        "content_hash": content_hash,
        "language": language
    })
}

#[allow(dead_code)]
async fn create_file_log(file_path: &Path, file_id: &str, project_id: &str, symbols: &[serde_json::Value], client: &AmpClient) -> Result<()> {
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("rs") => "rust",
        _ => "text",
    };

    let symbol_count = symbols.len();
    let _symbol_types: Vec<String> = symbols.iter().filter_map(|s| s.get("kind").and_then(|k| k.as_str()).map(|s| s.to_string())).collect();
    
    let purpose = if symbol_count > 0 {
        format!("Contains {} symbols", symbol_count)
    } else {
        "Code file".to_string()
    };

    let key_symbols: Vec<String> = symbols.iter().filter_map(|s| {
        let name = s.get("name").and_then(|n| n.as_str())?;
        let kind = s.get("kind").and_then(|k| k.as_str())?;
        Some(format!("{}:{}", kind, name))
    }).take(20).collect();

    let summary = format!("# {}\n\n**Language**: {}\n\n## Purpose\n{}\n\n## Symbols\n{}\n", file_path.display(), language, purpose, key_symbols.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"));

    let now = chrono::Utc::now();
    let file_log = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "FileLog",
        "tenant_id": "default",
        "project_id": project_id,
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": { "source": "amp-cli-filelog", "confidence": 0.9, "method": "symbol-based-summary" },
        "links": [],
        "file_path": file_path.to_string_lossy(),
        "file_id": file_id,
        "summary": summary,
        "summary_markdown": summary,
        "purpose": purpose,
        "key_symbols": key_symbols,
        "dependencies": [],
        "last_modified": now.to_rfc3339(),
        "change_count": 0,
        "linked_changesets": []
    });

    client.create_object(file_log).await?;
    Ok(())
}

fn create_file_node_object(file_path: &Path, file_id: &str, project_id: &str) -> Result<Value> {
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

    let file_size = std::fs::metadata(file_path)
        .map(|meta| meta.len())
        .unwrap_or(0);
    let line_count = std::fs::read_to_string(file_path)
        .map(|content| content.lines().count() as u64)
        .unwrap_or(0);
    
    Ok(json!({
        "id": file_id,
        "type": "symbol",
        "tenant_id": "default",
        "project_id": project_id,
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
        "file_size": file_size,
        "line_count": line_count,
        "content_hash": format!("{:x}", md5::compute(file_name.as_bytes())),
        "signature": format!("file: {}", file_name),
        "documentation": format!("File: {}", file_path.display())
    }))
}

async fn use_codebase_parser_hierarchical(file_path: &Path, file_id: &str, project_id: &str, client: &AmpClient) -> Result<(usize, Vec<String>, Vec<String>)> {
    let absolute_path = file_path.canonicalize()?;
    
    let parse_request = serde_json::json!({
        "file_path": absolute_path.to_string_lossy(),
        "project_id": project_id,
        "tenant_id": "default"
    });
    
    let response = client.parse_file(parse_request).await?;
    
    let mut dependencies: Vec<String> = Vec::new();
    let mut symbol_names: Vec<String> = Vec::new();

    if let Some(file_log) = response.get("file_log") {
        if let Some(symbols) = file_log.get("symbols") {
            if let Some(symbols_array) = symbols.as_array() {
                let mut created_count = 0;
                for symbol_data in symbols_array {
                    if let Ok(amp_symbol) = create_amp_symbol_from_parsed_hierarchical(symbol_data, file_path, file_id, project_id) {
                        match client.create_object(amp_symbol.clone()).await {
                            Ok(_) => {
                                created_count += 1;
                                if let Some(symbol_id) = amp_symbol.get("id").and_then(|v| v.as_str()) {
                                    match client.create_relationship_direct(file_id, symbol_id, "defined_in").await {
                                        Ok(_) => {},
                                        Err(e) => index_log!("  Failed to create relationship: {}", e),
                                    }
                                    match client.create_relationship_direct(symbol_id, file_id, "defined_in").await {
                                        Ok(_) => {},
                                        Err(e) => index_log!("  Failed to create reverse relationship: {}", e),
                                    }
                                }
                            },
                            Err(e) => index_log!("  Failed to create symbol: {}", e),
                        }
                    }
                    if let Some(name) = symbol_data.get("name").and_then(|v| v.as_str()) {
                        symbol_names.push(name.to_string());
                    }
                }
                if let Some(deps) = file_log.get("dependencies") {
                    if let Some(arr) = deps.as_array() {
                        for dep in arr {
                            if let Some(path) = dep.as_str() {
                                dependencies.push(path.to_string());
                            }
                        }
                    }
                }

                return Ok((created_count, dependencies, symbol_names));
            }
        }
    }
    
    Ok((0, dependencies, symbol_names))
}

fn create_file_chunks_objects(file_path: &Path, file_id: &str, project_id: &str) -> Result<Vec<Value>> {
    let content = std::fs::read_to_string(file_path)?;
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("rs") => "rust",
        _ => "text",
    };

    let words: Vec<&str> = content.split_whitespace().collect();
    let chunk_size = 500;
    let overlap = 50;
    
    if words.len() <= chunk_size {
        let chunk = create_chunk_object(file_path, file_id, project_id, &content, 0, 1, content.lines().count() as u32, language);
        return Ok(vec![chunk]);
    }

    let mut chunks = Vec::new();
    let mut start_idx = 0;
    let mut chunk_idx = 0;

    while start_idx < words.len() {
        let end_idx = (start_idx + chunk_size).min(words.len());
        let chunk_words = &words[start_idx..end_idx];
        let chunk_content = chunk_words.join(" ");
        
        let lines = content.lines().count();
        let start_line = ((start_idx as f32 / words.len() as f32) * lines as f32) as u32 + 1;
        let end_line = ((end_idx as f32 / words.len() as f32) * lines as f32) as u32 + 1;

        let chunk = create_chunk_object(file_path, file_id, project_id, &chunk_content, chunk_idx, start_line, end_line, language);
        chunks.push(chunk);

        chunk_idx += 1;
        start_idx = if end_idx < words.len() { end_idx - overlap } else { break };
    }

    Ok(chunks)
}

fn create_file_log_object(file_path: &Path, file_id: &str, project_id: &str, symbols: &[Value]) -> Result<Value> {
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("rs") => "rust",
        _ => "text",
    };

    let symbol_count = symbols.len();
    let purpose = if symbol_count > 0 {
        format!("Contains {} symbols", symbol_count)
    } else {
        "Code file".to_string()
    };

    let key_symbols: Vec<String> = symbols.iter().filter_map(|s| {
        let name = s.get("name").and_then(|n| n.as_str())?;
        let kind = s.get("kind").and_then(|k| k.as_str())?;
        Some(format!("{}:{}", kind, name))
    }).take(20).collect();

    let summary = format!("# {}\n\n**Language**: {}\n\n## Purpose\n{}\n\n## Symbols\n{}\n", file_path.display(), language, purpose, key_symbols.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"));

    let now = chrono::Utc::now();
    Ok(json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "FileLog",
        "tenant_id": "default",
        "project_id": project_id,
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": { "source": "amp-cli-filelog", "confidence": 0.9, "method": "symbol-based-summary" },
        "links": [],
        "file_path": file_path.to_string_lossy(),
        "file_id": file_id,
        "summary": summary,
        "summary_markdown": summary,
        "purpose": purpose,
        "key_symbols": key_symbols,
        "dependencies": [],
        "last_modified": now.to_rfc3339(),
        "change_count": 0,
        "linked_changesets": []
    }))
}

async fn create_file_log_object_ai(
    file_path: &Path,
    file_id: &str,
    project_id: &str,
    symbols: &[String],
    dependencies: &[String],
    client: &AmpClient,
) -> Result<Value> {
    let language = match file_path.extension().and_then(|e| e.to_str()) {
        Some("py") => "python",
        Some("ts") | Some("tsx") => "typescript",
        Some("js") | Some("jsx") => "javascript",
        Some("rs") => "rust",
        _ => "text",
    };

    let content = std::fs::read_to_string(file_path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let content_hash = format!("sha256:{:x}", hasher.finalize());
    let (prepared_content, was_truncated) = truncate_ai_log_content(&content);
    if was_truncated {
        index_log!("  Truncated AI log content for {}", file_path.display());
    }
    let payload = serde_json::json!({
        "file_path": file_path.to_string_lossy(),
        "language": language,
        "content_hash": content_hash,
        "content": prepared_content,
        "symbols": symbols,
        "dependencies": dependencies,
    });

    match client.generate_ai_file_log(payload).await {
        Ok(response) => {
            if let Some(file_log) = response.get("file_log") {
                return create_file_log_object_from_ai(file_path, file_id, project_id, file_log);
            }
            index_log!("  AI file log response missing file_log, using fallback");
        }
        Err(err) => {
            index_log!("  AI file log generation failed: {}", err);
        }
    }

    create_file_log_object(file_path, file_id, project_id, &[])
}

fn truncate_ai_log_content(content: &str) -> (String, bool) {
    let length = content.chars().count();
    if length <= MAX_AI_LOG_CONTENT_CHARS {
        return (content.to_string(), false);
    }

    let head: String = content.chars().take(AI_LOG_CONTENT_HEAD_CHARS).collect();
    let tail: String = content
        .chars()
        .rev()
        .take(AI_LOG_CONTENT_TAIL_CHARS)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    let combined = format!(
        "{}\n\n... [truncated for AI log generation] ...\n\n{}",
        head, tail
    );
    (combined, true)
}

fn create_file_log_object_from_ai(
    file_path: &Path,
    file_id: &str,
    project_id: &str,
    ai_log: &Value,
) -> Result<Value> {
    let now = chrono::Utc::now();
    let summary = ai_log.get("summary_markdown").and_then(|v| v.as_str()).unwrap_or("");
    let purpose = ai_log.get("purpose").and_then(|v| v.as_str()).map(|s| s.to_string());
    let notes = ai_log.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
    let key_symbols: Vec<String> = ai_log
        .get("key_symbols")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(|v| v.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let dependencies: Vec<String> = ai_log
        .get("dependencies")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.as_str().map(|v| v.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Ok(json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "FileLog",
        "tenant_id": "default",
        "project_id": project_id,
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
        "provenance": { "source": "amp-cli-filelog", "confidence": 0.9, "method": "ai-summary" },
        "links": [],
        "file_path": file_path.to_string_lossy(),
        "file_id": file_id,
        "summary": summary,
        "summary_markdown": summary,
        "purpose": purpose,
        "key_symbols": key_symbols,
        "dependencies": dependencies,
        "notes": notes,
        "last_modified": now.to_rfc3339(),
        "change_count": 0,
        "linked_changesets": []
    }))
}

async fn create_directory_log_ai(
    dir_path: &Path,
    dir_id: &str,
    project_id: &str,
    client: &AmpClient,
) -> Result<Value> {
    let mut entries: Vec<String> = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(dir_path) {
        for entry in read_dir.flatten().take(200) {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
    }

    let content = if entries.is_empty() {
        "Directory contains no readable entries.".to_string()
    } else {
        format!("Directory entries:\n- {}", entries.join("\n- "))
    };
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let content_hash = format!("sha256:{:x}", hasher.finalize());

    let payload = serde_json::json!({
        "file_path": dir_path.to_string_lossy(),
        "language": "directory",
        "content_hash": content_hash,
        "content": content,
        "symbols": [],
        "dependencies": entries,
    });

    match client.generate_ai_file_log(payload).await {
        Ok(response) => {
            if let Some(file_log) = response.get("file_log") {
                return create_file_log_object_from_ai(dir_path, dir_id, project_id, file_log);
            }
            index_log!("  Directory AI file log response missing file_log, using fallback");
        }
        Err(err) => {
            index_log!("  Directory AI file log generation failed: {}", err);
        }
    }

    let summary = format!(
        "# FILE_LOG v1\npath: {}\nlanguage: directory\nlast_indexed: {}\ncontent_hash: {}\n\n## Symbols (current snapshot)\n- None\n\n## Dependencies (best-effort)\nimports:\n- None\nexports:\n- None\n\n## Recent Changes (rolling, last N)\n- None\n\n## Notes / Decisions linked\n- {}\n",
        dir_path.to_string_lossy(),
        chrono::Utc::now().to_rfc3339(),
        content_hash,
        if entries.is_empty() { "Directory contains no readable entries." } else { "See directory entries list." }
    );

    Ok(json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "FileLog",
        "tenant_id": "default",
        "project_id": project_id,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339(),
        "provenance": { "source": "amp-cli-filelog", "confidence": 0.7, "method": "directory-fallback" },
        "links": [],
        "file_path": dir_path.to_string_lossy(),
        "file_id": dir_id,
        "summary": summary,
        "summary_markdown": summary,
        "purpose": "Directory overview",
        "key_symbols": [],
        "dependencies": entries,
        "notes": null,
        "last_modified": chrono::Utc::now().to_rfc3339(),
        "change_count": 0,
        "linked_changesets": []
    }))
}
