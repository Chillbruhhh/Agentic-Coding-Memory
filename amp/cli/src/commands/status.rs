use crate::{client::AmpClient, config::Config, session::Session};
use anyhow::Result;

pub async fn show_status(client: &AmpClient) -> Result<()> {
    println!("AMP Bridge Status");
    println!("================");
    
    // Check server health
    match client.health_check().await {
        Ok(true) => println!("✓ AMP Server: Connected"),
        Ok(false) => println!("✗ AMP Server: Disconnected"),
        Err(e) => println!("✗ AMP Server: Error - {}", e),
    }
    
    // Check for active sessions
    let config = Config::from_env()?;
    println!("Session directory: {:?}", config.session_dir);
    let sessions = Session::list_sessions(&config.session_dir).await?;
    
    let active_sessions: Vec<_> = sessions.iter()
        .filter(|s| matches!(s.status, crate::session::SessionStatus::Active))
        .collect();
    
    println!("Active Sessions: {}", active_sessions.len());
    
    for session in active_sessions {
        println!("  - {} ({})", session.id, session.agent_command);
    }
    
    println!("Total Sessions: {}", sessions.len());
    
    Ok(())
}
