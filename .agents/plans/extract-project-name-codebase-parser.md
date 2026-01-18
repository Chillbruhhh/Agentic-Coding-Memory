# Feature: Extract Project Name in Codebase Parser

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Enhance the codebase parser to intelligently extract and populate project names from common project configuration files (package.json, Cargo.toml, pyproject.toml, etc.) instead of using the directory name as fallback. This will provide more accurate project identification when indexing codebases, especially for projects with generic directory names or when indexing subdirectories.

## User Story

As a developer using AMP CLI to index codebases
I want the system to automatically detect the actual project name from configuration files
So that my indexed projects have meaningful names instead of generic directory names like "src" or "backend"

## Problem Statement

Currently, the codebase parser uses `root_path.file_name()` to determine project names, which results in:
- Generic names like "src", "backend", "frontend" when indexing subdirectories
- Directory names that don't match the actual project name
- Inconsistent project identification across different project structures
- Poor user experience when viewing indexed projects

## Solution Statement

Implement intelligent project name detection by:
1. Scanning for common project configuration files in the root directory
2. Parsing these files to extract the actual project name
3. Falling back to directory name only when no configuration files are found
4. Supporting multiple project types (Node.js, Rust, Python, PHP, Java, etc.)

## Feature Metadata

**Feature Type**: Enhancement
**Estimated Complexity**: Medium
**Primary Systems Affected**: Codebase Parser, CLI Index Command
**Dependencies**: serde_json (already available), toml parsing crate

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/cli/src/commands/index.rs` (lines 190-220) - Why: Contains current project name extraction logic that needs enhancement
- `amp/server/src/services/codebase_parser.rs` (lines 1-50) - Why: Main codebase parser structure and dependencies
- `amp/cli/Cargo.toml` - Why: Need to add toml parsing dependency
- `amp/server/Cargo.toml` - Why: May need toml parsing dependency for server-side parsing

### New Files to Create

- None (enhancement to existing functionality)

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [Serde JSON Documentation](https://docs.rs/serde_json/latest/serde_json/)
  - Specific section: Parsing JSON values
  - Why: Required for parsing package.json files
- [TOML Parsing Documentation](https://docs.rs/toml/latest/toml/)
  - Specific section: Basic usage and parsing
  - Why: Required for parsing Cargo.toml and pyproject.toml files

### Patterns to Follow

**Error Handling Pattern:**
```rust
match std::fs::read_to_string(config_path) {
    Ok(content) => { /* parse content */ },
    Err(_) => { /* continue to next config file */ }
}
```

**JSON Parsing Pattern:**
```rust
if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&content) {
    if let Some(name) = json_value.get("name").and_then(|v| v.as_str()) {
        return Some(name.to_string());
    }
}
```

**Logging Pattern:**
```rust
println!("🔍 Found project config: {}", config_file);
println!("📝 Extracted project name: {}", project_name);
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Add TOML parsing capability to handle Rust and Python project files.

**Tasks:**
- Add toml dependency to CLI Cargo.toml
- Import necessary parsing modules

### Phase 2: Core Implementation

Implement project name extraction logic with multiple configuration file support.

**Tasks:**
- Create project name detection function
- Add support for package.json (Node.js)
- Add support for Cargo.toml (Rust)
- Add support for pyproject.toml (Python)
- Add support for composer.json (PHP)
- Add support for pom.xml (Java/Maven)

### Phase 3: Integration

Integrate the new project name detection into the existing index command.

**Tasks:**
- Replace hardcoded directory name logic
- Add fallback mechanism
- Update logging and user feedback

### Phase 4: Testing & Validation

Test with various project types and edge cases.

**Tasks:**
- Test with Node.js projects
- Test with Rust projects
- Test with Python projects
- Test fallback behavior
- Validate error handling

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### ADD amp/cli/Cargo.toml

- **IMPLEMENT**: Add toml parsing dependency
- **PATTERN**: Follow existing dependency format in Cargo.toml
- **IMPORTS**: `toml = "0.8"`
- **GOTCHA**: Place in [dependencies] section, not [dev-dependencies]
- **VALIDATE**: `cd amp/cli && cargo check`

### CREATE amp/cli/src/commands/index.rs

- **IMPLEMENT**: Add project name detection function before `create_project_node`
- **PATTERN**: Follow existing function structure with Result return type
- **IMPORTS**: Add `use std::collections::HashMap;` and `use serde_json::Value;`
- **GOTCHA**: Handle file reading errors gracefully, don't fail the entire indexing
- **VALIDATE**: `cd amp/cli && cargo check`

```rust
fn detect_project_name(root_path: &Path) -> Option<String> {
    // Configuration files to check in priority order
    let config_files = vec![
        ("package.json", extract_name_from_package_json),
        ("Cargo.toml", extract_name_from_cargo_toml),
        ("pyproject.toml", extract_name_from_pyproject_toml),
        ("composer.json", extract_name_from_composer_json),
    ];
    
    for (filename, extractor) in config_files {
        let config_path = root_path.join(filename);
        if config_path.exists() {
            println!("🔍 Found project config: {}", filename);
            if let Some(name) = extractor(&config_path) {
                println!("📝 Extracted project name: {}", name);
                return Some(name);
            }
        }
    }
    
    None
}
```

### ADD amp/cli/src/commands/index.rs

- **IMPLEMENT**: Add JSON parsing function for package.json
- **PATTERN**: Follow error handling pattern with early returns
- **IMPORTS**: Already available serde_json
- **GOTCHA**: Handle malformed JSON gracefully
- **VALIDATE**: `cd amp/cli && cargo check`

```rust
fn extract_name_from_package_json(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let json_value: serde_json::Value = serde_json::from_str(&content).ok()?;
    json_value.get("name")?.as_str().map(|s| s.to_string())
}
```

### ADD amp/cli/src/commands/index.rs

- **IMPLEMENT**: Add TOML parsing function for Cargo.toml
- **PATTERN**: Mirror JSON parsing structure
- **IMPORTS**: Need to add `use toml;` at top of file
- **GOTCHA**: Cargo.toml has nested structure: [package].name
- **VALIDATE**: `cd amp/cli && cargo check`

```rust
fn extract_name_from_cargo_toml(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let toml_value: toml::Value = toml::from_str(&content).ok()?;
    toml_value.get("package")?.get("name")?.as_str().map(|s| s.to_string())
}
```

### ADD amp/cli/src/commands/index.rs

- **IMPLEMENT**: Add TOML parsing function for pyproject.toml
- **PATTERN**: Mirror Cargo.toml structure
- **IMPORTS**: Already imported toml
- **GOTCHA**: pyproject.toml has nested structure: [project].name
- **VALIDATE**: `cd amp/cli && cargo check`

```rust
fn extract_name_from_pyproject_toml(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let toml_value: toml::Value = toml::from_str(&content).ok()?;
    toml_value.get("project")?.get("name")?.as_str().map(|s| s.to_string())
}
```

### ADD amp/cli/src/commands/index.rs

- **IMPLEMENT**: Add JSON parsing function for composer.json
- **PATTERN**: Mirror package.json structure
- **IMPORTS**: Already available serde_json
- **GOTCHA**: Composer uses "name" field directly like package.json
- **VALIDATE**: `cd amp/cli && cargo check`

```rust
fn extract_name_from_composer_json(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let json_value: serde_json::Value = serde_json::from_str(&content).ok()?;
    json_value.get("name")?.as_str().map(|s| s.to_string())
}
```

### UPDATE amp/cli/src/commands/index.rs

- **IMPLEMENT**: Replace hardcoded project name logic in `create_project_node` function
- **PATTERN**: Use detect_project_name with fallback to directory name
- **IMPORTS**: No new imports needed
- **GOTCHA**: Maintain backward compatibility - always have a project name
- **VALIDATE**: `cd amp/cli && cargo check`

Replace this line (around line 195):
```rust
let project_name = root_path.file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("project");
```

With:
```rust
let project_name = detect_project_name(root_path)
    .unwrap_or_else(|| {
        root_path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "project".to_string())
    });
```

### UPDATE amp/cli/src/commands/index.rs

- **IMPLEMENT**: Update logging to show detection method
- **PATTERN**: Follow existing println! format with emoji
- **IMPORTS**: No new imports needed
- **GOTCHA**: Only log when project name is detected from config
- **VALIDATE**: `cd amp/cli && cargo check`

Add after project name detection:
```rust
if detect_project_name(root_path).is_some() {
    println!("📝 Using project name from configuration: {}", project_name);
} else {
    println!("📁 Using directory name as project name: {}", project_name);
}
```

---

## TESTING STRATEGY

### Unit Tests

Add tests to verify project name extraction from different configuration files:

- Test package.json parsing with valid and invalid JSON
- Test Cargo.toml parsing with valid and invalid TOML
- Test pyproject.toml parsing with valid and invalid TOML
- Test composer.json parsing with valid and invalid JSON
- Test fallback behavior when no config files exist

### Integration Tests

Test the full indexing workflow with different project types:

- Create temporary directories with different config files
- Run index command and verify correct project names are used
- Test with missing config files to ensure fallback works

### Edge Cases

- Empty config files
- Config files with missing name fields
- Config files with non-string name values
- Multiple config files present (priority order)
- Permission errors reading config files

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
cd amp/cli && cargo fmt
cd amp/cli && cargo clippy -- -D warnings
```

### Level 2: Unit Tests

```bash
cd amp/cli && cargo test
```

### Level 3: Integration Tests

```bash
cd amp/cli && cargo test --test integration
```

### Level 4: Manual Validation

Test with different project types:

```bash
# Test with Node.js project (create temp package.json)
mkdir /tmp/test-node && echo '{"name": "my-awesome-app"}' > /tmp/test-node/package.json
cd amp/cli && cargo run -- index /tmp/test-node

# Test with Rust project (use existing amp project)
cd amp/cli && cargo run -- index ../

# Test with fallback (empty directory)
mkdir /tmp/test-empty
cd amp/cli && cargo run -- index /tmp/test-empty
```

### Level 5: Additional Validation (Optional)

```bash
# Check that server can handle the indexed projects
cd amp/server && cargo run &
sleep 2
curl http://localhost:8105/v1/query -X POST -H "Content-Type: application/json" -d '{"text": "project"}'
```

---

## ACCEPTANCE CRITERIA

- [ ] CLI can extract project names from package.json files
- [ ] CLI can extract project names from Cargo.toml files  
- [ ] CLI can extract project names from pyproject.toml files
- [ ] CLI can extract project names from composer.json files
- [ ] Fallback to directory name works when no config files exist
- [ ] Error handling prevents crashes on malformed config files
- [ ] Priority order is respected when multiple config files exist
- [ ] Logging clearly indicates detection method used
- [ ] All validation commands pass with zero errors
- [ ] No regressions in existing indexing functionality
- [ ] Performance impact is minimal (config file parsing is fast)

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit + integration)
- [ ] No linting or type checking errors
- [ ] Manual testing confirms feature works with different project types
- [ ] Acceptance criteria all met
- [ ] Code reviewed for quality and maintainability

---

## NOTES

**Design Decisions:**
- Priority order: package.json > Cargo.toml > pyproject.toml > composer.json > directory name
- Graceful error handling: malformed config files don't break indexing
- Minimal dependencies: only add toml crate, reuse existing serde_json

**Performance Considerations:**
- Config file parsing happens once per indexing operation
- File reading is lazy (only when files exist)
- Parsing failures fall back quickly to next option

**Future Extensions:**
- Could add support for more project types (pom.xml, build.gradle, etc.)
- Could extract additional metadata (version, description) from config files
- Could cache parsed config data for repeated operations
