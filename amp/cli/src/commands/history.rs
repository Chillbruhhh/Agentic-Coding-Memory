use crate::{client::AmpClient, config::Config, session::Session};
use anyhow::Result;

pub async fn show_history(_client: &AmpClient) -> Result<()> {
    println!("AMP Session History");
    println!("==================");
    
    let config = Config::from_env()?;
    let sessions = Session::list_sessions(&config.session_dir).await?;
    
    if sessions.is_empty() {
        println!("No sessions found");
        return Ok(());
    }
    
    for session in sessions.iter().take(10) {
        let duration = if let Some(ended_at) = session.ended_at {
            let duration = ended_at - session.started_at;
            format!("{}s", duration.num_seconds())
        } else {
            "ongoing".to_string()
        };
        
        println!(
            "{} | {:?} | {} | {} | {}",
            session.started_at.format("%Y-%m-%d %H:%M:%S"),
            session.status,
            duration,
            session.agent_command,
            session.id
        );
    }
    
    if sessions.len() > 10 {
        println!("... and {} more sessions", sessions.len() - 10);
    }
    
    Ok(())
}
