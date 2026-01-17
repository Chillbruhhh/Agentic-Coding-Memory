# Implementation Plan: Vector Embeddings Integration

**Feature**: Vector embedding generation and semantic search
**Estimated Time**: 90-120 minutes
**Priority**: High (Core value proposition - semantic memory retrieval)

## Overview

Integrate vector embeddings into AMP to enable semantic search. Support both OpenAI and Ollama as embedding providers, allowing users to choose between cloud-based or local embedding generation.

## Current State

- ✅ Database schema has `embedding` field (Vec<f32>, 1536 dimensions)
- ✅ SurrealDB MTREE vector index defined in schema
- ✅ Query endpoint exists with text search
- ❌ No embedding generation service
- ❌ No vector similarity search in query endpoint
- ❌ Embedding field always null in objects

## Architecture Design

### Embedding Service Interface

```rust
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
    fn dimension(&self) -> usize;
}
```

### Supported Providers

1. **OpenAI** (cloud-based)
   - Model: `text-embedding-3-small` (1536 dimensions)
   - Requires: `OPENAI_API_KEY` environment variable
   - Fast, high quality, costs ~$0.02 per 1M tokens

2. **Ollama** (local)
   - Model: `nomic-embed-text` (768 dimensions) or configurable
   - Requires: Ollama running locally (default: http://localhost:11434)
   - Free, private, slower

3. **None** (disabled)
   - No embedding generation
   - Vector search disabled
   - Useful for testing or when embeddings not needed

### Configuration

Add to `config.rs`:
```rust
pub struct Config {
    // ... existing fields
    pub embedding_provider: EmbeddingProvider,
    pub openai_api_key: Option<String>,
    pub ollama_url: String,
    pub embedding_dimension: usize,
}

pub enum EmbeddingProvider {
    OpenAI,
    Ollama,
    None,
}
```

Environment variables:
- `EMBEDDING_PROVIDER` - "openai", "ollama", or "none" (default: "none")
- `OPENAI_API_KEY` - OpenAI API key (required if provider=openai)
- `OLLAMA_URL` - Ollama endpoint (default: "http://localhost:11434")
- `EMBEDDING_DIMENSION` - Vector dimensions (default: 1536 for OpenAI, 768 for Ollama)

## Implementation Strategy

### Phase 1: Embedding Service Infrastructure (30 minutes)

**Step 1.1: Create embedding service module**
- File: `amp/server/src/services/embedding.rs`
- Define `EmbeddingService` trait
- Define `EmbeddingError` enum
- Create factory function to instantiate correct provider

**Step 1.2: Implement OpenAI provider**
- File: `amp/server/src/services/embedding/openai.rs`
- Use `reqwest` to call OpenAI API
- Endpoint: `https://api.openai.com/v1/embeddings`
- Model: `text-embedding-3-small`
- Handle rate limits and errors

**Step 1.3: Implement Ollama provider**
- File: `amp/server/src/services/embedding/ollama.rs`
- Use `reqwest` to call Ollama API
- Endpoint: `{OLLAMA_URL}/api/embeddings`
- Model: `nomic-embed-text` (configurable)
- Handle connection errors

**Step 1.4: Implement None provider**
- File: `amp/server/src/services/embedding/none.rs`
- Returns error if called
- Used when embeddings disabled

### Phase 2: Automatic Embedding Generation (30 minutes)

**Step 2.1: Update object creation handlers**
- Modify `create_object()` in `handlers/objects.rs`
- Generate embedding from object content before storing
- Content extraction logic per object type:
  - Symbol: `name + signature + documentation`
  - Decision: `title + problem + rationale + outcome`
  - ChangeSet: `title + description`
  - Run: `input_summary + outputs`

**Step 2.2: Update batch creation handler**
- Modify `create_objects_batch()` to generate embeddings
- Generate embeddings in parallel for performance
- Continue on embedding errors (log warning, store null)

**Step 2.3: Update update handler**
- Regenerate embedding when object is updated
- Use same content extraction logic

### Phase 3: Vector Similarity Search (30 minutes)

**Step 3.1: Update query endpoint**
- Modify `query()` in `handlers/query.rs`
- If `request.vector` is provided, use vector search
- If `request.text` is provided without vector, generate embedding from text
- Combine vector search with existing filters

**Step 3.2: Implement vector search query**
- Use SurrealDB's vector search syntax:
  ```sql
  SELECT *, vector::similarity::cosine(embedding, $query_vector) AS similarity
  FROM objects
  WHERE embedding IS NOT NULL
  ORDER BY similarity DESC
  LIMIT 10
  ```

**Step 3.3: Update scoring logic**
- Combine vector similarity score with text match score
- Hybrid scoring: `final_score = (vector_score * 0.7) + (text_score * 0.3)`
- Update explanations to show vector similarity

### Phase 4: Testing (20 minutes)

**Step 4.1: Create embedding test script**
- File: `amp/scripts/test-embeddings.ps1`
- Test with OpenAI (if API key available)
- Test with Ollama (if running locally)
- Test with embeddings disabled

**Step 4.2: Create vector search test script**
- File: `amp/scripts/test-vector-search.ps1`
- Create objects with embeddings
- Query by text (auto-generate embedding)
- Query by vector (provide embedding directly)
- Verify semantic similarity works

## Detailed Implementation

### File: `amp/server/src/services/embedding.rs`

```rust
use async_trait::async_trait;
use thiserror::Error;

pub mod openai;
pub mod ollama;
pub mod none;

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
    ollama_url: String,
    dimension: usize,
) -> Box<dyn EmbeddingService> {
    match provider.to_lowercase().as_str() {
        "openai" => {
            if let Some(api_key) = openai_api_key {
                Box::new(openai::OpenAIEmbedding::new(api_key))
            } else {
                tracing::warn!("OpenAI provider selected but no API key provided, using None");
                Box::new(none::NoneEmbedding)
            }
        }
        "ollama" => Box::new(ollama::OllamaEmbedding::new(ollama_url, dimension)),
        _ => Box::new(none::NoneEmbedding),
    }
}
```

### File: `amp/server/src/services/embedding/openai.rs`

```rust
use super::{EmbeddingError, EmbeddingService};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OpenAIEmbedding {
    client: Client,
    api_key: String,
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
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl EmbeddingService for OpenAIEmbedding {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let request = EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
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
```

### File: `amp/server/src/services/embedding/ollama.rs`

```rust
use super::{EmbeddingError, EmbeddingService};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct OllamaEmbedding {
    client: Client,
    url: String,
    dimension: usize,
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
    pub fn new(url: String, dimension: usize) -> Self {
        Self {
            client: Client::new(),
            url,
            dimension,
        }
    }
}

#[async_trait]
impl EmbeddingService for OllamaEmbedding {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let request = OllamaRequest {
            model: "nomic-embed-text".to_string(),
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
```

### File: `amp/server/src/services/embedding/none.rs`

```rust
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
```

### Update: `amp/server/src/config.rs`

```rust
// Add to Config struct
pub embedding_provider: String,
pub openai_api_key: Option<String>,
pub ollama_url: String,
pub embedding_dimension: usize,

// Add to from_env()
embedding_provider: env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "none".to_string()),
openai_api_key: env::var("OPENAI_API_KEY").ok(),
ollama_url: env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
embedding_dimension: env::var("EMBEDDING_DIMENSION")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(1536),
```

### Update: `amp/server/src/main.rs`

```rust
// Add to AppState
pub embedding_service: Arc<dyn EmbeddingService>,

// In main(), after config load
let embedding_service = Arc::new(services::embedding::create_embedding_service(
    &config.embedding_provider,
    config.openai_api_key.clone(),
    config.ollama_url.clone(),
    config.embedding_dimension,
));

let state = AppState { 
    db, 
    config: config.clone(),
    embedding_service,
};
```

### Update: `amp/server/src/handlers/objects.rs`

Add helper function:
```rust
async fn generate_embedding_for_object(
    obj: &AmpObject,
    embedding_service: &Arc<dyn EmbeddingService>,
) -> Option<Vec<f32>> {
    if !embedding_service.is_enabled() {
        return None;
    }

    let text = extract_text_for_embedding(obj);
    
    match embedding_service.generate_embedding(&text).await {
        Ok(embedding) => Some(embedding),
        Err(e) => {
            tracing::warn!("Failed to generate embedding: {}", e);
            None
        }
    }
}

fn extract_text_for_embedding(obj: &AmpObject) -> String {
    match obj {
        AmpObject::Symbol(s) => {
            format!(
                "{} {} {}",
                s.name,
                s.signature.as_deref().unwrap_or(""),
                s.documentation.as_deref().unwrap_or("")
            )
        }
        AmpObject::Decision(d) => {
            format!(
                "{} {} {} {}",
                d.title, d.problem, d.rationale, d.outcome
            )
        }
        AmpObject::ChangeSet(c) => {
            format!(
                "{} {}",
                c.title,
                c.description.as_deref().unwrap_or("")
            )
        }
        AmpObject::Run(r) => {
            format!(
                "{} {}",
                r.input_summary.as_deref().unwrap_or(""),
                r.outputs.as_deref().unwrap_or("")
            )
        }
    }
}
```

Update `create_object()`:
```rust
// After deserializing payload, before storing
let embedding = generate_embedding_for_object(&payload, &state.embedding_service).await;

// Update payload with embedding
// (This requires making the payload mutable and updating the embedding field)
```

### Update: `amp/server/src/handlers/query.rs`

Add vector search support:
```rust
// In query() function, after building text query
let query_vector = if let Some(vector) = &request.vector {
    Some(vector.clone())
} else if let Some(text) = &request.text {
    // Generate embedding from text query
    if state.embedding_service.is_enabled() {
        match state.embedding_service.generate_embedding(text).await {
            Ok(vec) => Some(vec),
            Err(e) => {
                tracing::warn!("Failed to generate query embedding: {}", e);
                None
            }
        }
    } else {
        None
    }
} else {
    None
};

// If we have a vector, use vector search
if let Some(vector) = query_vector {
    // Build vector search query
    let vector_str = vector.iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    
    query = format!(
        "SELECT *, vector::similarity::cosine(embedding, [{}]) AS similarity FROM objects WHERE embedding IS NOT NULL",
        vector_str
    );
    
    // Add filters
    // ... (same filter logic as before)
    
    query.push_str(" ORDER BY similarity DESC");
} else {
    // Use existing text search query
    // ... (existing logic)
}
```

Update scoring to include vector similarity:
```rust
fn calculate_score(obj: &Value, text_query: Option<&String>, has_vector: bool) -> f32 {
    if has_vector {
        // Use similarity from query result
        if let Some(similarity) = obj.get("similarity").and_then(|v| v.as_f64()) {
            return similarity as f32;
        }
    }
    
    // Fall back to text scoring
    // ... (existing logic)
}
```

### File: `amp/scripts/test-embeddings.ps1`

```powershell
Write-Host "=== AMP Embedding Generation Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"

# Check which provider is configured
Write-Host "Testing embedding generation..." -ForegroundColor Yellow
Write-Host "Note: Set EMBEDDING_PROVIDER=openai or ollama to enable embeddings" -ForegroundColor Gray
Write-Host ""

# Create object with embedding
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$symbol = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "embedding_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test embedding generation"
    },
    "links": [],
    "embedding": null,
    "name": "authenticate_user",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "signature": "fn authenticate_user(username: &str, password: &str) -> Result<User>",
    "documentation": "Authenticates a user with username and password using bcrypt"
}
"@

Write-Host "1. Creating object (embedding should be auto-generated)..." -ForegroundColor Yellow
$response = Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol -ContentType "application/json"
$objectId = $response.id
Write-Host "Created object: $objectId" -ForegroundColor Green
Write-Host ""

Write-Host "2. Retrieving object to check embedding..." -ForegroundColor Yellow
$retrieved = Invoke-RestMethod -Uri "$baseUrl/v1/objects/$objectId" -Method Get

if ($retrieved.embedding -ne $null) {
    $embeddingLength = $retrieved.embedding.Length
    Write-Host "✅ Embedding generated! Dimension: $embeddingLength" -ForegroundColor Green
    Write-Host "First 5 values: $($retrieved.embedding[0..4] -join ', ')" -ForegroundColor Gray
} else {
    Write-Host "⚠️  No embedding generated (provider may be disabled)" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
```

### File: `amp/scripts/test-vector-search.ps1`

```powershell
Write-Host "=== AMP Vector Search Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# Create test objects
Write-Host "1. Creating test objects..." -ForegroundColor Yellow

$objects = @(
    @{
        name = "authenticate_user"
        doc = "Authenticates a user with username and password"
    },
    @{
        name = "hash_password"
        doc = "Hashes a password using bcrypt for secure storage"
    },
    @{
        name = "send_email"
        doc = "Sends an email notification to a user"
    }
)

foreach ($obj in $objects) {
    $symbol = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "vector_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test vector search"
    },
    "links": [],
    "embedding": null,
    "name": "$($obj.name)",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "signature": "fn $($obj.name)()",
    "documentation": "$($obj.doc)"
}
"@
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol -ContentType "application/json" | Out-Null
}

Write-Host "Created 3 test objects" -ForegroundColor Green
Write-Host ""

# Test semantic search
Write-Host "2. Testing semantic search for 'user login security'..." -ForegroundColor Yellow
$query = @{
    text = "user login security"
    filters = @{
        project_id = "vector_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 10

$result = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $query -ContentType "application/json"
Write-Host "Found $($result.total_count) results in $($result.execution_time_ms)ms" -ForegroundColor Green

if ($result.total_count -gt 0) {
    Write-Host ""
    Write-Host "Results (should rank authentication higher than email):" -ForegroundColor Gray
    $result.results | ForEach-Object {
        Write-Host "  - $($_.object.name) (score: $($_.score))" -ForegroundColor Gray
        Write-Host "    $($_.explanation)" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
```

## Testing Checklist

- [ ] OpenAI provider generates embeddings (with API key)
- [ ] Ollama provider generates embeddings (with local Ollama)
- [ ] None provider works (embeddings disabled)
- [ ] Embeddings auto-generated on object creation
- [ ] Embeddings regenerated on object update
- [ ] Vector search returns semantically similar results
- [ ] Text query auto-generates embedding for search
- [ ] Hybrid scoring combines vector and text scores
- [ ] Explanations show vector similarity
- [ ] Dimension validation works

## Success Criteria

1. ✅ Embeddings automatically generated for all objects
2. ✅ Support for both OpenAI and Ollama providers
3. ✅ Vector similarity search works
4. ✅ Semantic search finds related objects
5. ✅ Graceful degradation when embeddings disabled
6. ✅ Test scripts validate all functionality

## Dependencies

Add to `Cargo.toml`:
```toml
async-trait = "0.1"
thiserror = "1.0"
```

## Configuration Examples

**OpenAI:**
```bash
export EMBEDDING_PROVIDER=openai
export OPENAI_API_KEY=sk-...
export EMBEDDING_DIMENSION=1536
```

**Ollama:**
```bash
export EMBEDDING_PROVIDER=ollama
export OLLAMA_URL=http://localhost:11434
export EMBEDDING_DIMENSION=768
```

**Disabled:**
```bash
export EMBEDDING_PROVIDER=none
```

## Notes

- OpenAI embeddings are 1536 dimensions, Ollama nomic-embed-text is 768
- SurrealDB vector index needs to match embedding dimension
- Embedding generation adds ~100-500ms per object (OpenAI) or ~50-200ms (Ollama local)
- Consider batch embedding generation for performance
- Vector search is more expensive than text search (full table scan with similarity calculation)
- Cosine similarity returns values 0-1 (1 = identical, 0 = orthogonal)

## Future Enhancements

1. **Batch Embedding Generation**: Generate embeddings for multiple objects in one API call
2. **Embedding Caching**: Cache embeddings for common queries
3. **Multiple Models**: Support different embedding models per use case
4. **Dimension Reduction**: Support for smaller embedding dimensions
5. **Hybrid Search**: Combine vector, text, and graph scores intelligently

## Estimated Timeline

- Embedding service infrastructure: 30 minutes
- OpenAI provider: 15 minutes
- Ollama provider: 15 minutes
- Auto-generation in handlers: 30 minutes
- Vector search in query: 30 minutes
- Test scripts: 20 minutes
- Testing and debugging: 30 minutes

**Total**: 170 minutes (~2.8 hours with buffer)
