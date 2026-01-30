#![allow(dead_code)]
pub mod cache;
pub mod coordination;
pub mod focus;
pub mod discovery;
pub mod files;
pub mod memory;
pub mod query;

use anyhow::Result;

pub async fn register_tools(_handler: &impl rmcp::ServerHandler) -> Result<()> {
    // Tools are registered via the ServerHandler trait implementation
    // This function is kept for future manual registration if needed
    Ok(())
}
