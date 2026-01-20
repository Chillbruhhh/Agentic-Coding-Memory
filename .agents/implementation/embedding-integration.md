# Embedding Integration Implementation

## Problem
The CLI creates FileChunks and FileLogs but **embeddings are not generated** because:
1. CLI uses single object endpoint (`POST /v1/objects`)
2. Single endpoint does NOT call `apply_embedding()`
3. Only batch endpoint (`POST /v1/objects/batch`) generates embeddings

## Solution
Batch all objects per file and use the batch endpoint which automatically generates embeddings.

## Changes Made

### 1. Client - Added Batch Method
**File**: `amp/cli/src/client.rs`

```rust
pub async fn batch_create_objects(&self, objects: Vec<Value>) -> Result<Value> {
    let response = self.client
        .post(&format!("{}/v1/objects/batch", self.base_url))
        .json(&objects)
        .send()
        .await?;
    
    if response.status().is_success() || response.status().as_u16() == 207 {
        Ok(response.json().await?)
    } else {
        anyhow::bail!("Failed to batch create objects: {}", response.status())
    }
}
```

### 2. Indexer - Refactored to Batch
**File**: `amp/cli/src/commands/index.rs`

**New Flow**:
```rust
async fn process_file_hierarchical(...) -> Result<usize> {
    let mut batch = Vec::new();
    
    // 1. Create file node object (don't send yet)
    let file_node = create_file_node_object(...)?;
    batch.push(file_node);
    
    // 2. Parse symbols (don't send yet)
    let symbols = parse_file_symbols(...).await?;
    batch.extend(symbols.clone());
    
    // 3. Create chunks (don't send yet)
    let chunks = create_file_chunks_objects(...)?;
    batch.extend(chunks);
    
    // 4. Create file log (don't send yet)
    let file_log = create_file_log_object(...)?;
    batch.push(file_log);
    
    // 5. Batch create ALL objects (server generates embeddings)
    client.batch_create_objects(batch).await?;
    
    // 6. Create relationships
    // ...
}
```

**New Helper Functions**:
- `create_file_node_object()` - Returns Value instead of sending
- `parse_file_symbols()` - Returns Vec<Value> instead of sending
- `create_file_chunks_objects()` - Returns Vec<Value> instead of sending
- `create_file_log_object()` - Returns Value instead of sending

## Server Behavior (Already Implemented)

**Batch Endpoint** (`POST /v1/objects/batch`):
```rust
pub async fn create_objects_batch(...) -> Result<...> {
    for obj in payload {
        // ✅ Generates embeddings automatically
        let obj = apply_embedding(&state, obj).await;
        // ... insert into DB
    }
}
```

**apply_embedding()** function:
1. Checks if embedding service is enabled
2. Extracts text from object (name, content, summary, etc.)
3. Calls embedding service (OpenAI/Ollama)
4. Sets embedding field on object
5. Returns object with embedding

## Configuration Required

**Server** (`.env`):
```bash
# Enable embedding provider
EMBEDDING_PROVIDER=openai  # or ollama

# OpenAI
OPENAI_API_KEY=sk-your-key
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_DIMENSION=1536

# OR Ollama
OLLAMA_URL=http://localhost:11434
EMBEDDING_MODEL=nomic-embed-text
EMBEDDING_DIMENSION=768
```

## Testing

1. **Start server with embeddings enabled**:
   ```bash
   cd amp/server
   export EMBEDDING_PROVIDER=openai
   export OPENAI_API_KEY=sk-...
   cargo run
   ```

2. **Index a codebase**:
   ```bash
   cd amp/cli
   cargo run -- index /path/to/project
   ```

3. **Verify embeddings are stored**:
   ```bash
   curl http://localhost:8105/v1/objects/{id} | jq '.embedding'
   # Should return array of floats, not null
   ```

## Benefits

✅ **Automatic embeddings** - Server generates them during batch create
✅ **Reduced API calls** - One batch request instead of N individual requests
✅ **Better performance** - Fewer network round trips
✅ **Consistent behavior** - All objects get embeddings if service is enabled
✅ **Semantic search ready** - FileChunks, FileLogs, and Symbols all have embeddings

## Next Steps

1. Compile and test the CLI changes
2. Verify embeddings are generated and stored
3. Test hybrid retrieval with vector search
4. Update documentation with embedding configuration
