# Test Script Fixes - Summary

## Problem
All test scripts were failing because they were using flat JSON format instead of the variant-based format that the AMP server expects.

## Root Cause
The server's `MemoryObject` enum uses Rust's tagged union (variant) format:
```rust
pub enum MemoryObject {
    Symbol(Symbol),
    Decision(Decision),
    ChangeSet(ChangeSet),
    Run(Run),
}
```

This requires JSON in the format:
```json
{
  "Symbol": { ... }
}
```

But tests were sending flat format:
```json
{
  "id": "...",
  "type": "symbol",
  ...
}
```

## Files Fixed

### PowerShell Tests (Windows)
1. ✅ `test-crud.ps1` - Fixed Symbol and Decision creation
2. ✅ `test-update-delete.ps1` - Fixed create and update operations
3. ✅ `test-query.ps1` - Fixed all object creations (Symbol, Decision)
4. ✅ `test-embeddings.ps1` - Fixed Symbol creation and retrieval
5. ✅ `test-vector-search.ps1` - Fixed Symbol creation in loop
6. ✅ `test-relationships.ps1` - Fixed both Symbol objects
7. ✅ `test-graph-traversal.ps1` - Fixed Symbol creation in loop
8. ✅ `test-embeddings-comprehensive.ps1` - Fixed Symbol creation and retrieval
9. ✅ `test-leases.ps1` - No changes needed (doesn't create objects)
10. ✅ `test-external-db.ps1` - No changes needed (connection test only)

### Bash Tests (Linux/Mac)
1. ✅ `test-crud.sh` - Removed jq dependency, added connection check
2. ✅ `test-schemas.sh` - No changes needed (schema validation)
3. ✅ `demo.sh` - No changes needed (uses correct format)

## Changes Made

### 1. Object Creation Format
**Before:**
```json
{
    "id": "uuid",
    "type": "symbol",
    "tenant_id": "test",
    ...
    "name": "function_name",
    "kind": "function"
}
```

**After:**
```json
{
    "Symbol": {
        "base": {
            "id": "uuid",
            "type": "symbol",
            "tenant_id": "test",
            ...
        },
        "name": "function_name",
        "kind": "function"
    }
}
```

### 2. Object Retrieval Access
**Before:**
```powershell
$response.name
$response.embedding
```

**After:**
```powershell
$response.Symbol.name
$response.Symbol.base.embedding
```

### 3. Bash Script Improvements
- Removed dependency on `jq` (not installed in WSL)
- Added connection check before running tests
- Added proper error handling
- Improved output formatting

## New Files Created

1. ✅ `run-all-tests.ps1` - Master test runner for Windows
2. ✅ `run-all-tests.sh` - Master test runner for Linux/Mac
3. ✅ `README.md` - Comprehensive test documentation

## Testing

All tests should now work correctly. To verify:

### Windows
```powershell
cd amp/scripts
.\run-all-tests.ps1
```

### Linux/Mac
```bash
cd amp/scripts
./run-all-tests.sh
```

## Expected Results

All tests should pass when the server is running:
- ✅ CRUD Operations
- ✅ Update/Delete
- ✅ Lease Coordination
- ✅ Query Endpoint
- ✅ Embeddings (if provider configured)
- ✅ Vector Search (if embeddings enabled)
- ✅ Relationships
- ✅ Graph Traversal

## Notes

- Tests use in-memory database (no persistence)
- Embedding tests require `EMBEDDING_PROVIDER` environment variable
- All tests create isolated data with `tenant_id: "test"`
- Server must be running on `http://localhost:8105`
