use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct ChunkData {
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: u32,
    pub hash: String,
}

pub struct ChunkingService {
    chunk_size: usize,
    overlap_size: usize,
}

impl ChunkingService {
    pub fn new() -> Self {
        Self {
            chunk_size: 500,
            overlap_size: 100, // 100-token overlap for better retrieval
        }
    }

    /// Create with custom settings
    pub fn with_settings(chunk_size: usize, overlap_size: usize) -> Self {
        Self {
            chunk_size,
            overlap_size,
        }
    }

    pub fn chunk_file(&self, content: &str, _language: &str) -> Vec<ChunkData> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return vec![];
        }

        let tokens: Vec<&str> = content.split_whitespace().collect();
        let total_tokens = self.estimate_token_count(content);

        if total_tokens <= self.chunk_size {
            return vec![ChunkData {
                content: content.to_string(),
                start_line: 1,
                end_line: lines.len() as u32,
                token_count: total_tokens as u32,
                hash: self.compute_hash(content),
            }];
        }

        let mut chunks = Vec::new();
        let mut start_idx = 0;

        while start_idx < tokens.len() {
            let end_idx = (start_idx + self.chunk_size).min(tokens.len());
            let chunk_tokens = &tokens[start_idx..end_idx];
            let chunk_content = chunk_tokens.join(" ");

            let (start_line, end_line) =
                self.estimate_line_range(content, start_idx, end_idx, &tokens);

            chunks.push(ChunkData {
                content: chunk_content.clone(),
                start_line,
                end_line,
                token_count: chunk_tokens.len() as u32,
                hash: self.compute_hash(&chunk_content),
            });

            start_idx = if end_idx < tokens.len() {
                end_idx - self.overlap_size
            } else {
                break;
            };
        }

        chunks
    }

    pub fn estimate_token_count(&self, text: &str) -> usize {
        text.split_whitespace().count() * 13 / 10
    }

    fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn estimate_line_range(
        &self,
        content: &str,
        start_token: usize,
        end_token: usize,
        tokens: &[&str],
    ) -> (u32, u32) {
        let start_pos = tokens[..start_token]
            .iter()
            .map(|t| t.len() + 1)
            .sum::<usize>();
        let end_pos = tokens[..end_token]
            .iter()
            .map(|t| t.len() + 1)
            .sum::<usize>();

        let start_line = content[..start_pos.min(content.len())]
            .lines()
            .count()
            .max(1) as u32;
        let end_line = content[..end_pos.min(content.len())].lines().count().max(1) as u32;

        (start_line, end_line)
    }
}

impl Default for ChunkingService {
    fn default() -> Self {
        Self::new()
    }
}
