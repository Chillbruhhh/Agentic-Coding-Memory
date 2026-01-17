use super::{EmbeddingError, EmbeddingService};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaEmbedding {
    client: Client,
    url: String,
    dimension: usize,
    model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    embedding: Vec<f32>,
}

impl OllamaEmbedding {
    pub fn new(url: String, dimension: usize, model: String) -> Self {
        Self {
            client: Client::new(),
            url,
            dimension,
            model,
        }
    }
}

#[async_trait]
impl EmbeddingService for OllamaEmbedding {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/api/embeddings", self.url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(EmbeddingError::ApiError(error_text));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.embedding)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn is_enabled(&self) -> bool {
        true
    }
}
