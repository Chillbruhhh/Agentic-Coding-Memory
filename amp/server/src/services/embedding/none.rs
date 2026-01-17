use super::{EmbeddingError, EmbeddingService};
use async_trait::async_trait;

pub struct NoneEmbedding;

#[async_trait]
impl EmbeddingService for NoneEmbedding {
    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Err(EmbeddingError::Disabled)
    }

    fn dimension(&self) -> usize {
        0
    }

    fn is_enabled(&self) -> bool {
        false
    }
}
