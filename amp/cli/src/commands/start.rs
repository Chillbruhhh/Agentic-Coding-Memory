use crate::{session::Session, process::AgentProcess, client::AmpClient, config::Config, git};
use anyhow::Result;
use serde_json::json;
use tokio::signal;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant};

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
    
    let capture_output = should_capture_output();
    if capture_output {
        println!("AMP cache capture enabled (piped stdout/stderr). If the agent needs a TTY, set AMP_CAPTURE_AGENT_OUTPUT=0.");
    }

    // Spawn agent process
    let mut process = AgentProcess::spawn(agent_command, capture_output).await?;
    let cache_task = if capture_output {
        start_cache_capture(&mut process, &session, client.clone())
    } else {
        None
    };
    
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

    if let Some(handle) = cache_task {
        handle.abort();
    }
    
    // Release lease
    client.release_lease(lease_id).await?;
    
    println!("Session {} finalized", session_id);
    
    Ok(())
}

fn should_capture_output() -> bool {
    match std::env::var("AMP_CAPTURE_AGENT_OUTPUT") {
        Ok(value) => matches!(value.as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

fn start_cache_capture(
    process: &mut AgentProcess,
    session: &Session,
    client: AmpClient,
) -> Option<tokio::task::JoinHandle<()>> {
    let stdout = process.take_stdout()?;
    let stderr = process.take_stderr();
    let scope_id = format!("project:{}", session.project_id);
    let agent_label = session.agent_command.clone();

    let (tx, mut rx) = mpsc::channel::<String>(256);
    let tx_stdout = tx.clone();
    let tx_stderr = tx.clone();

    let stdout_handle = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_stdout.send(line.clone()).await;
            println!("{}", line);
        }
    });

    let stderr_handle = stderr.map(|stderr| {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = tx_stderr.send(format!("stderr: {}", line.clone())).await;
                eprintln!("{}", line);
            }
        })
    });

    Some(tokio::spawn(async move {
        let mut buffer = String::new();
        let mut ticker = interval(Duration::from_secs(5));
        let mut last_flush = Instant::now();

        loop {
            tokio::select! {
                line = rx.recv() => {
                    match line {
                        Some(line) => {
                            if !buffer.is_empty() {
                                buffer.push('\n');
                            }
                            buffer.push_str(&line);
                            if buffer.len() >= 2000 {
                                flush_cache_chunk(&client, &scope_id, &agent_label, &buffer).await;
                                buffer.clear();
                                last_flush = Instant::now();
                            }
                        }
                        None => {
                            if !buffer.is_empty() {
                                flush_cache_chunk(&client, &scope_id, &agent_label, &buffer).await;
                                buffer.clear();
                            }
                            break;
                        }
                    }
                }
                _ = ticker.tick() => {
                    if !buffer.is_empty() && last_flush.elapsed() >= Duration::from_secs(5) {
                        flush_cache_chunk(&client, &scope_id, &agent_label, &buffer).await;
                        buffer.clear();
                        last_flush = Instant::now();
                    } else if buffer.is_empty() && rx.is_closed() {
                        break;
                    }
                }
            }
        }

        stdout_handle.abort();
        if let Some(handle) = stderr_handle {
            handle.abort();
        }
    }))
}

async fn flush_cache_chunk(
    client: &AmpClient,
    scope_id: &str,
    agent_label: &str,
    content: &str,
) {
    let preview = content.chars().take(200).collect::<String>();
    let payload = json!({
        "scope_id": scope_id,
        "items": [{
            "kind": "snippet",
            "preview": format!("{}: {}", agent_label, preview),
            "facts": [],
            "importance": 0.4
        }]
    });

    if let Err(err) = client.cache_write_items(payload).await {
        tracing::warn!("Cache write failed: {}", err);
    }
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
