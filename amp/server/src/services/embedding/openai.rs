use super::{EmbeddingError, EmbeddingService};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIEmbedding {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

impl OpenAIEmbedding {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[async_trait]
impl EmbeddingService for OpenAIEmbedding {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let request = EmbeddingRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(EmbeddingError::ApiError(error_text));
        }

        let embedding_response: EmbeddingResponse = response.json().await?;
        
        embedding_response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| EmbeddingError::InvalidResponse("No embedding in response".to_string()))
    }

    fn dimension(&self) -> usize {
        1536
    }

    fn is_enabled(&self) -> bool {
        true
    }
}
