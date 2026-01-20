# FileChunk & FileLog Integration Plan

**Objective**: Integrate FileChunk and FileLog object types into AMP to enable full-coverage semantic search and compressed file summaries alongside existing Symbol-based indexing.

**Status**: Planning  
**Priority**: High  
**Estimated Time**: 6-8 hours

---

## Current State Analysis

### What We Have ✅
- **Symbol Objects**: Functions, classes, methods with embeddings
- **File Objects**: Basic file metadata (path, language, hash)
- **Graph Relationships**: `defined_in` connecting symbols to files
- **Codebase Parser**: Tree-sitter based parsing for Python/TypeScript
- **Vector Embeddings**: OpenAI/Ollama integration with auto-generation
- **Hybrid Retrieval**: Text + Vector + Graph search

### What's Missing ❌
- **FileChunk Objects**: Chunked file content for full semantic coverage
- **FileLog Objects**: Semantic summaries of files
- **Chunking Logic**: ~500 token chunks with optional overlap
- **Incremental Updates**: Hash-based change detection for chunks
- **Enhanced Retrieval**: Multi-layer vector search (Symbols + Chunks + Logs)

---

## Architecture Overview

```
Repository
    ↓
Codebase Parser
    ↓
┌─────────────┬──────────────┬─────────────┐
│   Symbols   │  FileChunks  │  FileLogs   │
│  (existing) │    (new)     │    (new)    │
└─────────────┴──────────────┴─────────────┘
         ↓            ↓             ↓
    Embeddings   Embeddings    Embeddings
         ↓            ↓             ↓
      SurrealDB Vector Index (1536 dims)
         ↓            ↓             ↓
    Hybrid Retrieval Engine
```

---

## Implementation Tasks

### Phase 1: Data Models & Schema (1.5 hours)

#### Task 1.1: Add FileChunk Object Type
**File**: `amp/server/src/models/mod.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    // Base fields (inherited)
    pub id: String,
    pub type_: String, // "FileChunk"
    pub tenant_id: String,
    pub project_id: String,
    pub created_at: String,
    pub updated_at: String,
    
    // FileChunk-specific fields
    pub file_path: String,
    pub chunk_index: u32,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: u32,
    pub content: String,
    pub content_hash: String,
    pub language: String,
    pub embedding: Option<Vec<f32>>,
    
    // Relationships
    pub file_id: String, // Parent file
}
```

#### Task 1.2: Add FileLog Object Type
**File**: `amp/server/src/models/mod.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLog {
    // Base fields
    pub id: String,
    pub type_: String, // "FileLog"
    pub tenant_id: String,
    pub project_id: String,
    pub created_at: String,
    pub updated_at: String,
    
    // FileLog-specific fields
    pub file_path: String,
    pub file_id: String,
    pub summary: String, // Markdown format
    pub purpose: Option<String>,
    pub key_symbols: Vec<String>,
    pub dependencies: Vec<String>,
    pub notes: Option<String>,
    pub embedding: Option<Vec<f32>>,
    
    // Change tracking
    pub last_modified: String,
    pub change_count: u32,
    pub linked_changesets: Vec<String>,
}
```

#### Task 1.3: Update SurrealDB Schema
**File**: `amp/spec/schema.surql`

```sql
-- FileChunk table
DEFINE TABLE file_chunks SCHEMAFULL;
DEFINE FIELD file_path ON file_chunks TYPE string;
DEFINE FIELD chunk_index ON file_chunks TYPE int;
DEFINE FIELD start_line ON file_chunks TYPE int;
DEFINE FIELD end_line ON file_chunks TYPE int;
DEFINE FIELD token_count ON file_chunks TYPE int;
DEFINE FIELD content ON file_chunks TYPE string;
DEFINE FIELD content_hash ON file_chunks TYPE string;
DEFINE FIELD language ON file_chunks TYPE string;
DEFINE FIELD embedding ON file_chunks TYPE array;
DEFINE FIELD file_id ON file_chunks TYPE record<objects>;

-- FileLog table
DEFINE TABLE file_logs SCHEMAFULL;
DEFINE FIELD file_path ON file_logs TYPE string;
DEFINE FIELD file_id ON file_logs TYPE record<objects>;
DEFINE FIELD summary ON file_logs TYPE string;
DEFINE FIELD purpose ON file_logs TYPE option<string>;
DEFINE FIELD key_symbols ON file_logs TYPE array<string>;
DEFINE FIELD dependencies ON file_logs TYPE array<string>;
DEFINE FIELD notes ON file_logs TYPE option<string>;
DEFINE FIELD embedding ON file_logs TYPE array;
DEFINE FIELD last_modified ON file_logs TYPE datetime;
DEFINE FIELD change_count ON file_logs TYPE int;
DEFINE FIELD linked_changesets ON file_logs TYPE array<string>;

-- Indexes
DEFINE INDEX file_chunks_path ON file_chunks FIELDS file_path;
DEFINE INDEX file_chunks_hash ON file_chunks FIELDS content_hash;
DEFINE INDEX file_logs_path ON file_logs FIELDS file_path;
```

#### Task 1.4: Update OpenAPI Spec
**File**: `amp/spec/openapi.yaml`

Add FileChunk and FileLog schemas to components/schemas section.

---

### Phase 2: Chunking Service (2 hours)

#### Task 2.1: Create Chunking Service
**File**: `amp/server/src/services/chunking.rs`

```rust
pub struct ChunkingService {
    chunk_size: usize,      // ~500 tokens
    overlap_size: usize,    // 50-100 tokens
}

impl ChunkingService {
    pub fn chunk_file(&self, content: &str, language: &str) -> Vec<FileChunkData> {
        // 1. Tokenize content (approximate with whitespace split)
        // 2. Create chunks of ~500 tokens
        // 3. Add overlap between chunks
        // 4. Compute line ranges for each chunk
        // 5. Generate content hash per chunk
        // 6. Return chunk metadata
    }
    
    pub fn estimate_token_count(&self, text: &str) -> usize {
        // Rough estimate: words * 1.3
        text.split_whitespace().count() * 13 / 10
    }
    
    pub fn compute_chunk_hash(&self, content: &str) -> String {
        // SHA256 hash of chunk content
    }
}
```

#### Task 2.2: Integrate into Codebase Parser
**File**: `amp/server/src/services/codebase_parser.rs`

```rust
// Add chunking after symbol extraction
pub async fn parse_file(&self, path: &Path) -> Result<ParseResult> {
    // ... existing symbol extraction ...
    
    // NEW: Chunk file content
    let content = fs::read_to_string(path)?;
    let chunks = self.chunking_service.chunk_file(&content, language);
    
    // Create FileChunk objects
    for (idx, chunk_data) in chunks.iter().enumerate() {
        let chunk = FileChunk {
            file_path: path.to_string(),
            chunk_index: idx as u32,
            content: chunk_data.content.clone(),
            content_hash: chunk_data.hash.clone(),
            // ... other fields
        };
        result.chunks.push(chunk);
    }
    
    Ok(result)
}
```

---

### Phase 3: FileLog Generation (1.5 hours)

#### Task 3.1: Create FileLog Generator
**File**: `amp/server/src/services/filelog_generator.rs`

```rust
pub struct FileLogGenerator;

impl FileLogGenerator {
    pub fn generate_summary(&self, file: &File, symbols: &[Symbol]) -> String {
        // Generate Markdown summary
        format!(
            "# {}\n\n## Purpose\n{}\n\n## Symbols\n{}\n\n## Dependencies\n{}",
            file.path,
            self.infer_purpose(file, symbols),
            self.format_symbols(symbols),
            self.format_dependencies(symbols)
        )
    }
    
    fn infer_purpose(&self, file: &File, symbols: &[Symbol]) -> String {
        // Heuristic: use first docstring or class/function name
        // For MVP: simple concatenation
        // Post-MVP: LLM-based summarization
    }
    
    fn format_symbols(&self, symbols: &[Symbol]) -> String {
        symbols.iter()
            .map(|s| format!("- `{}` ({})", s.name, s.kind))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

#### Task 3.2: Integrate into Parser
**File**: `amp/server/src/services/codebase_parser.rs`

```rust
// Add FileLog generation after symbols and chunks
pub async fn parse_file(&self, path: &Path) -> Result<ParseResult> {
    // ... symbols and chunks ...
    
    // NEW: Generate FileLog
    let file_log = self.filelog_generator.generate_summary(&file, &symbols);
    result.file_log = Some(file_log);
    
    Ok(result)
}
```

---

### Phase 4: Incremental Updates (1.5 hours)

#### Task 4.1: Hash-Based Change Detection
**File**: `amp/server/src/services/codebase_parser.rs`

```rust
pub async fn update_file(&self, path: &Path) -> Result<UpdateResult> {
    // 1. Compute new file hash
    let new_hash = compute_file_hash(path)?;
    
    // 2. Query existing file from database
    let existing_file = self.db.get_file_by_path(path).await?;
    
    // 3. Compare hashes
    if existing_file.content_hash == new_hash {
        return Ok(UpdateResult::NoChange);
    }
    
    // 4. Re-parse file
    let parse_result = self.parse_file(path).await?;
    
    // 5. Update chunks incrementally
    let existing_chunks = self.db.get_chunks_by_file(path).await?;
    for chunk in parse_result.chunks {
        let existing = existing_chunks.iter()
            .find(|c| c.chunk_index == chunk.chunk_index);
        
        if let Some(old_chunk) = existing {
            if old_chunk.content_hash != chunk.content_hash {
                // Re-embed only changed chunks
                self.update_chunk(chunk).await?;
            }
        } else {
            // New chunk
            self.create_chunk(chunk).await?;
        }
    }
    
    // 6. Remove orphaned chunks
    self.cleanup_orphaned_chunks(path, &parse_result.chunks).await?;
    
    Ok(UpdateResult::Updated)
}
```

---

### Phase 5: Enhanced Retrieval (1.5 hours)

#### Task 5.1: Multi-Layer Vector Search
**File**: `amp/server/src/services/hybrid.rs`

```rust
// Update hybrid retrieval to search across all layers
pub async fn hybrid_search(&self, query: &QueryRequest) -> Result<Vec<QueryResult>> {
    let (symbol_results, chunk_results, log_results) = tokio::try_join!(
        self.search_symbols(&query),
        self.search_chunks(&query),    // NEW
        self.search_file_logs(&query)  // NEW
    )?;
    
    // Merge and deduplicate results
    let merged = self.merge_multi_layer_results(
        symbol_results,
        chunk_results,
        log_results
    );
    
    Ok(merged)
}

fn merge_multi_layer_results(&self, 
    symbols: Vec<QueryResult>,
    chunks: Vec<QueryResult>,
    logs: Vec<QueryResult>
) -> Vec<QueryResult> {
    // Weighted scoring:
    // - Symbols: 40% (high precision)
    // - Chunks: 35% (full coverage)
    // - FileLogs: 25% (navigation)
    
    // Deduplicate by file_path
    // Combine scores for same file
}
```

#### Task 5.2: Update Query Handler
**File**: `amp/server/src/handlers/query.rs`

```rust
// Add support for filtering by object type
pub async fn query_objects(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>> {
    // Allow filtering: type=["Symbol", "FileChunk", "FileLog"]
    let results = if request.hybrid {
        state.hybrid_service.hybrid_search(&request).await?
    } else {
        // ... existing logic
    };
    
    Ok(Json(QueryResponse { results }))
}
```

---

### Phase 6: CLI Integration (1 hour)

#### Task 6.1: Update Index Command
**File**: `amp/cli/src/commands/index.rs`

```rust
// Update to create FileChunks and FileLogs
pub async fn index_directory(&self, path: &Path) -> Result<()> {
    for file in files {
        // Parse file (now includes chunks and logs)
        let parse_result = self.parser.parse_file(&file).await?;
        
        // Create File object
        self.create_file_object(&parse_result.file).await?;
        
        // Create Symbol objects
        for symbol in parse_result.symbols {
            self.create_symbol_object(&symbol).await?;
        }
        
        // NEW: Create FileChunk objects
        for chunk in parse_result.chunks {
            self.create_chunk_object(&chunk).await?;
        }
        
        // NEW: Create FileLog object
        if let Some(log) = parse_result.file_log {
            self.create_filelog_object(&log).await?;
        }
        
        // Create relationships
        self.create_relationships(&parse_result).await?;
    }
}
```

---

### Phase 7: Testing & Validation (1 hour)

#### Task 7.1: Create Test Scripts
**Files**: 
- `amp/scripts/test-chunking.ps1`
- `amp/scripts/test-filelog.ps1`
- `amp/scripts/test-incremental-update.ps1`

```powershell
# test-chunking.ps1
# 1. Index a large file (>1000 tokens)
# 2. Verify multiple chunks created
# 3. Verify embeddings generated
# 4. Query chunks by content
# 5. Verify retrieval works

# test-filelog.ps1
# 1. Index a file with multiple symbols
# 2. Verify FileLog created
# 3. Verify summary contains key symbols
# 4. Query by FileLog embedding
# 5. Verify navigation works

# test-incremental-update.ps1
# 1. Index a file
# 2. Modify file content
# 3. Re-index
# 4. Verify only changed chunks re-embedded
# 5. Verify orphaned chunks removed
```

#### Task 7.2: Integration Tests
**File**: `amp/server/tests/integration/chunking_tests.rs`

```rust
#[tokio::test]
async fn test_file_chunking() {
    // Test chunking logic
}

#[tokio::test]
async fn test_filelog_generation() {
    // Test FileLog creation
}

#[tokio::test]
async fn test_incremental_updates() {
    // Test hash-based updates
}

#[tokio::test]
async fn test_multi_layer_retrieval() {
    // Test hybrid search across all layers
}
```

---

## Success Criteria

### Functional Requirements ✅
- [ ] FileChunk objects created for files >500 tokens
- [ ] FileLog objects created with semantic summaries
- [ ] Chunks have embeddings and are searchable
- [ ] FileLogs have embeddings and are searchable
- [ ] Incremental updates only re-embed changed chunks
- [ ] Hybrid retrieval searches all three layers
- [ ] CLI index command creates all object types
- [ ] Graph relationships connect chunks to files

### Performance Requirements ✅
- [ ] Chunking adds <10% overhead to indexing time
- [ ] Incremental updates 5x faster than full re-index
- [ ] Multi-layer search completes in <2 seconds
- [ ] Memory usage stays under 500MB for 10k chunks

### Quality Requirements ✅
- [ ] All tests passing
- [ ] No duplicate chunks created
- [ ] Orphaned chunks properly cleaned up
- [ ] Embeddings generated for all chunks/logs
- [ ] Documentation updated

---

## Risk Mitigation

### Risk 1: Token Estimation Accuracy
**Impact**: Medium  
**Mitigation**: Use conservative estimates (words * 1.3), validate with real tokenizer post-MVP

### Risk 2: Embedding Cost
**Impact**: High  
**Mitigation**: 
- Batch embedding requests
- Cache embeddings by content hash
- Use Ollama for development

### Risk 3: Database Performance
**Impact**: Medium  
**Mitigation**:
- Index chunk_hash for fast lookups
- Limit chunk retrieval to top 20 results
- Use pagination for large result sets

### Risk 4: Incremental Update Complexity
**Impact**: Medium  
**Mitigation**:
- Start with simple hash comparison
- Add sophisticated diffing post-MVP
- Log all update operations for debugging

---

## Post-MVP Enhancements

1. **Smart Chunking**: Use AST boundaries instead of token counts
2. **LLM Summarization**: Use LLM to generate FileLog summaries
3. **Chunk Overlap Tuning**: Optimize overlap size based on retrieval metrics
4. **Cross-File Context**: Link chunks that reference same symbols
5. **Semantic Diff**: Embed diffs between file versions
6. **Visualization**: Show chunk coverage in UI

---

## Timeline

| Phase | Tasks | Time | Dependencies |
|-------|-------|------|--------------|
| 1 | Data Models & Schema | 1.5h | None |
| 2 | Chunking Service | 2h | Phase 1 |
| 3 | FileLog Generation | 1.5h | Phase 1 |
| 4 | Incremental Updates | 1.5h | Phase 2, 3 |
| 5 | Enhanced Retrieval | 1.5h | Phase 2, 3 |
| 6 | CLI Integration | 1h | Phase 2, 3, 4 |
| 7 | Testing & Validation | 1h | All phases |

**Total Estimated Time**: 6-8 hours

---

## Implementation Order

1. **Day 1 (3-4 hours)**: Phases 1-3 (Models, Chunking, FileLog)
2. **Day 2 (3-4 hours)**: Phases 4-7 (Updates, Retrieval, CLI, Testing)

---

## Notes

- This plan builds on existing Symbol-based indexing
- Maintains backward compatibility with current system
- Follows the strategy outlined in `AMP_Codebase_Embedding_and_Graph_Strategy.md`
- Prioritizes MVP delivery while enabling future enhancements
- All new code follows existing Rust patterns and error handling
