# Graph Traversal Fix - Inbound Direction

## Problem
Inbound graph traversal was returning 0 results due to incorrect SurrealDB query syntax.

## Root Cause
The inbound traversal syntax was backwards. We had:
```sql
SELECT * FROM objects<-[relationship]<-[start_nodes]
```

But SurrealDB's graph traversal operators work differently.

## Solution
Based on SurrealDB documentation (https://surrealdb.com/docs/surrealdb/models/graph), the example shows:
```
<-wrote-.*  means "traverse any node that has a wrote edge pointing to this post"
```

The correct syntax is:
```sql
-- Outbound: Find objects that start_nodes point TO
SELECT * FROM [start_nodes]->relationship->objects

-- Inbound: Find objects that point TO start_nodes  
SELECT * FROM [start_nodes]<-relationship<-objects
```

## Key Insight
The `<-` operator reads from right to left:
- `[start_nodes]<-relationship<-objects` means "find objects that have a relationship edge pointing to start_nodes"
- The start_nodes are on the LEFT, and we traverse backwards to find what points to them

## Implementation
Updated `build_graph_query_string()` in `amp/server/src/handlers/query.rs`:

```rust
GraphDirection::Inbound => {
    // Use <- operator for inbound traversal (find objects that point to start nodes)
    // Syntax: SELECT * FROM [start_nodes]<-relationship<-objects
    if let Some(types) = &graph.relation_types {
        let rel_list = types.iter().map(|t| format!("<-{}", t)).collect::<Vec<_>>().join("");
        format!("SELECT * FROM [{}]{}<-objects", start_ids_list, rel_list)
    } else {
        format!("SELECT * FROM [{}]<-[depends_on, defined_in, calls, justified_by, modifies, implements, produced]<-objects", start_ids_list)
    }
}
```

## Testing
Run the graph traversal test script to verify:
```powershell
cd amp/scripts
./test-graph-traversal.ps1
```

Expected results:
- ✅ Outbound traversal: Find objects that function1 depends on
- ✅ Inbound traversal: Find objects that depend on function2
- ✅ Both directions: Find all connected objects

## References
- SurrealDB Graph Models: https://surrealdb.com/docs/surrealdb/models/graph
- RELATE Statement: https://surrealdb.com/docs/surrealql/statements/relate
