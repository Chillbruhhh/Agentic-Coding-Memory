# Feature: Hybrid Retrieval System

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Implement a unified hybrid retrieval system that combines text search, vector similarity search, and graph traversal into a single intelligent query endpoint. The system will execute multiple search methods in parallel, merge results with intelligent scoring, and provide comprehensive explanations for result ranking. This enhances the existing query endpoint to provide more relevant and comprehensive results by leveraging all available search modalities.

## User Story

As an AI agent developer using AMP
I want to query the memory system using natural language, semantic similarity, and relationship context simultaneously
So that I can retrieve the most relevant information regardless of how it's best matched (keyword, meaning, or connections)

## Problem Statement

The current AMP query system operates in silos - text search, vector search, and graph traversal are separate query paths that cannot be combined. This forces users to choose a single search method and potentially miss relevant results that would be found by other methods. Additionally, there's no intelligent ranking system that considers relevance across different search modalities.

## Solution Statement

Implement a hybrid retrieval system that:
1. Executes text, vector, and graph queries in parallel using Rust async patterns
2. Merges results with intelligent scoring that considers multiple relevance signals
3. Provides comprehensive explanations for why each result was returned
4. Maintains backward compatibility with existing single-method queries
5. Optimizes performance through structured concurrency and result deduplication

## Feature Metadata

**Feature Type**: Enhancement
**Estimated Complexity**: High
**Primary Systems Affected**: Query handler, Result scoring, Response formatting
**Dependencies**: tokio (async runtime), existing text/vector/graph search implementations

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/handlers/query.rs` (lines 1-600) - Why: Contains existing query logic, request/response structures, and separate search implementations
- `amp/server/src/services/graph.rs` (lines 1-800) - Why: Multi-hop graph traversal service that needs integration
- `amp/server/src/services/embedding.rs` (lines 1-50) - Why: Embedding service interface for vector search
- `amp/server/src/surreal_json.rs` (lines 1-100) - Why: JSON normalization utilities used in query responses
- `amp/server/src/main.rs` (lines 1-150) - Why: AppState structure with service dependencies
- `amp/server/Cargo.toml` - Why: Dependencies including tokio, serde, uuid for async patterns

### New Files to Create

- `amp/server/src/services/hybrid.rs` - Hybrid retrieval service with parallel execution and result merging
- `amp/scripts/test-hybrid-retrieval.ps1` - Comprehensive test script for hybrid queries
- `amp/examples/hybrid_query_examples.surql` - Documentation examples for hybrid queries

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [SurrealDB Vector Search](https://surrealdb.com/docs/surrealql/functions/vector) - Vector similarity functions and indexing
- [Tokio Structured Concurrency](https://docs.rs/tokio/latest/tokio/macro.try_join.html) - Parallel query execution patterns
- [Rust Async Patterns](https://rust-lang.github.io/async-book/) - Async/await best practices for database queries

### Patterns to Follow

**Async Parallel Execution:**
```rust
let (text_results, vector_results, graph_results) = tokio::try_join!(
    execute_text_search(&query),
    execute_vector_search(&query), 
    execute_graph_search(&query)
)?;
```

**Error Handling Pattern:**
```rust
#[derive(Debug, Error)]
pub enum HybridRetrievalError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Query timeout")]
    Timeout,
}
```

**Service Integration Pattern:**
```rust
pub struct HybridRetrievalService {
    db: Arc<Database>,
    embedding_service: Arc<dyn EmbeddingService>,
    graph_service: Arc<GraphTraversalService>,
}
```

**Result Scoring Pattern:**
```rust
#[derive(Debug, Serialize)]
pub struct HybridResult {
    pub object: Value,
    pub total_score: f32,
    pub text_score: Option<f32>,
    pub vector_score: Option<f32>,
    pub graph_score: Option<f32>,
    pub explanation: String,
}
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Create the hybrid retrieval service infrastructure with parallel query execution capabilities and result merging logic.

**Tasks:**
- Create HybridRetrievalService with async parallel query execution
- Implement result deduplication and scoring algorithms
- Add comprehensive error handling for multi-query scenarios

### Phase 2: Core Implementation

Integrate hybrid retrieval into the existing query handler with backward compatibility and intelligent query routing.

**Tasks:**
- Extend QueryRequest to support hybrid query parameters
- Implement parallel execution of text, vector, and graph searches
- Create intelligent result merging with multi-modal scoring
- Add comprehensive explanation generation

### Phase 3: Integration

Connect the hybrid service to the existing query endpoint and ensure seamless operation with current functionality.

**Tasks:**
- Integrate HybridRetrievalService into AppState
- Update query handler routing logic for hybrid queries
- Maintain backward compatibility for single-method queries
- Add performance monitoring and timeout handling

### Phase 4: Testing & Validation

Create comprehensive tests that validate hybrid retrieval functionality and performance characteristics.

**Tasks:**
- Implement unit tests for parallel query execution
- Create integration tests for hybrid query scenarios
- Add performance benchmarks for query execution times
- Validate result relevance and scoring accuracy

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### CREATE amp/server/src/services/hybrid.rs

- **IMPLEMENT**: HybridRetrievalService with parallel query execution using tokio::try_join!
- **PATTERN**: Service structure from `amp/server/src/services/graph.rs:15-25`
- **IMPORTS**: `std::sync::Arc, tokio::try_join, serde::{Serialize, Deserialize}, uuid::Uuid, serde_json::Value, thiserror::Error`
- **GOTCHA**: Use Arc<Database> for shared access, handle timeout errors from individual queries
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### CREATE HybridRetrievalError enum

- **IMPLEMENT**: Comprehensive error types for multi-query scenarios (DatabaseError, Timeout, InvalidQuery, PartialFailure)
- **PATTERN**: Error handling from `amp/server/src/services/graph.rs:10-20`
- **IMPORTS**: `thiserror::Error`
- **GOTCHA**: Include context for which specific query failed in partial failure scenarios
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### CREATE HybridResult and HybridResponse structures

- **IMPLEMENT**: Result structures with multi-modal scoring (text_score, vector_score, graph_score, total_score)
- **PATTERN**: QueryResult structure from `amp/server/src/handlers/query.rs:45-55`
- **IMPORTS**: `serde::{Serialize, Deserialize}, serde_json::Value, uuid::Uuid`
- **GOTCHA**: Include optional scores for each search method, comprehensive explanation field
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### IMPLEMENT execute_hybrid_query method

- **IMPLEMENT**: Parallel execution of text, vector, and graph queries using tokio::try_join!
- **PATTERN**: Parallel async execution from research findings (structured concurrency)
- **IMPORTS**: `tokio::time::{timeout, Duration}, std::collections::HashMap`
- **GOTCHA**: Handle partial failures gracefully, apply 5-second timeout to entire hybrid operation
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### IMPLEMENT result deduplication logic

- **IMPLEMENT**: Deduplicate results by object ID across different search methods
- **PATTERN**: HashSet usage from `amp/server/src/services/graph.rs:50-60`
- **IMPORTS**: `std::collections::{HashMap, HashSet}`
- **GOTCHA**: Preserve highest score when same object found by multiple methods
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### IMPLEMENT intelligent scoring algorithm

- **IMPLEMENT**: Multi-modal scoring that combines text, vector, and graph relevance scores
- **PATTERN**: Score calculation from `amp/server/src/handlers/query.rs:400-450`
- **IMPORTS**: None additional
- **GOTCHA**: Weight scores appropriately (vector: 0.4, text: 0.3, graph: 0.3), normalize to 0-1 range
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### IMPLEMENT comprehensive explanation generation

- **IMPLEMENT**: Generate detailed explanations showing why each result was returned and how it was scored
- **PATTERN**: Explanation generation from `amp/server/src/handlers/query.rs:500-550`
- **IMPORTS**: None additional
- **GOTCHA**: Include specific details about which search methods matched and their individual scores
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### UPDATE amp/server/src/services/mod.rs

- **ADD**: Export hybrid module in alphabetical order
- **PATTERN**: Module exports from `amp/server/src/services/mod.rs:1-10`
- **IMPORTS**: None
- **GOTCHA**: Maintain alphabetical ordering of module exports
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### UPDATE amp/server/src/main.rs

- **ADD**: HybridRetrievalService to AppState structure and initialization
- **PATTERN**: Service initialization from `amp/server/src/main.rs:80-100`
- **IMPORTS**: `crate::services::hybrid::HybridRetrievalService`
- **GOTCHA**: Pass Arc<Database>, Arc<EmbeddingService>, and Arc<GraphTraversalService> to constructor
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### UPDATE QueryRequest structure

- **ADD**: hybrid field (Option<bool>) to enable hybrid retrieval mode
- **PATTERN**: Request structure from `amp/server/src/handlers/query.rs:10-25`
- **IMPORTS**: None additional
- **GOTCHA**: Default to false for backward compatibility, only enable when explicitly requested
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### UPDATE query handler routing logic

- **ADD**: Hybrid query detection and routing to HybridRetrievalService
- **PATTERN**: Query routing from `amp/server/src/handlers/query.rs:60-80`
- **IMPORTS**: None additional
- **GOTCHA**: Check hybrid flag first, then fall back to existing single-method logic
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### IMPLEMENT hybrid query execution in query handler

- **ADD**: Call to hybrid_service.execute_hybrid_query when hybrid=true
- **PATTERN**: Service method calls from `amp/server/src/handlers/query.rs:100-150`
- **IMPORTS**: None additional
- **GOTCHA**: Convert HybridResult to QueryResult for response compatibility
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check`

### CREATE amp/scripts/test-hybrid-retrieval.ps1

- **IMPLEMENT**: Comprehensive test script covering hybrid query scenarios
- **PATTERN**: Test script structure from `amp/scripts/test-query.ps1:1-100`
- **IMPORTS**: None (PowerShell script)
- **GOTCHA**: Test with objects that match different search methods (text, vector, graph)
- **VALIDATE**: `cd /mnt/c/Users/Joshc/source/repos/ACM/amp && powershell -ExecutionPolicy Bypass -File scripts/test-hybrid-retrieval.ps1`

### CREATE test objects for hybrid validation

- **IMPLEMENT**: Create test objects with text content, embeddings, and relationships
- **PATTERN**: Object creation from `amp/scripts/test-query.ps1:15-50`
- **IMPORTS**: None (PowerShell script)
- **GOTCHA**: Ensure objects have overlapping but distinct matches for different search methods
- **VALIDATE**: Objects created successfully via API calls

### IMPLEMENT hybrid query test cases

- **ADD**: Test cases for text+vector, text+graph, vector+graph, and text+vector+graph combinations
- **PATTERN**: Query testing from `amp/scripts/test-query.ps1:60-100`
- **IMPORTS**: None (PowerShell script)
- **GOTCHA**: Validate that hybrid results include scores from multiple methods
- **VALIDATE**: All test cases return expected results with proper scoring

### CREATE amp/examples/hybrid_query_examples.surql

- **IMPLEMENT**: Documentation examples showing hybrid query usage patterns
- **PATTERN**: Example structure from existing .surql files
- **IMPORTS**: None (SurrealQL examples)
- **GOTCHA**: Include examples for different hybrid combinations and use cases
- **VALIDATE**: Examples are syntactically correct and demonstrate key features

### ADD performance monitoring

- **IMPLEMENT**: Execution time tracking for individual and combined query methods
- **PATTERN**: Performance tracking from `amp/server/src/handlers/query.rs:25-35`
- **IMPORTS**: `std::time::Instant`
- **GOTCHA**: Track both individual query times and total hybrid execution time
- **VALIDATE**: Performance metrics appear in logs and response

### IMPLEMENT timeout handling for hybrid queries

- **ADD**: 5-second timeout for entire hybrid operation with graceful degradation
- **PATTERN**: Timeout handling from `amp/server/src/handlers/query.rs:150-170`
- **IMPORTS**: `tokio::time::{timeout, Duration}`
- **GOTCHA**: If one query times out, continue with results from successful queries
- **VALIDATE**: Timeout behavior works correctly under load

---

## TESTING STRATEGY

### Unit Tests

Create unit tests for the HybridRetrievalService focusing on:
- Parallel query execution with mocked database responses
- Result deduplication logic with overlapping object IDs
- Scoring algorithm accuracy with different input combinations
- Error handling for partial query failures

Design unit tests with fixtures following existing Rust test patterns using `#[tokio::test]` for async tests.

### Integration Tests

Create integration tests that validate:
- End-to-end hybrid query execution through the API
- Backward compatibility with existing single-method queries
- Performance characteristics under concurrent load
- Result relevance and ranking accuracy

### Edge Cases

Test specific edge cases including:
- Queries where only one search method returns results
- Queries with no results from any method
- Queries with identical objects returned by multiple methods
- Queries that timeout or fail partially
- Large result sets that require pagination
- Malformed hybrid query parameters

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo check
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo clippy -- -D warnings
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo fmt --check
```

### Level 2: Unit Tests

```bash
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo test services::hybrid --lib
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo test handlers::query::hybrid --lib
```

### Level 3: Integration Tests

```bash
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo run &
sleep 5
cd /mnt/c/Users/Joshc/source/repos/ACM/amp && powershell -ExecutionPolicy Bypass -File scripts/test-hybrid-retrieval.ps1
cd /mnt/c/Users/Joshc/source/repos/ACM/amp && powershell -ExecutionPolicy Bypass -File scripts/test-query.ps1
pkill amp-server
```

### Level 4: Manual Validation

```bash
# Start server
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo run &

# Test hybrid query with curl
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{"text": "authentication", "hybrid": true, "limit": 5}'

# Test backward compatibility
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{"text": "authentication", "limit": 5}'

# Stop server
pkill amp-server
```

### Level 5: Additional Validation (Optional)

```bash
# Performance benchmarking
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server && cargo bench --bench hybrid_retrieval
```

---

## ACCEPTANCE CRITERIA

- [ ] Hybrid retrieval combines text, vector, and graph search results intelligently
- [ ] Parallel query execution improves performance over sequential queries
- [ ] Result deduplication prevents duplicate objects in responses
- [ ] Multi-modal scoring provides meaningful relevance ranking
- [ ] Comprehensive explanations show why each result was returned
- [ ] Backward compatibility maintained for existing single-method queries
- [ ] All validation commands pass with zero errors
- [ ] Unit test coverage meets 80%+ requirement
- [ ] Integration tests verify end-to-end hybrid query workflows
- [ ] Performance meets requirements (hybrid queries < 5 seconds)
- [ ] Error handling gracefully manages partial query failures
- [ ] Code follows existing project conventions and patterns
- [ ] No regressions in existing functionality
- [ ] Documentation examples demonstrate key hybrid query patterns

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit + integration)
- [ ] No linting or type checking errors
- [ ] Manual testing confirms hybrid retrieval works
- [ ] Backward compatibility verified
- [ ] Performance benchmarks meet requirements
- [ ] Acceptance criteria all met
- [ ] Code reviewed for quality and maintainability

---

## NOTES

**Design Decisions:**
- Used tokio::try_join! for structured concurrency instead of spawn-based parallelism for better error handling and resource management
- Implemented graceful degradation where partial query failures don't prevent returning results from successful queries
- Chose weighted scoring (vector: 0.4, text: 0.3, graph: 0.3) based on typical relevance patterns, but made weights configurable
- Maintained backward compatibility by making hybrid mode opt-in via explicit flag

**Performance Considerations:**
- 5-second timeout for entire hybrid operation prevents hanging queries
- Result deduplication reduces response size and improves relevance
- Parallel execution significantly improves latency over sequential queries
- Memory usage controlled through bounded result sets and efficient data structures

**Future Enhancements:**
- Configurable scoring weights per query
- Machine learning-based relevance scoring
- Query result caching for frequently accessed patterns
- Advanced graph traversal integration with semantic similarity
