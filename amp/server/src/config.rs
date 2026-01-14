use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub embedding_service_url: Option<String>,
    pub max_embedding_dimension: usize,
    pub port: u16,
    pub bind_address: String,
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
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "memory".to_string()),
            embedding_service_url: env::var("EMBEDDING_SERVICE_URL").ok(),
            max_embedding_dimension,
            port,
            bind_address: env::var("BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
        })
    }
}
