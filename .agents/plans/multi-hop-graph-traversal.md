# Feature: Multi-hop Graph Traversal

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Implement multi-hop graph traversal capabilities in the AMP query system to enable deep relationship exploration beyond single-hop queries. This feature allows agents to traverse graph relationships across multiple levels (depth > 1) with configurable algorithms including path collection, shortest path finding, and recursive exploration with depth limits.

## User Story

As an AI agent developer
I want to perform multi-hop graph traversal queries
So that I can explore deep relationships in my codebase memory and understand complex dependency chains, call hierarchies, and architectural patterns across multiple levels

## Problem Statement

The current AMP implementation only supports single-hop graph traversal (depth = 1), limiting agents to immediate relationships. This prevents exploration of:
- Multi-level dependency chains (A depends on B depends on C)
- Deep call hierarchies in codebases
- Complex architectural decision justification paths
- Transitive relationships between symbols, decisions, and changesets

## Solution Statement

Implement SurrealDB's native recursive query capabilities using the `{depth+algorithm}` syntax to enable multi-hop traversal with three algorithms:
1. **Path Collection**: Return all unique nodes within specified depth
2. **Path Tracing**: Return all possible paths with full traversal history
3. **Shortest Path**: Find optimal path between start and target nodes

## Feature Metadata

**Feature Type**: Enhancement
**Estimated Complexity**: Medium
**Primary Systems Affected**: Query handler, Graph query builder, Response models
**Dependencies**: SurrealDB 2.1+ (recursive queries), Existing relationship system

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/handlers/query.rs` (lines 25-35, 340-400) - Why: Contains GraphQuery struct and current single-hop implementation
- `amp/server/src/handlers/query.rs` (lines 340-400) - Why: build_graph_query_string function needs multi-hop support
- `amp/server/src/handlers/query.rs` (lines 1-50) - Why: QueryRequest/QueryResponse models may need extension
- `amp/scripts/test-graph-traversal.ps1` - Why: Current test pattern for validation
- `amp/spec/example_queries.surql` (lines 32-40) - Why: Shows current graph traversal patterns

### New Files to Create

- `amp/scripts/test-multi-hop-traversal.ps1` - Comprehensive multi-hop test script
- `amp/examples/multi_hop_examples.surql` - Example queries for documentation

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [SurrealDB Recursive Queries](https://surrealdb.com/learn/tour/page-29)
  - Specific section: Recursive syntax `{depth+algorithm}` patterns
  - Why: Core implementation patterns for multi-hop traversal
- [SurrealDB Graph Algorithms](https://surrealdb.com/learn/tour/page-30)
  - Specific section: Path, collect, shortest algorithms
  - Why: Algorithm-specific syntax and behavior patterns
- [SurrealDB Idioms Documentation](https://surrealdb.com/docs/surrealql/datamodel/idioms)
  - Specific section: Graph traversal idioms
  - Why: Proper syntax for relationship navigation

### Patterns to Follow

**Naming Conventions:**
- Use snake_case for Rust struct fields: `max_depth`, `algorithm_type`
- Use kebab-case for JSON API fields: `max-depth`, `algorithm-type`
- Use PascalCase for enum variants: `PathCollection`, `ShortestPath`

**Error Handling:**
```rust
// Pattern from handlers/query.rs:200-220
match timeout(Duration::from_secs(5), db_operation).await {
    Ok(Ok(result)) => Ok(result),
    Ok(Err(e)) => {
        error!("Database error: {}", e);
        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { ... })))
    }
    Err(_) => {
        error!("Database timeout");
        Err((StatusCode::REQUEST_TIMEOUT, Json(ErrorResponse { ... })))
    }
}
```

**Query Building Pattern:**
```rust
// Pattern from handlers/query.rs:340-400
fn build_graph_query_string(graph: &GraphQuery, filters: Option<&QueryFilters>, limit: usize) -> String {
    let mut query = String::new();
    // Build base query
    // Add conditions
    // Add limits
    query
}
```

**Response Model Pattern:**
```rust
// Pattern from handlers/query.rs:40-60
#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub object: Value,
    pub score: f32,
    pub explanation: String,
    pub path: Option<Vec<String>>, // New field for multi-hop
}
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Extend existing GraphQuery model to support multi-hop parameters and algorithm selection.

**Tasks:**
- Add multi-hop fields to GraphQuery struct
- Create TraversalAlgorithm enum for algorithm types
- Update QueryResult to include path information
- Validate SurrealDB recursive syntax patterns

### Phase 2: Core Implementation

Implement multi-hop query building with SurrealDB recursive syntax.

**Tasks:**
- Extend build_graph_query_string for recursive queries
- Add algorithm-specific query generation
- Implement path result parsing and formatting
- Add depth validation and safety limits

### Phase 3: Integration

Integrate multi-hop traversal into existing query endpoint.

**Tasks:**
- Update query handler to process multi-hop requests
- Enhance response formatting for path data
- Add proper error handling for recursive queries
- Update result scoring for multi-hop results

### Phase 4: Testing & Validation

Create comprehensive test suite for multi-hop functionality.

**Tasks:**
- Implement multi-hop test scenarios
- Test all three algorithms (path, collect, shortest)
- Validate depth limits and performance
- Test integration with existing filters

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### 1. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Add TraversalAlgorithm enum and multi-hop fields to GraphQuery
- **PATTERN**: Enum pattern from GraphDirection (lines 33-38)
- **IMPORTS**: No new imports needed
- **GOTCHA**: Keep backward compatibility with existing single-hop queries
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphQuery {
    pub start_nodes: Vec<Uuid>,
    pub relation_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    pub direction: Option<GraphDirection>,
    pub algorithm: Option<TraversalAlgorithm>,
    pub target_node: Option<Uuid>, // For shortest path algorithm
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TraversalAlgorithm {
    Collect,    // Collect unique nodes
    Path,       // Return all paths
    Shortest,   // Shortest path to target
}
```

### 2. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Add path field to QueryResult for multi-hop responses
- **PATTERN**: QueryResult struct pattern (lines 40-60)
- **IMPORTS**: No new imports needed
- **GOTCHA**: Make path optional for backward compatibility
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

```rust
#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub object: Value,
    pub score: f32,
    pub explanation: String,
    pub path: Option<Vec<Value>>, // New field for traversal paths
}
```

### 3. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Extend build_graph_query_string for recursive syntax
- **PATTERN**: Query building pattern from lines 340-400
- **IMPORTS**: No new imports needed
- **GOTCHA**: SurrealDB recursive syntax is `{depth+algorithm}` not `{depth, algorithm}`
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

```rust
fn build_graph_query_string(graph: &GraphQuery, filters: Option<&QueryFilters>, limit: usize) -> String {
    let direction = graph.direction.as_ref().unwrap_or(&GraphDirection::Outbound);
    let max_depth = graph.max_depth.unwrap_or(3);
    
    // Build recursive syntax based on algorithm
    let recursive_syntax = match &graph.algorithm {
        Some(TraversalAlgorithm::Collect) => format!("{{{}+collect}}", max_depth),
        Some(TraversalAlgorithm::Path) => format!("{{{}+path}}", max_depth),
        Some(TraversalAlgorithm::Shortest) => {
            if let Some(target) = &graph.target_node {
                format!("{{..{}+shortest=objects:`{}`}}", max_depth, target)
            } else {
                format!("{{{}}}", max_depth) // Fallback to simple depth
            }
        }
        None => format!("{{{}}}", max_depth), // Simple depth traversal
    };
    
    // Rest of implementation...
}
```

### 4. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Update execute_graph_query to handle recursive results
- **PATTERN**: Database query execution pattern from lines 200-250
- **IMPORTS**: No new imports needed
- **GOTCHA**: Recursive queries return different result structures than single-hop
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### 5. CREATE amp/scripts/test-multi-hop-traversal.ps1

- **IMPLEMENT**: Comprehensive test script for multi-hop functionality
- **PATTERN**: Mirror test-graph-traversal.ps1 structure
- **IMPORTS**: PowerShell Invoke-RestMethod patterns
- **GOTCHA**: Test all three algorithms with proper validation
- **VALIDATE**: `powershell -ExecutionPolicy Bypass -File amp/scripts/test-multi-hop-traversal.ps1`

### 6. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Add depth validation and safety limits
- **PATTERN**: Validation pattern from existing query handlers
- **IMPORTS**: No new imports needed
- **GOTCHA**: Prevent excessive depth that could cause performance issues
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

```rust
// Add to GraphQuery validation
if let Some(depth) = &graph.max_depth {
    if *depth > 10 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "max_depth cannot exceed 10 for performance reasons".to_string(),
            })
        ));
    }
}
```

### 7. UPDATE amp/server/src/handlers/query.rs

- **IMPLEMENT**: Enhanced result parsing for path information
- **PATTERN**: Result processing pattern from lines 400-450
- **IMPORTS**: No new imports needed
- **GOTCHA**: Path results have different JSON structure than single objects
- **VALIDATE**: `cargo test --manifest-path amp/server/Cargo.toml`

### 8. CREATE amp/examples/multi_hop_examples.surql

- **IMPLEMENT**: Example queries demonstrating multi-hop capabilities
- **PATTERN**: Mirror amp/spec/example_queries.surql format
- **IMPORTS**: No imports needed for .surql files
- **GOTCHA**: Use proper SurrealDB recursive syntax
- **VALIDATE**: Manual review of syntax against SurrealDB documentation

---

## TESTING STRATEGY

### Unit Tests

Test multi-hop query building and validation logic in isolation:
- GraphQuery struct validation with new fields
- TraversalAlgorithm enum serialization/deserialization
- Recursive query string generation for each algorithm
- Depth validation and error handling

### Integration Tests

Test complete multi-hop workflow with database:
- Create test data with 3+ level relationships
- Execute collect algorithm queries
- Execute path algorithm queries  
- Execute shortest path algorithm queries
- Validate result structure and path information

### Edge Cases

- Maximum depth limits (depth > 10)
- Circular relationship handling
- Empty result sets for unreachable nodes
- Invalid target nodes for shortest path
- Performance with large graph structures

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
cargo test handlers::query::tests
```

### Level 3: Integration Tests

```bash
cd /mnt/c/Users/Joshc/source/repos/ACM/amp
# Start server in background
cargo run --manifest-path server/Cargo.toml &
sleep 5

# Run multi-hop tests
powershell -ExecutionPolicy Bypass -File scripts/test-multi-hop-traversal.ps1

# Stop server
pkill -f "amp.*server"
```

### Level 4: Manual Validation

```bash
# Test collect algorithm
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "graph": {
      "start_nodes": ["<test-uuid>"],
      "max_depth": 3,
      "algorithm": "collect",
      "direction": "outbound"
    }
  }'

# Test path algorithm
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "graph": {
      "start_nodes": ["<test-uuid>"],
      "max_depth": 2,
      "algorithm": "path",
      "direction": "outbound"
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
      "direction": "outbound"
    }
  }'
```

### Level 5: Additional Validation (Optional)

```bash
# Run all existing tests to ensure no regressions
cd /mnt/c/Users/Joshc/source/repos/ACM/amp
powershell -ExecutionPolicy Bypass -File scripts/run-all-tests.ps1
```

---

## ACCEPTANCE CRITERIA

- [ ] GraphQuery supports max_depth > 1 with algorithm selection
- [ ] Three algorithms implemented: collect, path, shortest
- [ ] Recursive queries use proper SurrealDB `{depth+algorithm}` syntax
- [ ] QueryResult includes path information for multi-hop results
- [ ] Depth validation prevents excessive traversal (max 10 levels)
- [ ] All validation commands pass with zero errors
- [ ] Multi-hop test script validates all algorithms
- [ ] Backward compatibility maintained for single-hop queries
- [ ] Performance acceptable for depth <= 5 on test data
- [ ] Error handling covers invalid targets and circular references

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Multi-hop test script passes all scenarios
- [ ] No linting or type checking errors
- [ ] Manual API testing confirms all algorithms work
- [ ] Acceptance criteria all met
- [ ] No regressions in existing single-hop functionality

---

## NOTES

**Design Decisions:**
- Used SurrealDB's native recursive syntax for optimal performance
- Made algorithm field optional to maintain backward compatibility
- Limited max_depth to 10 to prevent performance issues
- Added path field to QueryResult for traversal history

**Performance Considerations:**
- Recursive queries can be expensive on large graphs
- Depth limits prevent runaway queries
- Consider adding query timeouts for deep traversals

**Future Enhancements:**
- Add graph visualization support for path results
- Implement query result caching for expensive traversals
- Add metrics for traversal performance monitoring
