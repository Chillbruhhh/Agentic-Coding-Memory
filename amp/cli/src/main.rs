use clap::{Parser, Subcommand};
use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod config;
pub mod client;
pub mod session;
pub mod process;
pub mod app;
pub mod commands;
pub mod ui;
pub mod git;

use config::Config;
use client::AmpClient;

#[derive(Parser)]
#[command(name = "amp")]
#[command(about = "AMP Bridge - Agentic Memory Protocol CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show session history
    History,
    /// Index the current directory and create AMP memory objects
    Index {
        /// Directory to index (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
        /// Skip files matching these patterns
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,
    },
    /// Clear all objects from the AMP database
    Clear {
        /// Confirm the clear operation
        #[arg(long)]
        confirm: bool,
    },
    /// Start a new session with an agent
    Start { 
        /// Agent command to run
        agent: String 
    },
    /// Show current session status
    Status,
    /// Query objects and relationships from the AMP database
    Query {
        /// Query text to search for
        #[arg(short, long)]
        text: Option<String>,
        /// Show relationships
        #[arg(long)]
        relationships: bool,
    },
    /// Launch interactive TUI
    Tui,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    let config = Config::from_env()?;
    let client = AmpClient::new(&config.server_url);

    match cli.command {
        Commands::Clear { confirm } => {
            commands::clear::run_clear(confirm, &client).await?;
        }
        Commands::History => {
            commands::history::show_history(&client).await?;
        }
        Commands::Index { path, exclude } => {
            if should_run_index_in_container(&path)? {
                run_index_in_container(&path, &exclude)?;
            } else {
                commands::index::run_index(&path, &exclude, &client).await?;
            }
        }
        Commands::Query { text, relationships } => {
            commands::query::run_query(text.as_deref(), relationships, &client).await?;
        }
        Commands::Start { agent } => {
            commands::start::start_session(&agent, &client).await?;
        }
        Commands::Status => {
            commands::status::show_status(&client).await?;
        }
        Commands::Tui => {
            commands::tui::run_tui().await?;
        }
    }

    Ok(())
}

fn should_run_index_in_container(path: &str) -> Result<bool> {
    if env::var("AMP_INDEX_IN_CONTAINER").as_deref() == Ok("1") {
        return Ok(false);
    }
    if env::var("AMP_INDEX_CONTAINER").as_deref() == Ok("1") {
        return Ok(true);
    }
    if env::var("AMP_INDEX_CONTAINER").as_deref() == Ok("0") {
        return Ok(false);
    }
    if !cfg!(windows) {
        return Ok(false);
    }

    let compose_file = match find_compose_file(&env::current_dir()?) {
        Some(file) => file,
        None => return Ok(false),
    };

    let Some(compose_cmd) = detect_compose_command() else {
        return Ok(false);
    };

    let services = list_running_services(&compose_cmd, &compose_file)?;
    if services.iter().any(|s| s == "amp-server") {
        let abs_path = std::fs::canonicalize(path)?;
        let compose_root = compose_file
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Compose file has no parent directory"))?;
        if abs_path.starts_with(compose_root) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn run_index_in_container(path: &str, exclude: &[String]) -> Result<()> {
    let compose_file = find_compose_file(&env::current_dir()?)
        .ok_or_else(|| anyhow::anyhow!("docker-compose.yml not found"))?;
    let compose_root = compose_file
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Compose file has no parent directory"))?;

    let abs_path = std::fs::canonicalize(path)?;
    if !abs_path.starts_with(compose_root) {
        anyhow::bail!(
            "Path {} is outside the compose root {}; run AMP server locally or move into the repo.",
            abs_path.display(),
            compose_root.display()
        );
    }

    let container_path = to_container_path(compose_root, &abs_path);
    let Some(compose_cmd) = detect_compose_command() else {
        anyhow::bail!("Docker Compose not found in PATH");
    };

    let mut cmd = build_compose_command(&compose_cmd, &compose_file);
    cmd.arg("run")
        .arg("--rm")
        .arg("-e")
        .arg("AMP_INDEX_IN_CONTAINER=1")
        .arg("amp-cli")
        .arg("cargo")
        .arg("run")
        .arg("-p")
        .arg("amp")
        .arg("--")
        .arg("index")
        .arg(container_path);

    if !exclude.is_empty() {
        cmd.arg("--exclude").arg(exclude.join(","));
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Containerized index failed with status {}", status);
    }
    Ok(())
}

fn find_compose_file(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let candidate = dir.join("docker-compose.yml");
        if candidate.exists() {
            return Some(candidate);
        }
        current = dir.parent();
    }
    None
}

#[derive(Copy, Clone)]
enum ComposeCommand {
    Docker,
    DockerCompose,
}

fn detect_compose_command() -> Option<ComposeCommand> {
    if Command::new("docker")
        .args(["compose", "version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(ComposeCommand::Docker);
    }

    if Command::new("docker-compose")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some(ComposeCommand::DockerCompose);
    }

    None
}

fn build_compose_command(cmd: &ComposeCommand, compose_file: &Path) -> Command {
    let mut command = match cmd {
        ComposeCommand::Docker => {
            let mut c = Command::new("docker");
            c.arg("compose");
            c
        }
        ComposeCommand::DockerCompose => Command::new("docker-compose"),
    };

    command.arg("-f").arg(compose_file);
    command
}

fn list_running_services(cmd: &ComposeCommand, compose_file: &Path) -> Result<Vec<String>> {
    let output = build_compose_command(cmd, compose_file)
        .args(["ps", "--services", "--filter", "status=running"])
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect())
}

fn to_container_path(compose_root: &Path, abs_path: &Path) -> String {
    let relative = abs_path.strip_prefix(compose_root).unwrap_or(abs_path);
    let mut container_path = PathBuf::from("/app/amp");
    if !relative.as_os_str().is_empty() {
        container_path = container_path.join(relative);
    }
    container_path.to_string_lossy().replace('\\', "/")
}
