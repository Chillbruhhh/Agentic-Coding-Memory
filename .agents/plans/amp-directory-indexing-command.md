# Feature: AMP Directory Indexing Command

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Add an `amp index` command that scans the current directory (repository root) and automatically creates AMP memory objects for all code symbols, files, and project structure. This provides instant project context for agents by populating the AMP server with Symbol objects representing functions, classes, modules, and other code structures.

## User Story

As a developer using AMP with coding agents
I want to run `amp index` in my project root
So that my agents have immediate access to comprehensive project memory without manual setup

## Problem Statement

Currently, AMP requires manual creation of memory objects. Developers need an automated way to populate AMP with their entire codebase structure so agents can immediately understand project context, navigate code relationships, and make informed decisions.

## Solution Statement

Implement an `amp index` CLI command that leverages the existing codebase parser service to scan the current directory, extract all code symbols, and automatically create Symbol objects in the AMP server. The command will provide progress feedback and summary statistics.

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: Medium
**Primary Systems Affected**: CLI, Codebase Parser Service, AMP Server
**Dependencies**: Existing codebase parser, AMP client, file system traversal

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/services/codebase_parser.rs` (lines 1-200) - Why: Contains complete parsing logic we'll use
- `amp/server/src/handlers/codebase.rs` (lines 1-100) - Why: Shows API endpoints for codebase operations
- `amp/cli/src/main.rs` (lines 1-50) - Why: CLI command structure and patterns
- `amp/cli/src/commands/mod.rs` - Why: Command module organization
- `amp/cli/src/client.rs` (lines 1-100) - Why: AMP client usage patterns
- `amp/cli/src/config.rs` - Why: Configuration handling patterns

### New Files to Create

- `amp/cli/src/commands/index.rs` - Index command implementation
- `amp/cli/tests/test_index.rs` - Unit tests for index command

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [AMP Codebase Parser Documentation](amp/CODEBASE_PARSER.md)
  - Specific section: Parser API and Symbol extraction
  - Why: Understanding of parsing capabilities and output format
- [Clap Command Documentation](https://docs.rs/clap/latest/clap/)
  - Specific section: Subcommands and argument parsing
  - Why: Required for implementing CLI commands properly

### Patterns to Follow

**CLI Command Pattern:**
```rust
// From amp/cli/src/main.rs
#[derive(Subcommand)]
enum Commands {
    /// Command description
    CommandName { 
        /// Argument description
        arg: String 
    },
}
```

**AMP Client Usage Pattern:**
```rust
// From amp/cli/src/client.rs
let client = AmpClient::new(&config.server_url);
let response = client.post("/v1/objects")
    .json(&object)
    .send()
    .await?;
```

**Error Handling Pattern:**
```rust
// Standard Rust Result pattern used throughout
async fn function_name() -> Result<()> {
    // Implementation
    Ok(())
}
```

**Progress Reporting Pattern:**
```rust
// Console output pattern from existing scripts
println!("🔍 Scanning directory: {}", path);
println!("✅ Created {} symbols", count);
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Set up the CLI command structure and basic argument parsing for the index command.

**Tasks:**
- Add index subcommand to CLI enum
- Create index command module with basic structure
- Set up argument parsing for optional directory path

### Phase 2: Core Implementation

Implement the directory scanning and symbol extraction logic using existing codebase parser.

**Tasks:**
- Implement directory traversal with file filtering
- Integrate with codebase parser service
- Create Symbol objects from parsed results
- Add progress reporting and statistics

### Phase 3: Integration

Connect the index command to the AMP server and handle API communication.

**Tasks:**
- Integrate AMP client for object creation
- Implement batch creation for performance
- Add error handling and retry logic
- Handle server connectivity issues gracefully

### Phase 4: Testing & Validation

Create comprehensive tests and validation for the index functionality.

**Tasks:**
- Implement unit tests for index logic
- Create integration tests with mock server
- Add edge case handling tests
- Validate against real project directories

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### UPDATE amp/cli/src/main.rs

- **IMPLEMENT**: Add Index variant to Commands enum
- **PATTERN**: Follow existing command pattern (Start, Status, History, Tui)
- **IMPORTS**: No new imports needed
- **GOTCHA**: Maintain alphabetical order in enum
- **VALIDATE**: `cargo check` in amp/cli directory

```rust
/// Index the current directory and create AMP memory objects
Index {
    /// Directory to index (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    path: String,
    /// Skip files matching these patterns
    #[arg(long, value_delimiter = ',')]
    exclude: Vec<String>,
},
```

### UPDATE amp/cli/src/main.rs

- **IMPLEMENT**: Add Index command handler in match statement
- **PATTERN**: Mirror existing command handlers
- **IMPORTS**: Add `commands::index`
- **GOTCHA**: Handle async properly with await
- **VALIDATE**: `cargo check` in amp/cli directory

```rust
Commands::Index { path, exclude } => {
    commands::index::run_index(&path, &exclude, &client).await?;
}
```

### CREATE amp/cli/src/commands/index.rs

- **IMPLEMENT**: Complete index command implementation
- **PATTERN**: Follow existing command module structure from status.rs
- **IMPORTS**: `std::path::Path, walkdir::WalkDir, crate::client::AmpClient, anyhow::Result`
- **GOTCHA**: Handle file system errors gracefully
- **VALIDATE**: `cargo check` in amp/cli directory

### UPDATE amp/cli/src/commands/mod.rs

- **IMPLEMENT**: Add index module declaration
- **PATTERN**: Follow existing module declarations
- **IMPORTS**: None needed
- **GOTCHA**: Maintain alphabetical order
- **VALIDATE**: `cargo check` in amp/cli directory

```rust
pub mod index;
```

### UPDATE amp/cli/Cargo.toml

- **IMPLEMENT**: Add walkdir dependency for directory traversal
- **PATTERN**: Follow existing dependency format
- **IMPORTS**: `walkdir = "2.0"`
- **GOTCHA**: Use compatible version with existing deps
- **VALIDATE**: `cargo check` in amp/cli directory

### CREATE amp/cli/tests/test_index.rs

- **IMPLEMENT**: Unit tests for index functionality
- **PATTERN**: Follow Rust testing conventions
- **IMPORTS**: `tempfile, tokio-test, amp_cli::commands::index`
- **GOTCHA**: Use async test framework
- **VALIDATE**: `cargo test test_index` in amp/cli directory

---

## TESTING STRATEGY

### Unit Tests

Design unit tests with temporary directories and mock file structures following Rust testing patterns:

- Test directory scanning with various file types
- Test exclude pattern filtering
- Test Symbol object creation from parsed files
- Test error handling for invalid directories
- Test progress reporting output

### Integration Tests

- Test full workflow with running AMP server
- Test batch object creation performance
- Test handling of large codebases
- Test network error recovery

### Edge Cases

- Empty directories
- Directories with no supported file types
- Permission denied scenarios
- Network connectivity issues
- Invalid exclude patterns
- Very large files or deep directory structures

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
cd amp/cli
cargo fmt --check
cargo clippy -- -D warnings
```

### Level 2: Unit Tests

```bash
cd amp/cli
cargo test
cargo test test_index
```

### Level 3: Integration Tests

```bash
cd amp/cli
cargo test --test integration
```

### Level 4: Manual Validation

```bash
# Build and test the command
cd amp/cli
cargo build

# Test help output
cargo run -- index --help

# Test actual indexing (requires AMP server running)
cargo run -- index
cargo run -- index --path ../server --exclude "target,*.log"
```

### Level 5: Additional Validation

```bash
# Test with real project
cd amp/cli
cargo run -- index --path ../../ --exclude ".git,target,node_modules"
```

---

## ACCEPTANCE CRITERIA

- [ ] `amp index` command successfully scans current directory
- [ ] Creates Symbol objects for all supported file types (Python, TypeScript)
- [ ] Provides progress feedback during scanning
- [ ] Supports --path argument for custom directory
- [ ] Supports --exclude argument for filtering files
- [ ] Handles errors gracefully (network, file system, permissions)
- [ ] Shows summary statistics after completion
- [ ] All validation commands pass with zero errors
- [ ] Unit test coverage meets requirements (80%+)
- [ ] Integration tests verify end-to-end workflow
- [ ] Code follows existing CLI patterns and conventions
- [ ] No regressions in existing CLI functionality
- [ ] Performance acceptable for typical project sizes

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit + integration)
- [ ] No linting or type checking errors
- [ ] Manual testing confirms feature works
- [ ] Acceptance criteria all met
- [ ] Code reviewed for quality and maintainability

---

## NOTES

**Design Decisions:**
- Using `amp index` over `amp init` because it's more descriptive of the action
- Supporting exclude patterns for flexibility with large projects
- Defaulting to current directory for convenience
- Using existing codebase parser to maintain consistency
- Implementing batch creation for better performance

**Performance Considerations:**
- Large codebases may take time to process
- Consider adding --dry-run flag for testing
- Batch API calls to reduce network overhead
- Progress reporting for user feedback

**Future Enhancements:**
- Watch mode for continuous indexing
- Incremental updates based on file changes
- Support for additional programming languages
- Configuration file for default exclude patterns
