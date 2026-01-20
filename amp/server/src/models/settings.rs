use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsConfig {
    // Server Settings
    pub port: u16,
    pub bind_address: String,
    
    // Database Settings
    pub database_url: String,
    pub db_user: String,
    pub db_pass: String,
    
    // Embedding Provider
    pub embedding_provider: String, // "openai", "openrouter", "ollama", or "none"
    
    // OpenAI Settings
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_dimension: u32,

    // OpenRouter Settings
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openrouter_dimension: u32,
    
    // Ollama Settings
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_dimension: u32,

    // Index Model Settings
    pub index_provider: String, // "openai", "openrouter", "ollama", or "none"
    pub index_openai_model: String,
    pub index_openrouter_model: String,
    pub index_ollama_model: String,
    pub index_workers: u32,
    
    // Legacy
    pub max_embedding_dimension: u32,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            port: 8105,
            bind_address: "127.0.0.1".to_string(),
            database_url: "ws://localhost:7505/rpc".to_string(),
            db_user: "root".to_string(),
            db_pass: "root".to_string(),
            embedding_provider: "none".to_string(),
            openai_api_key: String::new(),
            openai_model: "text-embedding-3-small".to_string(),
            openai_dimension: 1536,
            openrouter_api_key: String::new(),
            openrouter_model: "text-embedding-3-small".to_string(),
            openrouter_dimension: 1536,
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "nomic-embed-text".to_string(),
            ollama_dimension: 768,
            index_provider: "none".to_string(),
            index_openai_model: "gpt-4o-mini".to_string(),
            index_openrouter_model: "openai/gpt-4o-mini".to_string(),
            index_ollama_model: "llama3.1".to_string(),
            index_workers: 4,
            max_embedding_dimension: 1536,
        }
    }
}
