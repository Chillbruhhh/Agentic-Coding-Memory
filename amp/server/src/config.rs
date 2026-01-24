use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub embedding_service_url: Option<String>,
    pub max_embedding_dimension: usize,
    pub port: u16,
    pub bind_address: String,
    pub embedding_provider: String,
    pub openai_api_key: Option<String>,
    pub ollama_url: String,
    pub embedding_dimension: usize,
    pub embedding_model: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "8105".to_string())
            .parse()?;

        if port == 0 {
            anyhow::bail!("PORT must be greater than 0");
        }

        let max_embedding_dimension: usize = env::var("MAX_EMBEDDING_DIMENSION")
            .unwrap_or_else(|_| "1536".to_string())
            .parse()?;

        if max_embedding_dimension == 0 || max_embedding_dimension > 10000 {
            anyhow::bail!("MAX_EMBEDDING_DIMENSION must be between 1 and 10000");
        }

        Ok(Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "memory".to_string()),
            embedding_service_url: env::var("EMBEDDING_SERVICE_URL").ok(),
            max_embedding_dimension,
            port,
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
            embedding_provider: env::var("EMBEDDING_PROVIDER")
                .unwrap_or_else(|_| "none".to_string()),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            ollama_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            embedding_dimension: env::var("EMBEDDING_DIMENSION")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1536),
            embedding_model: env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".to_string()),
        })
    }
}
