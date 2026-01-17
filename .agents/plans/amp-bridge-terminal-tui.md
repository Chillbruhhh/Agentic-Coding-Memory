# Feature: AMP Bridge - Terminal-Based Agent Integration TUI

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

"The Bridge" is a terminal-based TUI (Text User Interface) that serves as the primary integration layer between coding agents and AMP (Agentic Memory Protocol). It provides a seamless interface for users to index codebases with AMP, start sessions, launch their favorite coding agents, and automatically capture all agent activity for persistent memory storage.

This is the critical infrastructure piece that enables AMP to function as a system-of-record for agentic development activity without requiring modifications to existing coding agents.

## User Story

As a developer using coding agents (Claude Code, Kiro CLI, Cursor, etc.)
I want to seamlessly integrate my agent workflows with AMP memory storage
So that all my coding sessions are automatically captured, indexed, and made available for future agent context and retrieval

## Problem Statement

Current coding agents operate as isolated terminal applications without persistent memory or cross-session context sharing. Developers lose valuable context between sessions, and agents cannot learn from previous interactions or build upon past work. Traditional wrapper approaches are insufficient for interactive TUI-based agents that require complex terminal management.

## Solution Statement

Build a terminal-based TUI that acts as "The Bridge" between any coding agent and AMP. The solution provides transparent session management, automatic output capture, and durable provenance tracking while preserving native agent workflows. Users can launch agents through the Bridge, which handles all AMP integration invisibly.

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: High
**Primary Systems Affected**: New CLI component, AMP Server API, Session Management, Process Management
**Dependencies**: ratatui, crossterm, tokio, reqwest, clap

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/main.rs` (lines 1-100) - Why: Server initialization patterns and AppState structure
- `amp/server/src/config.rs` (lines 1-50) - Why: Configuration patterns to extend for CLI
- `amp/server/src/handlers/objects.rs` (lines 1-50) - Why: API patterns for creating Run and ChangeSet objects
- `amp/server/src/handlers/query.rs` (lines 1-30) - Why: Query API patterns for session retrieval
- `amp/server/src/database.rs` (lines 1-50) - Why: Database connection patterns (for reference)
- `amp/server/Cargo.toml` - Why: Dependency patterns and workspace structure
- `amp-session-lifecycle.md` - Why: Complete session lifecycle specification

### New Files to Create

- `amp/cli/Cargo.toml` - CLI workspace member with TUI dependencies
- `amp/cli/src/main.rs` - CLI entry point with clap commands
- `amp/cli/src/app.rs` - Main TUI application state and event loop
- `amp/cli/src/client.rs` - HTTP client for AMP server communication
- `amp/cli/src/session.rs` - Session management and lifecycle
- `amp/cli/src/ui/` - TUI components and rendering
- `amp/cli/src/commands/` - CLI command implementations
- `amp/cli/src/process.rs` - Agent process spawning and monitoring
- `amp/cli/src/config.rs` - CLI-specific configuration

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [Ratatui Book](https://ratatui.rs/tutorials/hello-world/)
  - Specific section: Basic TUI setup and event loops
  - Why: Core TUI patterns and architecture
- [Crossterm Documentation](https://docs.rs/crossterm/latest/crossterm/)
  - Specific section: Terminal management and events
  - Why: Terminal control and process integration
- [Tokio Process Documentation](https://docs.rs/tokio/latest/tokio/process/)
  - Specific section: Async process spawning
  - Why: Agent process management patterns
- [Clap Documentation](https://docs.rs/clap/latest/clap/)
  - Specific section: Command-line parsing
  - Why: CLI interface design

### Patterns to Follow

**Configuration Pattern:**
```rust
// From amp/server/src/config.rs
#[derive(Debug, Clone)]
pub struct Config {
    pub server_url: String,
    pub session_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Environment-based configuration
    }
}
```

**HTTP Client Pattern:**
```rust
// Use reqwest for AMP server communication
let client = reqwest::Client::new();
let response = client.post(&format!("{}/v1/objects", config.server_url))
    .json(&object)
    .send()
    .await?;
```

**Error Handling Pattern:**
```rust
// Use anyhow::Result throughout
use anyhow::{anyhow, Result};

pub async fn create_session() -> Result<Session> {
    // Implementation with ? operator
}
```

**Async Runtime Pattern:**
```rust
// From amp/server/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tokio async runtime
}
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Set up the CLI workspace structure and basic TUI framework with session management capabilities.

**Tasks:**
- Create CLI workspace member in amp/cli/
- Set up basic TUI application structure with ratatui + crossterm
- Implement configuration management extending server patterns
- Create HTTP client for AMP server communication

### Phase 2: Core Session Management

Implement the session lifecycle management system with lease-based tracking.

**Tasks:**
- Implement Session model and local storage
- Create lease management with heartbeat system
- Add session state persistence and recovery
- Implement automatic session finalization

### Phase 3: Agent Integration

Build the process management system for spawning and monitoring coding agents.

**Tasks:**
- Implement agent process spawning with output capture
- Add terminal passthrough for interactive agents
- Create process monitoring and cleanup
- Implement git diff capture on session end

### Phase 4: TUI Interface

Create the interactive terminal interface for session management and monitoring.

**Tasks:**
- Build main TUI layout and navigation
- Add session status display and controls
- Implement real-time agent output display
- Create session history and management views

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### CREATE amp/cli/Cargo.toml

- **IMPLEMENT**: CLI workspace member with TUI dependencies
- **PATTERN**: Mirror amp/server/Cargo.toml structure
- **IMPORTS**: 
  ```toml
  [dependencies]
  tokio = { workspace = true }
  anyhow = { workspace = true }
  serde = { workspace = true }
  serde_json = { workspace = true }
  uuid = { workspace = true }
  chrono = { workspace = true }
  
  ratatui = "0.26"
  crossterm = "0.27"
  clap = { version = "4.0", features = ["derive"] }
  reqwest = { version = "0.11", features = ["json"] }
  tokio-stream = "0.1"
  dirs = "5.0"
  ```
- **GOTCHA**: Use workspace dependencies where available
- **VALIDATE**: `cd amp/cli && cargo check`

### UPDATE amp/Cargo.toml

- **IMPLEMENT**: Add cli as workspace member
- **PATTERN**: Follow existing workspace structure
- **IMPORTS**: Add `"cli"` to members array
- **GOTCHA**: Maintain workspace dependency consistency
- **VALIDATE**: `cargo check --workspace`

### CREATE amp/cli/src/main.rs

- **IMPLEMENT**: CLI entry point with clap commands
- **PATTERN**: Mirror server main.rs async structure
- **IMPORTS**: 
  ```rust
  use clap::{Parser, Subcommand};
  use anyhow::Result;
  
  #[derive(Parser)]
  #[command(name = "amp")]
  #[command(about = "AMP Bridge - Agentic Memory Protocol CLI")]
  struct Cli {
      #[command(subcommand)]
      command: Commands,
  }
  
  #[derive(Subcommand)]
  enum Commands {
      Start { agent: String },
      Status,
      History,
      Tui,
  }
  ```
- **GOTCHA**: Use tokio::main for async runtime
- **VALIDATE**: `cd amp/cli && cargo run -- --help`

### CREATE amp/cli/src/config.rs

- **IMPLEMENT**: CLI configuration management
- **PATTERN**: Mirror amp/server/src/config.rs structure
- **IMPORTS**: 
  ```rust
  use std::path::PathBuf;
  use anyhow::Result;
  use dirs;
  
  #[derive(Debug, Clone)]
  pub struct Config {
      pub server_url: String,
      pub session_dir: PathBuf,
      pub data_dir: PathBuf,
  }
  ```
- **GOTCHA**: Use dirs crate for cross-platform paths
- **VALIDATE**: Unit test config loading

### CREATE amp/cli/src/client.rs

- **IMPLEMENT**: HTTP client for AMP server communication
- **PATTERN**: Use reqwest with JSON serialization like server handlers
- **IMPORTS**: 
  ```rust
  use reqwest::Client;
  use serde_json::Value;
  use anyhow::Result;
  
  pub struct AmpClient {
      client: Client,
      base_url: String,
  }
  
  impl AmpClient {
      pub async fn create_object(&self, object: Value) -> Result<Value> {
          // POST /v1/objects
      }
      
      pub async fn query(&self, query: &str) -> Result<Value> {
          // POST /v1/query
      }
  }
  ```
- **GOTCHA**: Handle server connection errors gracefully
- **VALIDATE**: `cargo test client_tests`

### CREATE amp/cli/src/session.rs

- **IMPLEMENT**: Session lifecycle management
- **PATTERN**: Follow session model from amp-session-lifecycle.md
- **IMPORTS**: 
  ```rust
  use uuid::Uuid;
  use chrono::{DateTime, Utc};
  use serde::{Deserialize, Serialize};
  use std::path::PathBuf;
  
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Session {
      pub id: Uuid,
      pub run_id: Uuid,
      pub project_id: String,
      pub started_at: DateTime<Utc>,
      pub ended_at: Option<DateTime<Utc>>,
      pub status: SessionStatus,
      pub agent_command: String,
      pub lease_id: Uuid,
  }
  
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub enum SessionStatus {
      Active,
      Completed,
      Aborted,
      Disconnected,
  }
  ```
- **GOTCHA**: Implement session persistence to local files
- **VALIDATE**: Unit tests for session CRUD operations

### CREATE amp/cli/src/process.rs

- **IMPLEMENT**: Agent process spawning and monitoring
- **PATTERN**: Use tokio::process::Command for async process management
- **IMPORTS**: 
  ```rust
  use tokio::process::{Command, Child};
  use tokio::io::{AsyncBufReadExt, BufReader};
  use std::process::Stdio;
  use anyhow::Result;
  
  pub struct AgentProcess {
      child: Child,
      command: String,
  }
  
  impl AgentProcess {
      pub async fn spawn(command: &str) -> Result<Self> {
          let mut cmd = Command::new("sh");
          cmd.arg("-c").arg(command);
          cmd.stdout(Stdio::piped());
          cmd.stderr(Stdio::piped());
          // Implementation
      }
  }
  ```
- **GOTCHA**: Handle process cleanup on session end
- **VALIDATE**: Test process spawning and termination

### CREATE amp/cli/src/app.rs

- **IMPLEMENT**: Main TUI application state and event loop
- **PATTERN**: Use The Elm Architecture (TEA) pattern
- **IMPORTS**: 
  ```rust
  use ratatui::{
      backend::CrosstermBackend,
      Terminal,
      widgets::{Block, Borders, Paragraph},
      layout::{Layout, Constraint, Direction},
  };
  use crossterm::{
      event::{self, Event, KeyCode},
      terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
      ExecutableCommand,
  };
  
  pub struct App {
      pub should_quit: bool,
      pub current_session: Option<Session>,
  }
  
  impl App {
      pub fn new() -> Self {
          // Initialize app state
      }
      
      pub async fn run(&mut self) -> Result<()> {
          // Main event loop
      }
  }
  ```
- **GOTCHA**: Proper terminal cleanup on exit
- **VALIDATE**: `cargo run tui` shows basic interface

### CREATE amp/cli/src/ui/mod.rs

- **IMPLEMENT**: TUI component organization
- **PATTERN**: Modular UI components
- **IMPORTS**: 
  ```rust
  pub mod layout;
  pub mod session_view;
  pub mod status_bar;
  
  use ratatui::{Frame, widgets::*};
  
  pub fn render_main_layout(f: &mut Frame, app: &App) {
      // Main layout rendering
  }
  ```
- **GOTCHA**: Keep UI components stateless where possible
- **VALIDATE**: UI renders without panics

### CREATE amp/cli/src/commands/mod.rs

- **IMPLEMENT**: CLI command implementations
- **PATTERN**: One module per command
- **IMPORTS**: 
  ```rust
  pub mod start;
  pub mod status;
  pub mod history;
  pub mod tui;
  
  use crate::{Config, AmpClient};
  use anyhow::Result;
  ```
- **GOTCHA**: Each command should be independently testable
- **VALIDATE**: All commands compile and have basic tests

### UPDATE amp/cli/src/commands/start.rs

- **IMPLEMENT**: Session start command with agent launching
- **PATTERN**: Create session, spawn agent, monitor process
- **IMPORTS**: 
  ```rust
  use crate::{Session, AgentProcess, AmpClient};
  use anyhow::Result;
  
  pub async fn start_session(agent_command: &str, client: &AmpClient) -> Result<()> {
      // 1. Create session in AMP
      // 2. Spawn agent process
      // 3. Monitor and capture output
      // 4. Finalize session on exit
  }
  ```
- **GOTCHA**: Handle agent process cleanup on Ctrl+C
- **VALIDATE**: `amp start "echo hello"` creates and finalizes session

### UPDATE amp/cli/src/commands/tui.rs

- **IMPLEMENT**: Interactive TUI mode
- **PATTERN**: Launch full TUI application
- **IMPORTS**: 
  ```rust
  use crate::App;
  use anyhow::Result;
  
  pub async fn run_tui() -> Result<()> {
      let mut app = App::new();
      app.run().await
  }
  ```
- **GOTCHA**: Proper terminal state restoration
- **VALIDATE**: `amp tui` launches interactive interface

### ADD amp/cli/src/git.rs

- **IMPLEMENT**: Git diff capture for changeset creation
- **PATTERN**: Use git2 crate or git command execution
- **IMPORTS**: 
  ```rust
  use std::process::Command;
  use anyhow::Result;
  
  pub fn capture_diff() -> Result<String> {
      let output = Command::new("git")
          .args(&["diff", "--no-color"])
          .output()?;
      Ok(String::from_utf8(output.stdout)?)
  }
  ```
- **GOTCHA**: Handle repositories without git
- **VALIDATE**: Test diff capture in git repository

### UPDATE amp/cli/src/session.rs - Add Heartbeat

- **IMPLEMENT**: Lease heartbeat system
- **PATTERN**: Background tokio task with interval timer
- **IMPORTS**: 
  ```rust
  use tokio::time::{interval, Duration};
  use tokio::task::JoinHandle;
  
  impl Session {
      pub fn start_heartbeat(&self, client: AmpClient) -> JoinHandle<()> {
          let lease_id = self.lease_id;
          tokio::spawn(async move {
              let mut interval = interval(Duration::from_secs(30));
              loop {
                  interval.tick().await;
                  if let Err(e) = client.renew_lease(lease_id).await {
                      tracing::error!("Heartbeat failed: {}", e);
                      break;
                  }
              }
          })
      }
  }
  ```
- **GOTCHA**: Cancel heartbeat on session end
- **VALIDATE**: Heartbeat maintains session lease

### ADD Integration Tests

- **IMPLEMENT**: End-to-end integration tests
- **PATTERN**: Test full session lifecycle
- **IMPORTS**: 
  ```rust
  #[tokio::test]
  async fn test_session_lifecycle() {
      // Start session
      // Run simple agent
      // Verify session creation and finalization
  }
  ```
- **GOTCHA**: Tests need running AMP server
- **VALIDATE**: `cargo test --test integration`

---

## TESTING STRATEGY

### Unit Tests

Design unit tests with fixtures and assertions following existing testing approaches:

- **Session Management**: Test session creation, persistence, and state transitions
- **Process Management**: Test agent spawning, monitoring, and cleanup
- **HTTP Client**: Test AMP server communication with mock responses
- **Configuration**: Test config loading from environment and files
- **Git Integration**: Test diff capture and changeset creation

### Integration Tests

- **Full Session Lifecycle**: End-to-end test of session start → agent run → session end
- **TUI Interaction**: Test TUI components and event handling
- **Error Recovery**: Test session cleanup on agent crashes and disconnects
- **Concurrent Sessions**: Test multiple simultaneous sessions

### Edge Cases

- Agent process crashes or hangs
- Network disconnection during session
- Terminal resize during TUI operation
- Invalid git repositories
- AMP server unavailable
- Concurrent session conflicts

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
# Workspace-wide checks
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check --all

# CLI-specific checks
cd amp/cli
cargo check
cargo clippy -- -D warnings
```

### Level 2: Unit Tests

```bash
# All unit tests
cargo test --workspace --lib

# CLI unit tests
cd amp/cli
cargo test --lib
```

### Level 3: Integration Tests

```bash
# Start AMP server in background
cd amp/server && cargo run &
SERVER_PID=$!

# Run integration tests
cd amp/cli
cargo test --test integration

# Cleanup
kill $SERVER_PID
```

### Level 4: Manual Validation

```bash
# Test CLI help
amp --help
amp start --help

# Test basic session
amp start "echo 'Hello AMP'"

# Test TUI mode
amp tui

# Test session history
amp history

# Test status
amp status
```

### Level 5: Additional Validation

```bash
# Test with real coding agents (if available)
amp start "kiro-cli"
amp start "cursor"

# Test error conditions
amp start "nonexistent-command"
amp start "sleep 1000" # Then Ctrl+C

# Test concurrent sessions
amp start "sleep 5" &
amp start "sleep 5" &
wait
```

---

## ACCEPTANCE CRITERIA

- [ ] CLI provides transparent agent launching with `amp start <agent>`
- [ ] TUI mode provides interactive session management interface
- [ ] Sessions are automatically created and finalized with proper Run objects
- [ ] Agent output is captured and preserved in ChangeSet objects
- [ ] Lease-based heartbeat system prevents orphaned sessions
- [ ] Git diff capture works for changeset creation
- [ ] Process cleanup handles crashes and forced termination
- [ ] Configuration supports multiple AMP server environments
- [ ] All validation commands pass with zero errors
- [ ] Unit test coverage meets 80%+ requirement
- [ ] Integration tests verify end-to-end workflows
- [ ] TUI handles terminal resize and cleanup properly
- [ ] Error messages are clear and actionable
- [ ] Session history and status commands work correctly
- [ ] Concurrent session support works without conflicts

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in dependency order
- [ ] Each task validation passed immediately after implementation
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit + integration)
- [ ] No linting or type checking errors
- [ ] Manual testing with real agents confirms functionality
- [ ] TUI interface is responsive and user-friendly
- [ ] Session lifecycle management is robust and reliable
- [ ] Error handling covers all edge cases
- [ ] Documentation is complete and accurate
- [ ] Performance is acceptable for interactive use
- [ ] Memory usage is reasonable for long-running sessions

---

## NOTES

### Design Decisions

- **TUI Framework**: Chose ratatui + crossterm for mature, cross-platform terminal support
- **Architecture**: The Elm Architecture (TEA) for predictable state management
- **Process Management**: Tokio async processes for non-blocking agent monitoring
- **Session Storage**: Local file-based persistence for offline capability
- **HTTP Client**: Thin client over existing AMP API to maintain consistency

### Trade-offs

- **Complexity vs Features**: Balancing rich TUI features with implementation complexity
- **Performance vs Monitoring**: Process monitoring overhead vs session fidelity
- **Local vs Remote State**: Local session cache vs always querying AMP server
- **Terminal Compatibility**: Supporting various terminal emulators and capabilities

### Future Extensions

- **Multi-Agent Sessions**: Support for coordinated multi-agent workflows
- **Live Streaming**: Real-time session data streaming to AMP server
- **Plugin System**: Extensible agent integration beyond process spawning
- **Distributed Sessions**: Cross-machine session coordination
- **Advanced TUI**: Rich terminal features like mouse support and custom themes

### Security Considerations

- **Process Isolation**: Ensure agent processes cannot escape session boundaries
- **Credential Management**: Secure handling of AMP server authentication
- **File Permissions**: Proper session file permissions and cleanup
- **Network Security**: Secure communication with AMP server
