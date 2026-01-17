# AMP Test Scripts

This directory contains test scripts for the AMP server. All tests have been updated to use the correct variant-based JSON format.

## Prerequisites

1. **Start the AMP server**:
   ```bash
   cd ../server
   cargo run
   ```

2. The server should be running on `http://localhost:8105`

## Running Tests

### Windows (PowerShell)

Run all tests:
```powershell
cd amp/scripts
.\run-all-tests.ps1
```

Run individual tests:
```powershell
.\test-crud.ps1              # Basic CRUD operations
.\test-update-delete.ps1     # Update and delete operations
.\test-leases.ps1            # Lease coordination
.\test-query.ps1             # Query endpoint
.\test-embeddings.ps1        # Embedding generation
.\test-vector-search.ps1     # Vector similarity search
.\test-relationships.ps1     # Relationship management
.\test-graph-traversal.ps1   # Graph traversal queries
.\test-embeddings-comprehensive.ps1  # Comprehensive embedding tests
.\test-external-db.ps1       # External SurrealDB connection
```

### Linux/Mac (Bash)

Run all tests:
```bash
cd amp/scripts
./run-all-tests.sh
```

Run individual tests:
```bash
./test-crud.sh        # Basic CRUD operations
./test-schemas.sh     # Schema validation
./demo.sh             # Complete demo workflow
```

## Test Coverage

### Core Functionality
- ✅ **CRUD Operations** - Create, read, update, delete memory objects
- ✅ **Batch Operations** - Create multiple objects in one request
- ✅ **Object Types** - Symbol, Decision, ChangeSet, Run

### Advanced Features
- ✅ **Lease Coordination** - Multi-agent resource locking
- ✅ **Query Endpoint** - Text search, filters, pagination
- ✅ **Relationships** - Create and query object relationships
- ✅ **Graph Traversal** - Follow relationship chains
- ✅ **Embeddings** - Auto-generate vector embeddings (requires provider)
- ✅ **Vector Search** - Semantic similarity search (requires embeddings)

### Configuration
- ✅ **External Database** - Connect to external SurrealDB instance

## JSON Format

All tests use the correct variant-based format:

```json
{
  "Symbol": {
    "base": {
      "id": "uuid",
      "type": "symbol",
      "tenant_id": "test",
      "project_id": "project",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z",
      "provenance": {
        "agent": "test",
        "summary": "Test object"
      },
      "links": [],
      "embedding": null
    },
    "name": "function_name",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn function_name()",
    "documentation": "Function description"
  }
}
```

## Troubleshooting

### Server Not Running
```
✗ Server is not running on http://localhost:8105
```
**Solution**: Start the server with `cd ../server && cargo run`

### Connection Refused
```
Failed to connect to localhost:8105
```
**Solution**: Check if the server is running and listening on the correct port

### Test Failures
If tests fail, check:
1. Server logs for errors
2. Database connection (memory vs external)
3. Embedding provider configuration (if testing embeddings)

## Adding New Tests

When creating new test scripts:

1. Use the variant format for all objects (Symbol, Decision, ChangeSet, Run)
2. Include proper error handling
3. Add to the appropriate test runner (run-all-tests.ps1 or run-all-tests.sh)
4. Document the test in this README

## Notes

- Tests use in-memory database by default (no persistence between runs)
- Embedding tests require `EMBEDDING_PROVIDER` environment variable
- External DB tests require SurrealDB running on port 7505
- All tests create objects with `tenant_id: "test"` for isolation
