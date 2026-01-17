use crate::{session::Session, process::AgentProcess, client::AmpClient, config::Config, git};
use anyhow::Result;
use serde_json::json;
use tokio::signal;

pub async fn start_session(agent_command: &str, client: &AmpClient) -> Result<()> {
    println!("Starting AMP session with agent: {}", agent_command);
    
    // Check server health
    if !client.health_check().await? {
        anyhow::bail!("AMP server is not available");
    }
    
    let config = Config::from_env()?;
    
    // Create session
    let mut session = Session::new(agent_command.to_string(), "default".to_string());
    
    // Acquire lease
    let pid = std::process::id();
    client.acquire_lease(session.lease_id, pid).await?;
    
    // Start heartbeat
    let heartbeat_handle = session.start_heartbeat(client.clone());
    
    // Save session
    session.save_to_file(&config.session_dir).await?;
    
    // Create Run object in AMP
    let run_object = json!({
        "id": session.run_id,
        "type": "Run",
        "tenant_id": "default",
        "project_id": session.project_id,
        "created_at": session.started_at.to_rfc3339(),
        "updated_at": session.started_at.to_rfc3339(),
        "provenance": {
            "source": "amp-cli",
            "version": "0.1.0"
        },
        "input_summary": format!("Agent command: {}", agent_command),
        "status": "running"
    });
    
    client.create_object(run_object).await?;
    
    println!("Session {} started", session.id);
    
    // Spawn agent process
    let mut process = AgentProcess::spawn(agent_command).await?;
    
    // Setup Ctrl+C handler
    let session_id = session.id;
    let lease_id = session.lease_id;
    let client_clone = client.clone();
    let config_clone = config.clone();
    
    tokio::select! {
        exit_code = process.wait_for_completion() => {
            match exit_code {
                Ok(0) => {
                    println!("Agent completed successfully");
                    session.complete();
                }
                Ok(code) => {
                    println!("Agent exited with code: {}", code);
                    session.abort();
                }
                Err(e) => {
                    println!("Agent process error: {}", e);
                    session.abort();
                }
            }
        }
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, terminating session...");
            let _ = process.kill().await;
            session.abort();
        }
    }
    
    // Finalize session
    finalize_session(session, &client_clone, &config_clone).await?;
    
    // Cancel heartbeat
    heartbeat_handle.abort();
    
    // Release lease
    client.release_lease(lease_id).await?;
    
    println!("Session {} finalized", session_id);
    
    Ok(())
}

async fn finalize_session(session: Session, client: &AmpClient, config: &Config) -> Result<()> {
    // Capture git diff if available
    let diff = git::capture_diff().unwrap_or_default();
    
    // Update Run object
    let run_update = json!({
        "id": session.run_id,
        "type": "Run",
        "tenant_id": "default",
        "project_id": session.project_id,
        "created_at": session.started_at.to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339(),
        "provenance": {
            "source": "amp-cli",
            "version": "0.1.0"
        },
        "input_summary": format!("Agent command: {}", session.agent_command),
        "status": match session.status {
            crate::session::SessionStatus::Completed => "completed",
            crate::session::SessionStatus::Aborted => "aborted",
            crate::session::SessionStatus::Disconnected => "disconnected",
            _ => "unknown"
        },
        "outputs": {
            "diff": diff
        }
    });
    
    client.create_object(run_update).await?;
    
    // Create ChangeSet if there are changes
    if !diff.is_empty() {
        let changeset = json!({
            "id": uuid::Uuid::new_v4(),
            "type": "ChangeSet",
            "tenant_id": "default",
            "project_id": session.project_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339(),
            "provenance": {
                "source": "amp-cli",
                "version": "0.1.0"
            },
            "title": format!("Changes from session {}", session.id),
            "description": format!("Automated capture from agent: {}", session.agent_command),
            "diff": diff,
            "status": "applied"
        });
        
        client.create_object(changeset).await?;
    }
    
    // Save final session state with updated status
    session.save_to_file(&config.session_dir).await?;
    
    Ok(())
}
