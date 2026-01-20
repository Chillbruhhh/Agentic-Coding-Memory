use crate::models::settings::SettingsConfig;
use anyhow::Result;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use std::env;

pub struct SettingsService {
    db: Surreal<Any>,
}

impl SettingsService {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Load settings from database or environment variables
    pub async fn load_settings(&self) -> Result<SettingsConfig> {
        // Try to load from database first
        match self.load_from_db().await {
            Ok(settings) => Ok(settings),
            Err(_) => {
                // Fall back to environment variables
                Ok(self.load_from_env())
            }
        }
    }

    /// Load settings from database
    async fn load_from_db(&self) -> Result<SettingsConfig> {
        let result: Option<SettingsConfig> = self.db
            .select(("settings", "config"))
            .await?;
        
        result.ok_or_else(|| anyhow::anyhow!("Settings not found in database"))
    }

    /// Load settings from environment variables
    fn load_from_env(&self) -> SettingsConfig {
        SettingsConfig {
            port: env::var("PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8105),
            bind_address: env::var("BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "ws://localhost:7505/rpc".to_string()),
            db_user: env::var("DB_USER")
                .unwrap_or_else(|_| "root".to_string()),
            db_pass: env::var("DB_PASS")
                .unwrap_or_else(|_| "root".to_string()),
            embedding_provider: env::var("EMBEDDING_PROVIDER")
                .unwrap_or_else(|_| "none".to_string()),
            openai_api_key: env::var("OPENAI_API_KEY")
                .unwrap_or_default(),
            openai_model: env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
            openai_dimension: env::var("EMBEDDING_DIMENSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1536),
            openrouter_api_key: env::var("OPENROUTER_API_KEY")
                .unwrap_or_default(),
            openrouter_model: env::var("OPENROUTER_EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
            openrouter_dimension: env::var("OPENROUTER_EMBEDDING_DIMENSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1536),
            ollama_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".to_string()),
            ollama_dimension: env::var("EMBEDDING_DIMENSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(768),
            index_provider: env::var("INDEX_PROVIDER")
                .unwrap_or_else(|_| "none".to_string()),
            index_openai_model: env::var("INDEX_OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            index_openrouter_model: env::var("INDEX_OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openai/gpt-4o-mini".to_string()),
            index_ollama_model: env::var("INDEX_OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama3.1".to_string()),
            index_workers: env::var("INDEX_WORKERS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            max_embedding_dimension: env::var("MAX_EMBEDDING_DIMENSION")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1536),
        }
    }

    /// Save settings to database
    pub async fn save_settings(&self, settings: SettingsConfig) -> Result<SettingsConfig> {
        let saved: Option<SettingsConfig> = self.db
            .upsert(("settings", "config"))
            .content(settings)
            .await?;

        saved.ok_or_else(|| anyhow::anyhow!("Failed to save settings"))
    }
}
