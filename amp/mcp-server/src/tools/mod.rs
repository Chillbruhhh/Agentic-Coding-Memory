pub mod context;
pub mod query;
pub mod memory;
pub mod files;
pub mod coordination;
pub mod discovery;

use anyhow::Result;

pub async fn register_tools(_handler: &impl rmcp::ServerHandler) -> Result<()> {
    // Tools are registered via the ServerHandler trait implementation
    // This function is kept for future manual registration if needed
    Ok(())
}
