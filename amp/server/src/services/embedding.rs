use async_trait::async_trait;
use thiserror::Error;

pub mod none;
pub mod ollama;
pub mod openai;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Embeddings disabled")]
    Disabled,
}

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    fn dimension(&self) -> usize;
    fn is_enabled(&self) -> bool;
}

pub fn create_embedding_service(
    provider: &str,
    openai_api_key: Option<String>,
    openrouter_api_key: Option<String>,
    ollama_url: String,
    dimension: usize,
    model: String,
) -> Box<dyn EmbeddingService> {
    match provider.to_lowercase().as_str() {
        "openai" => {
            if let Some(api_key) = openai_api_key {
                Box::new(openai::OpenAIEmbedding::new(
                    api_key,
                    model,
                    "https://api.openai.com/v1".to_string(),
                    dimension,
                ))
            } else {
                tracing::warn!("OpenAI provider selected but no API key provided, using None");
                Box::new(none::NoneEmbedding)
            }
        }
        "openrouter" => {
            if let Some(api_key) = openrouter_api_key {
                Box::new(openai::OpenAIEmbedding::new(
                    api_key,
                    model,
                    "https://openrouter.ai/api/v1".to_string(),
                    dimension,
                ))
            } else {
                tracing::warn!("OpenRouter provider selected but no API key provided, using None");
                Box::new(none::NoneEmbedding)
            }
        }
        "ollama" => Box::new(ollama::OllamaEmbedding::new(ollama_url, dimension, model)),
        _ => Box::new(none::NoneEmbedding),
    }
}
