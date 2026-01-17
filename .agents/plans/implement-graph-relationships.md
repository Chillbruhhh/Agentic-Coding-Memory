# Implementation Plan: Graph Relationship Queries

**Feature**: Graph relationship traversal using SurrealDB's graph capabilities
**Estimated Time**: 90-120 minutes
**Priority**: High (Core value proposition - understanding code relationships)

## Overview

Implement graph relationship queries to enable agents to traverse connections between memory objects. This allows discovering how code elements relate to each other (e.g., "what functions call this function?", "what decisions justify this change?").

## Current State

- ✅ Database schema has relationship tables defined in `schema.surql`
- ✅ Query endpoint exists with text and vector search
- ✅ Base object model has `links` field for relationships
- ❌ No relationship creation endpoints
- ❌ No graph traversal in query endpoint
- ❌ Relationships not being created

## Architecture Design

### Relationship Types (from schema.surql)

**Code Structure:**
- `depends_on` - Symbol → Symbol (imports, dependencies)
- `defined_in` - Symbol → Symbol (class contains method)
- `calls` - Symbol → Symbol (function calls function)

**Decision Tracking:**
- `justified_by` - ChangeSet → Decision (change implements decision)
- `modifies` - ChangeSet → Symbol (change modifies code)
- `implements` - ChangeSet → Symbol (change implements feature)

**Execution Tracking:**
- `produced` - Run → ChangeSet (run produced changes)

### Graph Query Approach

SurrealDB supports graph traversal with `->` and `<-` operators:

```sql
-- Find all functions that call authenticate_user
SELECT * FROM symbols WHERE id IN (
    SELECT <-calls<-symbol FROM symbols:authenticate_user
)

-- Find all decisions that justify a changeset
SELECT * FROM decisions WHERE id IN (
    SELECT ->justified_by->decision FROM changesets:change123
)

-- Multi-hop: Find all changes that implement decisions made in last 30 days
SELECT * FROM changesets WHERE id IN (
    SELECT <-justified_by<-changeset 
    FROM decisions 
    WHERE created_at > time::now() - 30d
)
```

## Implementation Strategy

### Phase 1: Relationship Management Endpoints (30 minutes)

**Step 1.1: Create relationship models**
- File: `amp/server/src/models/relationships.rs`
- Define relationship types enum
- Define relationship creation request/response

**Step 1.2: Create relationship handlers**
- File: `amp/server/src/handlers/relationships.rs`
- `create_relationship()` - POST /v1/relationships
- `get_relationships()` - GET /v1/relationships?source_id=...&type=...
- `delete_relationship()` - DELETE /v1/relationships/{id}

**Step 1.3: Update routing**
- Add relationship routes to main.rs

### Phase 2: Graph Traversal in Query Endpoint (40 minutes)

**Step 2.1: Add graph query support to QueryRequest**
- Update `GraphQuery` struct with proper fields
- Support `start_nodes`, `relation_types`, `max_depth`, `direction`

**Step 2.2: Implement graph query builder**
- Function: `build_graph_query_string()`
- Use SurrealDB's `->` and `<-` operators
- Support multi-hop traversal with depth limits
- Combine with filters

**Step 2.3: Update query endpoint**
- Detect when graph query is requested
- Execute graph traversal
- Merge results with text/vector search if both provided

### Phase 3: Relationship Auto-Creation (20 minutes)

**Step 3.1: Add relationship extraction helpers**
- Extract relationships from object content
- For Symbols: Parse imports/calls from signature
- For ChangeSets: Link to modified symbols
- For Decisions: Link to related symbols

**Step 3.2: Create relationships on object creation**
- Optional: Auto-create relationships based on content
- Or: Require explicit relationship creation via API

### Phase 4: Testing (20 minutes)

**Step 4.1: Create relationship test script**
- Create objects with relationships
- Test relationship creation
- Test relationship queries

**Step 4.2: Create graph traversal test script**
- Create connected objects
- Test single-hop traversal
- Test multi-hop traversal
- Test filtered graph queries

## Detailed Implementation

### File: `amp/server/src/models/relationships.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use surrealdb::sql::Datetime;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    DependsOn,
    DefinedIn,
    Calls,
    JustifiedBy,
    Modifies,
    Implements,
    Produced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub metadata: Option<serde_json::Value>,
    pub created_at: Datetime,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationshipRequest {
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RelationshipResponse {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub relation_type: RelationType,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub created_at: String,
}
```

### File: `amp/server/src/handlers/relationships.rs`

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;
use tokio::time::{timeout, Duration};

use crate::{models::relationships::*, AppState};

#[derive(Debug, Deserialize)]
pub struct RelationshipQuery {
    pub source_id: Option<Uuid>,
    pub target_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub relation_type: Option<String>,
}

pub async fn create_relationship(
    State(state): State<AppState>,
    Json(request): Json<CreateRelationshipRequest>,
) -> Result<(StatusCode, Json<RelationshipResponse>), StatusCode> {
    let relationship_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    let surreal_now = surrealdb::sql::Datetime::from(now);
    
    // Determine table name based on relationship type
    let table_name = match request.relation_type {
        RelationType::DependsOn => "depends_on",
        RelationType::DefinedIn => "defined_in",
        RelationType::Calls => "calls",
        RelationType::JustifiedBy => "justified_by",
        RelationType::Modifies => "modifies",
        RelationType::Implements => "implements",
        RelationType::Produced => "produced",
    };
    
    let relationship = Relationship {
        id: relationship_id,
        relation_type: request.relation_type.clone(),
        source_id: request.source_id,
        target_id: request.target_id,
        metadata: request.metadata,
        created_at: surreal_now,
    };
    
    // Insert relationship
    let result: Result<Result<Option<Value>, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client
            .insert((table_name, relationship_id.to_string()))
            .content(relationship)
    ).await;
    
    match result {
        Ok(Ok(_)) => {
            tracing::info!("Created relationship: {} -> {} ({})", 
                request.source_id, request.target_id, table_name);
            Ok((
                StatusCode::CREATED,
                Json(RelationshipResponse {
                    id: relationship_id,
                    relation_type: request.relation_type,
                    source_id: request.source_id,
                    target_id: request.target_id,
                    created_at: now.to_rfc3339(),
                }),
            ))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to create relationship: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout creating relationship");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn get_relationships(
    State(state): State<AppState>,
    Query(query): Query<RelationshipQuery>,
) -> Result<Json<Vec<Value>>, StatusCode> {
    // Build query based on filters
    let mut query_str = String::from("SELECT * FROM [");
    
    if let Some(rel_type) = &query.relation_type {
        query_str.push_str(rel_type);
    } else {
        query_str.push_str("depends_on, defined_in, calls, justified_by, modifies, implements, produced");
    }
    
    query_str.push_str("]");
    
    let mut conditions = Vec::new();
    if let Some(source) = query.source_id {
        conditions.push(format!("source_id = '{}'", source));
    }
    if let Some(target) = query.target_id {
        conditions.push(format!("target_id = '{}'", target));
    }
    
    if !conditions.is_empty() {
        query_str.push_str(" WHERE ");
        query_str.push_str(&conditions.join(" AND "));
    }
    
    tracing::debug!("Relationship query: {}", query_str);
    
    let result: Result<Result<surrealdb::Response, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.query(query_str)
    ).await;
    
    match result {
        Ok(Ok(mut response)) => {
            let relationships: Vec<Value> = response.take(0).unwrap_or_default();
            Ok(Json(relationships))
        }
        Ok(Err(e)) => {
            tracing::error!("Failed to query relationships: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout querying relationships");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}

pub async fn delete_relationship(
    State(state): State<AppState>,
    Path((rel_type, id)): Path<(String, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result: Result<Result<Option<Value>, _>, _> = timeout(
        Duration::from_secs(5),
        state.db.client.delete((rel_type.as_str(), id.to_string()))
    ).await;
    
    match result {
        Ok(Ok(Some(_))) => {
            tracing::info!("Deleted relationship: {}:{}", rel_type, id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(Ok(None)) => Err(StatusCode::NOT_FOUND),
        Ok(Err(e)) => {
            tracing::error!("Failed to delete relationship: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => {
            tracing::error!("Timeout deleting relationship");
            Err(StatusCode::GATEWAY_TIMEOUT)
        }
    }
}
```

### Update: `amp/server/src/handlers/query.rs`

Add graph query support:

```rust
// In QueryRequest, update GraphQuery
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphQuery {
    pub start_nodes: Vec<Uuid>,
    pub relation_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    pub direction: Option<GraphDirection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphDirection {
    Outbound,  // Follow -> relationships
    Inbound,   // Follow <- relationships
    Both,      // Follow both directions
}

// Add graph query builder
fn build_graph_query_string(graph: &GraphQuery) -> String {
    let direction = graph.direction.as_ref().unwrap_or(&GraphDirection::Outbound);
    let max_depth = graph.max_depth.unwrap_or(3);
    
    let relation_tables = if let Some(types) = &graph.relation_types {
        types.join(", ")
    } else {
        "depends_on, defined_in, calls, justified_by, modifies, implements, produced".to_string()
    };
    
    let start_ids = graph.start_nodes.iter()
        .map(|id| format!("objects:{}", id))
        .collect::<Vec<_>>()
        .join(", ");
    
    let operator = match direction {
        GraphDirection::Outbound => "->",
        GraphDirection::Inbound => "<-",
        GraphDirection::Both => "<->",
    };
    
    // Build graph traversal query
    format!(
        "SELECT * FROM objects WHERE id IN (
            SELECT {}[{}]{}object.id FROM [{}] LIMIT {}
        )",
        operator, relation_tables, operator, start_ids, max_depth * 100
    )
}

// In query() function, add graph query support
if let Some(graph) = &request.graph {
    let graph_query = build_graph_query_string(graph);
    tracing::debug!("Executing graph query: {}", graph_query);
    
    // Execute graph query and merge with other results
    // ...
}
```

### File: `amp/scripts/test-relationships.ps1`

```powershell
Write-Host "=== AMP Relationship Management Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# Create test objects
Write-Host "1. Creating test objects..." -ForegroundColor Yellow

$symbol1Id = [guid]::NewGuid().ToString()
$symbol2Id = [guid]::NewGuid().ToString()

$symbol1 = @"
{
    "id": "$symbol1Id",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "graph_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Test"},
    "links": [],
    "embedding": null,
    "name": "authenticate_user",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "signature": "fn authenticate_user()",
    "documentation": "Authenticates a user"
}
"@

$symbol2 = @"
{
    "id": "$symbol2Id",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "graph_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Test"},
    "links": [],
    "embedding": null,
    "name": "hash_password",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "signature": "fn hash_password()",
    "documentation": "Hashes a password"
}
"@

Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol1 -ContentType "application/json" | Out-Null
Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol2 -ContentType "application/json" | Out-Null

Write-Host "Created 2 symbols" -ForegroundColor Green
Write-Host ""

# Create relationship
Write-Host "2. Creating 'calls' relationship..." -ForegroundColor Yellow

$relationship = @{
    type = "calls"
    source_id = $symbol1Id
    target_id = $symbol2Id
    metadata = @{
        line_number = 42
    }
} | ConvertTo-Json

$relResponse = Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $relationship -ContentType "application/json"
Write-Host "Created relationship: $($relResponse.id)" -ForegroundColor Green
Write-Host ""

# Query relationships
Write-Host "3. Querying relationships from source..." -ForegroundColor Yellow
$rels = Invoke-RestMethod -Uri "$baseUrl/v1/relationships?source_id=$symbol1Id" -Method Get
Write-Host "Found $($rels.Count) relationship(s)" -ForegroundColor Green
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
```

### File: `amp/scripts/test-graph-traversal.ps1`

```powershell
Write-Host "=== AMP Graph Traversal Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# Create a chain of functions: A calls B calls C
Write-Host "1. Creating function chain..." -ForegroundColor Yellow

$funcA = [guid]::NewGuid().ToString()
$funcB = [guid]::NewGuid().ToString()
$funcC = [guid]::NewGuid().ToString()

# Create functions
@($funcA, $funcB, $funcC) | ForEach-Object {
    $name = switch ($_) {
        $funcA { "function_a" }
        $funcB { "function_b" }
        $funcC { "function_c" }
    }
    
    $obj = @"
{
    "id": "$_",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "graph_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Test"},
    "links": [],
    "embedding": null,
    "name": "$name",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "signature": "fn $name()",
    "documentation": "Test function"
}
"@
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $obj -ContentType "application/json" | Out-Null
}

# Create relationships: A -> B -> C
$rel1 = @{type = "calls"; source_id = $funcA; target_id = $funcB} | ConvertTo-Json
$rel2 = @{type = "calls"; source_id = $funcB; target_id = $funcC} | ConvertTo-Json

Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $rel1 -ContentType "application/json" | Out-Null
Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $rel2 -ContentType "application/json" | Out-Null

Write-Host "Created chain: function_a -> function_b -> function_c" -ForegroundColor Green
Write-Host ""

# Test graph traversal
Write-Host "2. Testing graph traversal from function_a..." -ForegroundColor Yellow

$query = @{
    graph = @{
        start_nodes = @($funcA)
        relation_types = @("calls")
        max_depth = 2
        direction = "outbound"
    }
    limit = 10
} | ConvertTo-Json -Depth 10

$result = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $query -ContentType "application/json"

Write-Host "Found $($result.total_count) connected functions" -ForegroundColor Green
$result.results | ForEach-Object {
    Write-Host "  - $($_.object.name)" -ForegroundColor Gray
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
```

## Testing Checklist

- [ ] Create relationship endpoint works
- [ ] Get relationships with filters works
- [ ] Delete relationship works
- [ ] Graph traversal finds connected nodes
- [ ] Multi-hop traversal works (depth > 1)
- [ ] Direction filtering works (outbound/inbound/both)
- [ ] Relationship type filtering works
- [ ] Graph query combines with text/vector search
- [ ] Timeout handling works (5 seconds)

## Success Criteria

1. ✅ Relationships can be created between objects
2. ✅ Relationships can be queried by source/target/type
3. ✅ Graph traversal finds connected objects
4. ✅ Multi-hop traversal works with depth limits
5. ✅ Direction control works (follow outbound/inbound)
6. ✅ Graph queries can be combined with filters
7. ✅ All operations have 5-second timeout
8. ✅ Test scripts validate all functionality

## Notes

- SurrealDB's graph syntax: `->relation->target` (outbound), `<-relation<-source` (inbound)
- Relationships stored in separate tables per type (depends_on, calls, etc.)
- Graph queries can be expensive - use depth limits and filters
- Consider caching frequently traversed paths
- Relationship metadata can store additional context (line numbers, confidence, etc.)

## Future Enhancements

1. **Relationship Inference**: Auto-detect relationships from code analysis
2. **Relationship Strength**: Weight relationships by frequency/importance
3. **Path Finding**: Find shortest path between two nodes
4. **Subgraph Extraction**: Extract connected components
5. **Relationship Validation**: Ensure relationships make semantic sense

## Estimated Timeline

- Relationship models and handlers: 30 minutes
- Graph query implementation: 40 minutes
- Routing and integration: 10 minutes
- Test scripts: 20 minutes
- Testing and debugging: 20 minutes

**Total**: 120 minutes (~2 hours)
