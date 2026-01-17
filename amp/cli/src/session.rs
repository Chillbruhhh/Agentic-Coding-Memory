use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use tokio::time::{interval, Duration};
use tokio::task::JoinHandle;

use crate::client::AmpClient;

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

impl Session {
    pub fn new(agent_command: String, project_id: String) -> Self {
        let id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        let lease_id = Uuid::new_v4();
        
        Self {
            id,
            run_id,
            project_id,
            started_at: Utc::now(),
            ended_at: None,
            status: SessionStatus::Active,
            agent_command,
            lease_id,
        }
    }

    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
        self.ended_at = Some(Utc::now());
    }

    pub fn abort(&mut self) {
        self.status = SessionStatus::Aborted;
        self.ended_at = Some(Utc::now());
    }

    pub fn disconnect(&mut self) {
        self.status = SessionStatus::Disconnected;
        self.ended_at = Some(Utc::now());
    }

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

    pub async fn save_to_file(&self, session_dir: &PathBuf) -> Result<()> {
        let file_path = session_dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(file_path, json).await?;
        Ok(())
    }

    pub async fn load_from_file(session_dir: &PathBuf, session_id: Uuid) -> Result<Self> {
        let file_path = session_dir.join(format!("{}.json", session_id));
        let json = tokio::fs::read_to_string(file_path).await?;
        let session: Session = serde_json::from_str(&json)?;
        Ok(session)
    }

    pub async fn list_sessions(session_dir: &PathBuf) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();
        let mut entries = tokio::fs::read_dir(session_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Ok(json) = tokio::fs::read_to_string(entry.path()).await {
                        if let Ok(session) = serde_json::from_str::<Session>(&json) {
                            sessions.push(session);
                        }
                    }
                }
            }
        }
        
        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_session_creation() {
        let session = Session::new("echo hello".to_string(), "test-project".to_string());
        assert_eq!(session.agent_command, "echo hello");
        assert_eq!(session.project_id, "test-project");
        assert!(matches!(session.status, SessionStatus::Active));
    }

    #[tokio::test]
    async fn test_session_persistence() {
        let dir = tempdir().unwrap();
        let session_dir = dir.path().to_path_buf();
        
        let mut session = Session::new("test".to_string(), "project".to_string());
        session.save_to_file(&session_dir).await.unwrap();
        
        let loaded = Session::load_from_file(&session_dir, session.id).await.unwrap();
        assert_eq!(session.id, loaded.id);
        assert_eq!(session.agent_command, loaded.agent_command);
    }
}
