# Feature: Application-level Multi-hop Graph Traversal Logic

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Implement true multi-hop graph traversal logic at the application level in Rust to enable depth > 1 relationship exploration. This feature will provide the core traversal algorithms (Collect, Path, Shortest) that work with AMP's existing relationship-based graph model, performing multiple database queries and combining results to achieve deep graph exploration.

## User Story

As an AI agent developer
I want to perform true multi-hop graph traversal with depth > 1
So that I can explore complex dependency chains, call hierarchies, and architectural patterns across multiple levels in my codebase memory

## Problem Statement

The current AMP implementation has the API structure for multi-hop queries but falls back to single-hop traversal due to SurrealDB's recursive syntax being incompatible with relationship-based graphs. Agents need to explore deep relationships like "find all functions that depend on library X through any number of intermediate dependencies" or "trace the complete justification chain from implementation to architectural decision."

## Solution Statement

Implement application-level multi-hop logic in Rust that performs iterative database queries, maintains visited node tracking, and implements the three core algorithms (Collect, Path, Shortest) using efficient graph traversal patterns. This approach provides full control over traversal logic while working within SurrealDB's relationship model constraints.

## Feature Metadata

**Feature Type**: Enhancement
**Estimated Complexity**: Medium-High
**Primary Systems Affected**: Query handler, Graph traversal service, Response processing
**Dependencies**: Existing relationship system, SurrealDB client, Rust collections (HashSet, Vec)

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/handlers/query.rs` (lines 25-50, 80-150) - Why: Contains GraphQuery struct, TraversalAlgorithm enum, and current query handling logic
- `amp/server/src/handlers/query.rs` (lines 366-450) - Why: build_graph_query_string function shows current single-hop query patterns
- `amp/server/src/surreal_json.rs` (lines 1-20) - Why: take_json_values function for processing SurrealDB responses
- `amp/server/src/database.rs` (lines 1-50) - Why: Database connection patterns and query execution
- `amp/server/src/services/mod.rs` - Why: Service module structure for new graph service
- `amp/server/src/handlers/relationships.rs` (lines 20-60) - Why: Relationship query patterns and response processing

### New Files to Create

- `amp/server/src/services/graph.rs` - Multi-hop traversal service implementation
- `amp/scripts/test-multi-hop-logic.ps1` - Comprehensive test script for multi-hop algorithms
- `amp/examples/multi_hop_traversal_examples.surql` - Example queries demonstrating multi-hop capabilities

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [Rust HashSet Documentation](https://doc.rust-lang.org/std/collections/struct.HashSet.html)
  - Specific section: insert, contains, and iteration methods
  - Why: Essential for visited node tracking and cycle prevention
- [Rust async/await Patterns](https://rust-lang.github.io/async-book/01_getting_started/04_async_await_primer.html)
  - Specific section: Async functions and error handling
  - Why: Multi-hop queries require multiple async database calls
- [SurrealDB Rust SDK Query Methods](https://surrealdb.com/docs/sdk/rust)
  - Specific section: Query execution and response handling
  - Why: Understanding query patterns for iterative traversal

### Patterns to Follow

**Naming Conventions:**
- Use snake_case for Rust functions: `execute_collect_traversal`, `find_shortest_path`
- Use PascalCase for struct names: `TraversalResult`, `PathNode`
- Use SCREAMING_SNAKE_CASE for constants: `MAX_TRAVERSAL_DEPTH`, `DEFAULT_BATCH_SIZE`

**Error Handling:**
```rust
// Pattern from handlers/query.rs:200-220
match timeout(Duration::from_secs(5), db_operation).await {
    Ok(Ok(result)) => Ok(result),
    Ok(Err(e)) => {
        tracing::error!("Database error: {}", e);
        Err(GraphTraversalError::DatabaseError(e.to_string()))
    }
    Err(_) => {
        tracing::error!("Database timeout");
        Err(GraphTraversalError::Timeout)
    }
}
```

**Async Service Pattern:**
```rust
// Pattern from services/embedding.rs:20-30
pub struct GraphTraversalService {
    db: Database,
}

impl GraphTraversalService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
    
    pub async fn execute_multi_hop(&self, query: &GraphQuery) -> Result<Vec<QueryResult>, GraphTraversalError> {
        // Implementation
    }
}
```

**Response Processing Pattern:**
```rust
// Pattern from handlers/relationships.rs:50-60
let objects: Vec<Value> = match query_result {
    Ok(Ok(mut response)) => {
        let raw_results: Vec<Value> = take_json_values(&mut response, 0);
        normalize_object_ids(&mut raw_results);
        raw_results
    }
    Ok(Err(e)) => return Err(GraphTraversalError::DatabaseError(e.to_string())),
    Err(_) => return Err(GraphTraversalError::Timeout),
};
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Create the graph traversal service infrastructure and core data structures.

**Tasks:**
- Create GraphTraversalService with database connection
- Define TraversalResult and PathNode data structures
- Implement error types for graph traversal operations
- Add service to module exports and dependency injection

### Phase 2: Core Algorithms

Implement the three core multi-hop algorithms with proper cycle detection and depth limits.

**Tasks:**
- Implement Collect algorithm (breadth-first unique node collection)
- Implement Path algorithm (all paths enumeration with backtracking)
- Implement Shortest algorithm (Dijkstra-style shortest path finding)
- Add comprehensive logging and performance monitoring

### Phase 3: Integration

Integrate multi-hop service into existing query handler and update response processing.

**Tasks:**
- Update query handler to use GraphTraversalService for multi-hop queries
- Enhance QueryResult to include path information from multi-hop results
- Update response formatting to handle complex traversal results
- Add proper timeout and resource management

### Phase 4: Testing & Validation

Create comprehensive test suite and validation scripts for all algorithms.

**Tasks:**
- Implement unit tests for each algorithm with various graph structures
- Create integration tests with real relationship data
- Add performance tests for large graph traversals
- Validate edge cases (cycles, disconnected graphs, maximum depth)

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### 1. CREATE amp/server/src/services/graph.rs

- **IMPLEMENT**: GraphTraversalService struct and core infrastructure
- **PATTERN**: Service pattern from services/embedding.rs (lines 15-30)
- **IMPORTS**: `use std::collections::{HashSet, VecDeque}`, `use uuid::Uuid`, `use serde_json::Value`
- **GOTCHA**: Use VecDeque for BFS traversal, HashSet for visited tracking
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 2. UPDATE amp/server/src/services/mod.rs

- **IMPLEMENT**: Add graph module export
- **PATTERN**: Module export pattern (existing file)
- **IMPORTS**: `pub mod graph;`
- **GOTCHA**: Maintain alphabetical ordering of module exports
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 3. UPDATE amp/server/src/services/graph.rs

- **IMPLEMENT**: TraversalResult and PathNode data structures
- **PATTERN**: Data structure pattern from models/mod.rs (lines 10-30)
- **IMPORTS**: `use serde::{Deserialize, Serialize}`
- **GOTCHA**: Make structures serializable for JSON responses
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 4. UPDATE amp/server/src/services/graph.rs

- **IMPLEMENT**: GraphTraversalError enum with proper error types
- **PATTERN**: Error handling pattern from handlers/query.rs (lines 200-220)
- **IMPORTS**: `use thiserror::Error`
- **GOTCHA**: Include context information in error messages
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 5. UPDATE amp/server/src/services/graph.rs

- **IMPLEMENT**: execute_collect_traversal method (breadth-first unique collection)
- **PATTERN**: Async method pattern from database.rs (lines 40-60)
- **IMPORTS**: `use tokio::time::{timeout, Duration}`
- **GOTCHA**: Use HashSet to prevent revisiting nodes, respect max_depth limit
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 6. UPDATE amp/server/src/services/graph.rs

- **IMPLEMENT**: execute_path_traversal method (all paths enumeration)
- **PATTERN**: Recursive traversal with backtracking
- **IMPORTS**: No additional imports needed
- **GOTCHA**: Use Vec<Vec<Uuid>> to store all paths, implement cycle detection
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 7. UPDATE amp/server/src/services/graph.rs

- **IMPLEMENT**: execute_shortest_path method (Dijkstra-style pathfinding)
- **PATTERN**: Priority queue traversal pattern
- **IMPORTS**: `use std::collections::BinaryHeap`
- **GOTCHA**: Implement early termination when target found, handle unreachable targets
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 8. UPDATE amp/server/src/main.rs

- **IMPLEMENT**: Add GraphTraversalService to AppState
- **PATTERN**: Service injection pattern from existing AppState (lines 20-40)
- **IMPORTS**: `use crate::services::graph::GraphTraversalService`
- **GOTCHA**: Initialize service with database connection
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 9. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Integration of GraphTraversalService in query handler
- **PATTERN**: Service usage pattern from existing handlers (lines 80-120)
- **IMPORTS**: `use crate::services::graph::{GraphTraversalService, TraversalResult}`
- **GOTCHA**: Only use multi-hop service when algorithm is specified and depth > 1
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 10. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Enhanced QueryResult processing for multi-hop results
- **PATTERN**: Result processing pattern from lines 130-150
- **IMPORTS**: No additional imports needed
- **GOTCHA**: Convert TraversalResult to QueryResult format, include path information
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 11. CREATE amp/scripts/test-multi-hop-logic.ps1

- **IMPLEMENT**: Comprehensive test script for all three algorithms
- **PATTERN**: Test script pattern from test-multi-hop-fixed.ps1
- **IMPORTS**: PowerShell Invoke-RestMethod patterns
- **GOTCHA**: Create complex relationship chains (A->B->C->D->E) for thorough testing
- **VALIDATE**: `powershell -ExecutionPolicy Bypass -File amp/scripts/test-multi-hop-logic.ps1`

### 12. CREATE amp/examples/multi_hop_traversal_examples.surql

- **IMPLEMENT**: Example queries demonstrating multi-hop capabilities
- **PATTERN**: Example format from existing .surql files
- **IMPORTS**: No imports needed for .surql files
- **GOTCHA**: Include complex scenarios (cycles, multiple paths, disconnected components)
- **VALIDATE**: Manual review of syntax and completeness

---

## TESTING STRATEGY

### Unit Tests

Test each algorithm in isolation with controlled graph structures:
- Small graphs (3-5 nodes) with known traversal results
- Cycle detection and prevention
- Maximum depth enforcement
- Empty result handling for unreachable nodes

### Integration Tests

Test complete workflow with database integration:
- Create complex relationship chains (depth 3-5)
- Test all three algorithms with same data set
- Validate response format and path information
- Test performance with larger graphs (50+ nodes)

### Edge Cases

- **Cycles**: Ensure algorithms don't infinite loop
- **Disconnected graphs**: Handle unreachable target nodes gracefully
- **Maximum depth**: Respect depth limits and return partial results
- **Large graphs**: Performance testing with 100+ nodes and relationships
- **Concurrent queries**: Multiple multi-hop queries running simultaneously

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server
cargo fmt
cargo clippy -- -D warnings
```

### Level 2: Unit Tests

```bash
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server
cargo test services::graph::tests
cargo test handlers::query::tests
```

### Level 3: Integration Tests

```bash
cd /mnt/c/Users/Joshc/source/repos/ACM/amp
# Start server in background
cargo run --manifest-path server/Cargo.toml &
sleep 5

# Run multi-hop logic tests
powershell -ExecutionPolicy Bypass -File scripts/test-multi-hop-logic.ps1

# Stop server
pkill -f "amp.*server"
```

### Level 4: Manual Validation

```bash
# Test collect algorithm with depth 3
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "graph": {
      "start_nodes": ["<test-uuid>"],
      "max_depth": 3,
      "algorithm": "collect",
      "direction": "outbound",
      "relation_types": ["calls"]
    }
  }'

# Test path algorithm with depth 2
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "graph": {
      "start_nodes": ["<test-uuid>"],
      "max_depth": 2,
      "algorithm": "path",
      "direction": "outbound",
      "relation_types": ["calls"]
    }
  }'

# Test shortest path algorithm
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "graph": {
      "start_nodes": ["<start-uuid>"],
      "target_node": "<target-uuid>",
      "max_depth": 5,
      "algorithm": "shortest",
      "direction": "outbound",
      "relation_types": ["calls"]
    }
  }'
```

### Level 5: Performance Validation

```bash
# Test with large graph (create 50+ nodes and relationships first)
# Measure response times for each algorithm
# Validate memory usage doesn't exceed reasonable limits
```

---

## ACCEPTANCE CRITERIA

- [ ] Collect algorithm returns all unique nodes reachable within max_depth
- [ ] Path algorithm returns all possible paths from start to reachable nodes
- [ ] Shortest algorithm finds optimal path to target node (if reachable)
- [ ] All algorithms respect max_depth limits (default 3, maximum 10)
- [ ] Cycle detection prevents infinite loops in all algorithms
- [ ] QueryResult includes path information for multi-hop results
- [ ] Performance acceptable for graphs up to 100 nodes (< 5 seconds)
- [ ] All validation commands pass with zero errors
- [ ] Comprehensive test coverage for all algorithms and edge cases
- [ ] Backward compatibility maintained for single-hop queries
- [ ] Proper error handling for unreachable targets and timeouts

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Multi-hop test script passes all scenarios
- [ ] No linting or type checking errors
- [ ] Manual API testing confirms all algorithms work correctly
- [ ] Performance testing shows acceptable response times
- [ ] Acceptance criteria all met
- [ ] No regressions in existing functionality

---

## NOTES

**Design Decisions:**
- Used application-level logic to work within SurrealDB relationship model constraints
- Implemented breadth-first search for Collect algorithm to ensure optimal performance
- Added comprehensive cycle detection to prevent infinite loops
- Used efficient data structures (HashSet, VecDeque) for optimal memory usage

**Performance Considerations:**
- Multi-hop queries require multiple database round trips
- Memory usage scales with graph size and depth
- Timeout protection prevents runaway queries
- Consider caching for frequently accessed paths

**Future Enhancements:**
- Parallel traversal for independent branches
- Query result caching for expensive multi-hop operations
- Graph analytics (centrality, clustering coefficients)
- Weighted shortest paths with custom edge weights
