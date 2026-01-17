use clap::{Parser, Subcommand};
use anyhow::Result;

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
            commands::index::run_index(&path, &exclude, &client).await?;
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
