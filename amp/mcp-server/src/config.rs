use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub amp_server_url: String,
    pub amp_server_timeout: u64,
    pub server_name: String,
    pub server_version: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            amp_server_url: env::var("AMP_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:8105".to_string()),
            amp_server_timeout: env::var("AMP_SERVER_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .context("Invalid AMP_SERVER_TIMEOUT")?,
            server_name: env::var("MCP_SERVER_NAME")
                .unwrap_or_else(|_| "amp-mcp-server".to_string()),
            server_version: env::var("MCP_SERVER_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
        })
    }
}
