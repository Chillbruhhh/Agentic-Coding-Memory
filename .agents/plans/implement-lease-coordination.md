# Feature: Lease Coordination Endpoints

The following plan should be complete, but it's important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Implement lease-based coordination primitives that enable multiple AI agents to safely coordinate access to shared resources. Leases provide a time-bound exclusive lock mechanism that prevents conflicts when multiple agents attempt to modify the same code, files, or memory objects simultaneously.

## User Story

As an AI agent developer
I want to acquire and release leases on shared resources
So that multiple agents can coordinate safely without conflicts or race conditions

## Problem Statement

When multiple AI agents work on the same codebase or memory system simultaneously, they can create conflicts:
1. Two agents modifying the same file at the same time
2. Concurrent updates to the same memory object causing data loss
3. Race conditions in multi-agent workflows
4. No mechanism to signal "I'm working on this resource"

Without coordination primitives, agents either:
- Overwrite each other's work (last write wins)
- Need complex external coordination systems
- Cannot safely work in parallel

## Solution Statement

Implement a lease-based coordination system with three endpoints:
1. **POST /v1/leases/acquire** - Acquire exclusive access to a resource
2. **POST /v1/leases/release** - Release a held lease
3. **POST /v1/leases/renew** - Extend lease duration (bonus)

Leases will:
- Have configurable TTL (time-to-live) with default of 5 minutes
- Automatically expire to prevent deadlocks from crashed agents
- Store in SurrealDB `leases` table
- Return 409 Conflict if resource already leased
- Support lease renewal for long-running operations

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: Medium
**Primary Systems Affected**:
- API handlers (`server/src/handlers/leases.rs`)
- Database schema (`spec/schema.surql` - already defined)
**Dependencies**: SurrealDB leases table (already in schema)

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/handlers/leases.rs` (lines 1-55) - Why: Contains placeholder implementations and request/response types
- `amp/server/src/handlers/objects.rs` (lines 1-250) - Why: Pattern for timeout handling, error responses, database operations
- `amp/spec/schema.surql` (search for "leases") - Why: Database schema for leases table
- `amp/server/src/database.rs` (lines 1-45) - Why: Database client usage patterns
- `amp/server/src/main.rs` (lines 60-75) - Why: Router registration (already has lease routes)

### New Files to Create

None - all changes are to existing `leases.rs` file

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [SurrealDB Rust SDK - Create](https://surrealdb.com/docs/sdk/rust/methods/create)
  - Specific section: `.create()` with record ID
  - Why: For creating lease records
- [SurrealDB Rust SDK - Select](https://surrealdb.com/docs/sdk/rust/methods/select)
  - Specific section: Querying records
  - Why: For checking existing leases
- [SurrealDB Rust SDK - Delete](https://surrealdb.com/docs/sdk/rust/methods/delete)
  - Specific section: `.delete()` method
  - Why: For releasing leases
- [Chrono Duration](https://docs.rs/chrono/latest/chrono/struct.Duration.html)
  - Specific section: Adding duration to DateTime
  - Why: For calculating lease expiration times

### Patterns to Follow

**Timeout Pattern:**
```rust
// From objects.rs
let result: Result<Result<Option<Value>, _>, _> = timeout(
    Duration::from_secs(5),
    state.db.client.select(("table", id.to_string()))
).await;
```

**Error Handling Pattern:**
```rust
// From objects.rs
match result {
    Ok(Ok(Some(data))) => { /* success */ },
    Ok(Ok(None)) => Err(StatusCode::NOT_FOUND),
    Ok(Err(e)) => {
        tracing::error!("Failed to X: {}", e);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
    Err(_) => {
        tracing::error!("Timeout");
        Err(StatusCode::GATEWAY_TIMEOUT)
    }
}
```

**Database Schema (from schema.surql):**
```sql
DEFINE TABLE leases SCHEMAFULL;
DEFINE FIELD id ON leases TYPE record<leases>;
DEFINE FIELD resource ON leases TYPE string;
DEFINE FIELD holder ON leases TYPE string;
DEFINE FIELD expires_at ON leases TYPE datetime;
DEFINE FIELD created_at ON leases TYPE datetime DEFAULT time::now();
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Update the lease data structures to match database schema and add helper functions.

### Phase 2: Core Implementation

Implement the three lease operations:
1. `acquire_lease()` - Check for existing lease, create if available
2. `release_lease()` - Delete lease by ID
3. `renew_lease()` - Update expiration time (bonus feature)

### Phase 3: Integration

- Routes already registered in main.rs
- Add renew route if implementing renewal

### Phase 4: Testing & Validation

Create test script that:
- Acquires a lease successfully
- Attempts to acquire same resource (should fail with 409)
- Releases the lease
- Acquires again (should succeed)
- Tests automatic expiration

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### UPDATE `amp/server/src/handlers/leases.rs`

- **ADD**: Import statements at top of file
- **IMPORTS**: Add timeout and Duration from tokio
  ```rust
  use tokio::time::{timeout, Duration};
  use serde_json::Value;
  use surrealdb::sql::Datetime as SurrealDatetime;
  ```
- **VALIDATE**: `cargo check` should pass

### UPDATE `amp/server/src/handlers/leases.rs`

- **ADD**: Internal lease structure for database operations
- **LOCATION**: After the existing structs
- **IMPLEMENTATION**:
  ```rust
  #[derive(Debug, Serialize, Deserialize)]
  struct LeaseRecord {
      resource: String,
      holder: String,
      expires_at: SurrealDatetime,
      created_at: SurrealDatetime,
  }
  ```
- **GOTCHA**: Use SurrealDatetime for proper database serialization
- **VALIDATE**: `cargo check` should pass

### UPDATE `amp/server/src/handlers/leases.rs`

- **REPLACE**: `acquire_lease` function with full implementation
- **PATTERN**: Mirror objects.rs timeout and error handling patterns
- **IMPLEMENTATION**:
  ```rust
  pub async fn acquire_lease(
      State(state): State<AppState>,
      Json(request): Json<LeaseRequest>,
  ) -> Result<(StatusCode, Json<LeaseResponse>), StatusCode> {
      let lease_id = Uuid::new_v4();
      let ttl_seconds = request.ttl_seconds.unwrap_or(300); // Default 5 minutes
      
      // Check for existing lease on this resource
      let query = format!(
          "SELECT * FROM leases WHERE resource = '{}' AND expires_at > time::now()",
          request.resource.replace("'", "\\'") // Escape single quotes
      );
      
      let check_result: Result<Result<Vec<Value>, _>, _> = timeout(
          Duration::from_secs(5),
          state.db.client.query(query)
      ).await;

      match check_result {
          Ok(Ok(results)) => {
              if !results.is_empty() {
                  tracing::warn!("Lease conflict for resource: {}", request.resource);
                  return Err(StatusCode::CONFLICT);
              }
          }
          Ok(Err(e)) => {
              tracing::error!("Failed to check existing leases: {}", e);
              return Err(StatusCode::INTERNAL_SERVER_ERROR);
          }
          Err(_) => {
              tracing::error!("Timeout checking leases");
              return Err(StatusCode::GATEWAY_TIMEOUT);
          }
      }

      // Calculate expiration
      let now = chrono::Utc::now();
      let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);
      let surreal_expires = SurrealDatetime::from(expires_at);
      let surreal_created = SurrealDatetime::from(now);

      // Create lease record
      let lease_record = LeaseRecord {
          resource: request.resource.clone(),
          holder: request.holder.clone(),
          expires_at: surreal_expires,
          created_at: surreal_created,
      };

      let create_result: Result<Result<Option<Value>, _>, _> = timeout(
          Duration::from_secs(5),
          state.db.client
              .create(("leases", lease_id.to_string()))
              .content(lease_record)
      ).await;

      match create_result {
          Ok(Ok(_)) => {
              tracing::info!("Lease acquired: {} by {}", request.resource, request.holder);
              Ok((
                  StatusCode::CREATED,
                  Json(LeaseResponse {
                      lease_id,
                      resource: request.resource,
                      holder: request.holder,
                      expires_at,
                  }),
              ))
          }
          Ok(Err(e)) => {
              tracing::error!("Failed to create lease: {}", e);
              Err(StatusCode::INTERNAL_SERVER_ERROR)
          }
          Err(_) => {
              tracing::error!("Timeout creating lease");
              Err(StatusCode::GATEWAY_TIMEOUT)
          }
      }
  }
  ```
- **GOTCHA**: Must check for existing leases before creating new one
- **GOTCHA**: Escape single quotes in resource names to prevent SQL injection
- **VALIDATE**: `cargo check` should pass

### UPDATE `amp/server/src/handlers/leases.rs`

- **REPLACE**: `release_lease` function with full implementation
- **PATTERN**: Mirror delete_object pattern from objects.rs
- **IMPLEMENTATION**:
  ```rust
  pub async fn release_lease(
      State(state): State<AppState>,
      Json(request): Json<ReleaseRequest>,
  ) -> Result<StatusCode, StatusCode> {
      let result: Result<Result<Option<Value>, _>, _> = timeout(
          Duration::from_secs(5),
          state.db.client.delete(("leases", request.lease_id.to_string()))
      ).await;

      match result {
          Ok(Ok(Some(_))) => {
              tracing::info!("Lease released: {}", request.lease_id);
              Ok(StatusCode::OK)
          }
          Ok(Ok(None)) => {
              tracing::warn!("Lease not found: {}", request.lease_id);
              Err(StatusCode::NOT_FOUND)
          }
          Ok(Err(e)) => {
              tracing::error!("Failed to release lease {}: {}", request.lease_id, e);
              Err(StatusCode::INTERNAL_SERVER_ERROR)
          }
          Err(_) => {
              tracing::error!("Timeout releasing lease {}", request.lease_id);
              Err(StatusCode::GATEWAY_TIMEOUT)
          }
      }
  }
  ```
- **VALIDATE**: `cargo check` should pass

### UPDATE `amp/server/src/handlers/leases.rs`

- **ADD**: Renew lease function (bonus feature)
- **LOCATION**: After release_lease function
- **IMPLEMENTATION**:
  ```rust
  #[derive(Debug, Deserialize)]
  pub struct RenewRequest {
      pub lease_id: Uuid,
      pub ttl_seconds: Option<u64>,
  }

  pub async fn renew_lease(
      State(state): State<AppState>,
      Json(request): Json<RenewRequest>,
  ) -> Result<(StatusCode, Json<LeaseResponse>), StatusCode> {
      let ttl_seconds = request.ttl_seconds.unwrap_or(300);
      
      // Get existing lease
      let get_result: Result<Result<Option<Value>, _>, _> = timeout(
          Duration::from_secs(5),
          state.db.client.select(("leases", request.lease_id.to_string()))
      ).await;

      let lease_data = match get_result {
          Ok(Ok(Some(data))) => data,
          Ok(Ok(None)) => return Err(StatusCode::NOT_FOUND),
          Ok(Err(e)) => {
              tracing::error!("Failed to get lease: {}", e);
              return Err(StatusCode::INTERNAL_SERVER_ERROR);
          }
          Err(_) => {
              tracing::error!("Timeout getting lease");
              return Err(StatusCode::GATEWAY_TIMEOUT);
          }
      };

      // Extract resource and holder
      let resource = lease_data.get("resource")
          .and_then(|v| v.as_str())
          .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
          .to_string();
      let holder = lease_data.get("holder")
          .and_then(|v| v.as_str())
          .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
          .to_string();

      // Calculate new expiration
      let now = chrono::Utc::now();
      let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);
      let surreal_expires = SurrealDatetime::from(expires_at);

      // Update expiration using query
      let update_query = format!(
          "UPDATE leases:{} SET expires_at = $expires",
          request.lease_id
      );
      
      let update_result: Result<Result<Vec<Value>, _>, _> = timeout(
          Duration::from_secs(5),
          state.db.client.query(update_query).bind(("expires", surreal_expires))
      ).await;

      match update_result {
          Ok(Ok(_)) => {
              tracing::info!("Lease renewed: {}", request.lease_id);
              Ok((
                  StatusCode::OK,
                  Json(LeaseResponse {
                      lease_id: request.lease_id,
                      resource,
                      holder,
                      expires_at,
                  }),
              ))
          }
          Ok(Err(e)) => {
              tracing::error!("Failed to renew lease: {}", e);
              Err(StatusCode::INTERNAL_SERVER_ERROR)
          }
          Err(_) => {
              tracing::error!("Timeout renewing lease");
              Err(StatusCode::GATEWAY_TIMEOUT)
          }
      }
  }
  ```
- **VALIDATE**: `cargo check` should pass

### UPDATE `amp/server/src/main.rs`

- **ADD**: Route for renew endpoint (if implementing renewal)
- **LOCATION**: After existing lease routes
- **IMPLEMENTATION**:
  ```rust
  .route("/leases/renew", post(handlers::leases::renew_lease))
  ```
- **VALIDATE**: `cargo build` should compile

### CREATE `amp/scripts/test-leases.ps1`

- **CREATE**: Test script for lease coordination
- **PATTERN**: Mirror test-update-delete.ps1 structure
- **IMPLEMENTATION**:
  ```powershell
  # AMP Lease Coordination Test Script
  $BASE_URL = "http://localhost:8105"

  Write-Host "=== AMP Lease Coordination Test ===" -ForegroundColor Cyan

  # 1. Acquire a lease
  Write-Host "`n1. Acquiring lease on resource 'test-file.rs'..." -ForegroundColor Yellow
  $acquireData = @"
  {
      "resource": "test-file.rs",
      "holder": "agent-1",
      "ttl_seconds": 60
  }
  "@

  try {
      $response = Invoke-RestMethod -Uri "$BASE_URL/v1/leases/acquire" -Method Post -Body $acquireData -ContentType "application/json"
      $leaseId = $response.lease_id
      Write-Host "Lease acquired: $leaseId" -ForegroundColor Green
      Write-Host "Expires at: $($response.expires_at)" -ForegroundColor Gray
  } catch {
      Write-Host "Failed to acquire lease: $_" -ForegroundColor Red
      exit 1
  }

  # 2. Try to acquire same resource (should fail)
  Write-Host "`n2. Attempting to acquire same resource (should fail)..." -ForegroundColor Yellow
  $conflictData = @"
  {
      "resource": "test-file.rs",
      "holder": "agent-2",
      "ttl_seconds": 60
  }
  "@

  try {
      Invoke-RestMethod -Uri "$BASE_URL/v1/leases/acquire" -Method Post -Body $conflictData -ContentType "application/json"
      Write-Host "ERROR: Should have gotten 409 Conflict!" -ForegroundColor Red
  } catch {
      if ($_.Exception.Response.StatusCode -eq 409) {
          Write-Host "Correctly rejected (409 Conflict)" -ForegroundColor Green
      } else {
          Write-Host "Unexpected error: $_" -ForegroundColor Red
      }
  }

  # 3. Renew the lease (if implemented)
  Write-Host "`n3. Renewing lease..." -ForegroundColor Yellow
  $renewData = @"
  {
      "lease_id": "$leaseId",
      "ttl_seconds": 120
  }
  "@

  try {
      $response = Invoke-RestMethod -Uri "$BASE_URL/v1/leases/renew" -Method Post -Body $renewData -ContentType "application/json"
      Write-Host "Lease renewed, new expiration: $($response.expires_at)" -ForegroundColor Green
  } catch {
      Write-Host "Renew not implemented or failed: $_" -ForegroundColor Yellow
  }

  # 4. Release the lease
  Write-Host "`n4. Releasing lease..." -ForegroundColor Yellow
  $releaseData = @"
  {
      "lease_id": "$leaseId"
  }
  "@

  try {
      Invoke-RestMethod -Uri "$BASE_URL/v1/leases/release" -Method Post -Body $releaseData -ContentType "application/json"
      Write-Host "Lease released successfully" -ForegroundColor Green
  } catch {
      Write-Host "Failed to release: $_" -ForegroundColor Red
      exit 1
  }

  # 5. Acquire again (should succeed now)
  Write-Host "`n5. Acquiring same resource again (should succeed)..." -ForegroundColor Yellow
  try {
      $response = Invoke-RestMethod -Uri "$BASE_URL/v1/leases/acquire" -Method Post -Body $acquireData -ContentType "application/json"
      Write-Host "Successfully acquired: $($response.lease_id)" -ForegroundColor Green
      
      # Clean up
      $cleanupData = @"
  {
      "lease_id": "$($response.lease_id)"
  }
  "@
      Invoke-RestMethod -Uri "$BASE_URL/v1/leases/release" -Method Post -Body $cleanupData -ContentType "application/json" | Out-Null
  } catch {
      Write-Host "Failed to re-acquire: $_" -ForegroundColor Red
      exit 1
  }

  Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
  ```
- **VALIDATE**: Run after server is running

---

## TESTING STRATEGY

### Unit Tests

Not required - handlers are thin wrappers. Integration tests provide better coverage.

### Integration Tests

PowerShell script tests:
1. Acquire lease successfully
2. Conflict detection (409 when resource already leased)
3. Lease renewal (if implemented)
4. Lease release
5. Re-acquisition after release

### Edge Cases

Test manually:
1. **Expired lease** - Wait for TTL to expire, should be able to acquire
2. **Invalid lease ID** - Release non-existent lease (404)
3. **Concurrent acquisition** - Two agents trying simultaneously
4. **Long resource names** - Test with 255+ character resource names
5. **Special characters** - Resource names with quotes, slashes, etc.

---

## VALIDATION COMMANDS

### Level 1: Compilation

```bash
cd amp/server
cargo build
```

### Level 2: Server Start

```bash
cd amp/server
cargo run
# Should start without errors
```

### Level 3: Manual Validation

```bash
cd amp/scripts
./test-leases.ps1
# All tests should pass
```

### Level 4: Regression Testing

```bash
# Verify existing endpoints still work
./test-crud.ps1
./test-update-delete.ps1
```

---

## ACCEPTANCE CRITERIA

- [ ] POST /v1/leases/acquire endpoint implemented
- [ ] POST /v1/leases/release endpoint implemented
- [ ] POST /v1/leases/renew endpoint implemented (bonus)
- [ ] Leases stored in SurrealDB leases table
- [ ] Conflict detection returns 409 when resource already leased
- [ ] Automatic expiration via expires_at timestamp
- [ ] 5-second timeouts on all database operations
- [ ] Comprehensive error logging
- [ ] Test script validates full workflow
- [ ] No regressions in existing endpoints

---

## COMPLETION CHECKLIST

- [ ] LeaseRecord struct added
- [ ] acquire_lease fully implemented
- [ ] release_lease fully implemented
- [ ] renew_lease implemented (bonus)
- [ ] Renew route added to main.rs (if implementing)
- [ ] Test script created
- [ ] cargo build passes
- [ ] Server starts successfully
- [ ] Test script passes all checks
- [ ] Existing tests still pass

---

## NOTES

### Design Decisions

1. **Default TTL of 5 minutes**: Balances between too short (frequent renewals) and too long (slow recovery from crashes)

2. **Query-based conflict check**: Uses SurrealDB query to check for existing non-expired leases. More flexible than trying to create and catching errors.

3. **Automatic expiration**: Relies on `expires_at` timestamp. Expired leases remain in database but are ignored. Could add cleanup job later.

4. **No lease ownership validation**: Any agent can release any lease. For MVP simplicity. Production would validate holder matches.

5. **SQL injection prevention**: Escapes single quotes in resource names to prevent injection attacks.

### Performance Considerations

- Lease checks require a query (not just a lookup)
- Could add index on `resource` field for faster lookups
- Expired leases accumulate - consider periodic cleanup

### Security Considerations

- No authentication yet - any agent can acquire/release any lease
- Resource names should be validated/sanitized
- Consider rate limiting to prevent lease exhaustion attacks

### Future Enhancements

- Lease ownership validation (only holder can release/renew)
- Automatic cleanup of expired leases
- Lease status endpoint (GET /v1/leases/{resource})
- Lease listing endpoint (GET /v1/leases)
- Lease transfer between agents
- Priority-based lease acquisition
- Deadlock detection
