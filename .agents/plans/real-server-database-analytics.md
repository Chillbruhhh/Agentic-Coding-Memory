# Feature: Real Server and Database Analytics

The following plan should be complete, but its important that you validate documentation and codebase patterns and task sanity before you start implementing.

Pay special attention to naming of existing utils types and models. Import from the right files etc.

## Feature Description

Replace the mock analytics data in the UI with real server and database metrics by implementing a comprehensive analytics endpoint that provides live system information including object counts, relationship statistics, server performance metrics, database health, and recent activity logs.

## User Story

As a developer using the AMP system
I want to see real-time analytics about my server and database performance
So that I can monitor system health, track usage patterns, and identify potential issues

## Problem Statement

The current Analytics tab in the UI displays only mock/static data that doesn't reflect the actual state of the AMP server or database. Users cannot monitor real system performance, object counts, relationship statistics, or identify issues in their AMP deployment.

## Solution Statement

Implement a new `/v1/analytics` endpoint in the AMP server that aggregates real-time data from SurrealDB and system metrics, then update the UI to consume this live data instead of mock data. The analytics will include object statistics, relationship counts, system performance metrics, recent activity, and database health indicators.

## Feature Metadata

**Feature Type**: New Capability
**Estimated Complexity**: Medium
**Primary Systems Affected**: Server (new analytics handler), UI (Analytics component update)
**Dependencies**: SurrealDB queries, system metrics collection, existing server infrastructure

---

## CONTEXT REFERENCES

### Relevant Codebase Files IMPORTANT: YOU MUST READ THESE FILES BEFORE IMPLEMENTING!

- `amp/server/src/handlers/mod.rs` (line 6) - Why: Shows handler module pattern to follow for new analytics handler
- `amp/server/src/handlers/objects.rs` (lines 1-30) - Why: Contains handler pattern with State extraction and error handling
- `amp/server/src/main.rs` (lines 80-100) - Why: Shows how to register new routes in api_routes() function
- `amp/server/src/database.rs` (lines 1-50) - Why: Database connection pattern and query execution methods
- `amp/server/src/config.rs` - Why: Configuration structure and environment variable handling
- `amp/ui/src/hooks/useAnalytics.ts` - Why: Current analytics data structure and API call pattern
- `amp/ui/src/components/Analytics.tsx` - Why: UI component that needs to display real data
- `amp/spec/schema.surql` (lines 1-50) - Why: Database schema for understanding table structure and indexes

### New Files to Create

- `amp/server/src/handlers/analytics.rs` - Analytics endpoint handler with system metrics collection
- `amp/server/src/services/analytics.rs` - Analytics service for data aggregation and system metrics
- `amp/server/src/models/analytics.rs` - Analytics data models and response structures

### Relevant Documentation YOU SHOULD READ THESE BEFORE IMPLEMENTING!

- [SurrealDB Aggregation Functions](https://surrealdb.com/docs/surrealql/functions/aggregation)
  - Specific section: COUNT, GROUP BY operations
  - Why: Required for counting objects by type and calculating statistics
- [Rust sysinfo crate](https://docs.rs/sysinfo/latest/sysinfo/)
  - Specific section: System, Process, Memory usage
  - Why: Needed for collecting real system performance metrics
- [Axum Handler Documentation](https://docs.rs/axum/latest/axum/handler/index.html)
  - Specific section: Handler functions and State extraction
  - Why: Shows proper async handler implementation patterns

### Patterns to Follow

**Handler Pattern:**
```rust
use axum::{extract::State, http::StatusCode, response::Json};
use tokio::time::{timeout, Duration};

pub async fn handler_name(
    State(state): State<AppState>,
) -> Result<Json<ResponseType>, StatusCode> {
    let result = timeout(
        Duration::from_secs(5),
        actual_work(&state)
    ).await
    .map_err(|_| StatusCode::REQUEST_TIMEOUT)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(result))
}
```

**Database Query Pattern:**
```rust
let query = "SELECT count() FROM objects GROUP BY type";
let mut result = self.client.query(query).await?;
let counts: Vec<ObjectCount> = result.take(0)?;
```

**Error Handling Pattern:**
```rust
.map_err(|e| {
    tracing::error!("Operation failed: {}", e);
    StatusCode::INTERNAL_SERVER_ERROR
})
```

**Route Registration Pattern:**
```rust
// In main.rs api_routes() function
.route("/analytics", get(handlers::analytics::get_analytics))
```

---

## IMPLEMENTATION PLAN

### Phase 1: Foundation

Set up analytics data models and service infrastructure before implementing the endpoint.

**Tasks:**
- Create analytics data models matching the existing AnalyticsData interface
- Set up analytics service with system metrics collection capabilities
- Add sysinfo dependency for system performance monitoring

### Phase 2: Core Implementation

Implement the analytics service with real data collection from SurrealDB and system metrics.

**Tasks:**
- Implement database queries for object counts, relationship statistics, and recent activity
- Add system metrics collection (CPU, memory, disk usage, uptime)
- Create analytics aggregation service that combines all data sources

### Phase 3: Integration

Connect the analytics service to the HTTP API and update the UI to consume real data.

**Tasks:**
- Create analytics HTTP handler endpoint
- Register the new route in the server
- Update UI to remove mock data fallback and handle real server responses
- Add proper error handling for analytics endpoint failures

### Phase 4: Testing & Validation

Ensure the analytics endpoint works correctly and provides accurate data.

**Tasks:**
- Test analytics endpoint with various database states
- Validate system metrics accuracy
- Test UI integration with real server data
- Add error handling for server unavailable scenarios

---

## STEP-BY-STEP TASKS

IMPORTANT: Execute every task in order, top to bottom. Each task is atomic and independently testable.

### CREATE amp/server/src/models/analytics.rs

- **IMPLEMENT**: Analytics data models matching UI interface
- **PATTERN**: Follow existing model patterns in `amp/server/src/models/mod.rs`
- **IMPORTS**: `serde::{Deserialize, Serialize}`, `std::collections::HashMap`, `chrono::{DateTime, Utc}`
- **GOTCHA**: Use exact same field names as AnalyticsData interface in UI
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### UPDATE amp/server/src/models/mod.rs

- **IMPLEMENT**: Add `pub mod analytics;` to expose analytics models
- **PATTERN**: Mirror existing module declarations
- **IMPORTS**: None needed
- **GOTCHA**: Must be added to make analytics models available to handlers
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### UPDATE amp/server/Cargo.toml

- **IMPLEMENT**: Add `sysinfo = "0.30"` to dependencies section
- **PATTERN**: Follow existing dependency format in Cargo.toml
- **IMPORTS**: None needed
- **GOTCHA**: Use compatible version that works with current Rust version
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### CREATE amp/server/src/services/analytics.rs

- **IMPLEMENT**: AnalyticsService struct with data collection methods
- **PATTERN**: Follow service pattern from `amp/server/src/services/storage.rs`
- **IMPORTS**: `sysinfo::System`, `std::sync::Arc`, `crate::database::Database`, analytics models
- **GOTCHA**: Initialize sysinfo System once and refresh before each use
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### UPDATE amp/server/src/services/mod.rs

- **IMPLEMENT**: Add `pub mod analytics;` to expose analytics service
- **PATTERN**: Mirror existing service module declarations
- **IMPORTS**: None needed
- **GOTCHA**: Required to make AnalyticsService available to handlers
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### CREATE amp/server/src/handlers/analytics.rs

- **IMPLEMENT**: `get_analytics` handler function with proper error handling
- **PATTERN**: Follow handler pattern from `amp/server/src/handlers/objects.rs` (lines 1-30)
- **IMPORTS**: `axum::{extract::State, http::StatusCode, response::Json}`, `tokio::time::{timeout, Duration}`
- **GOTCHA**: Use 5-second timeout pattern consistent with other handlers
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### UPDATE amp/server/src/handlers/mod.rs

- **IMPLEMENT**: Add `pub mod analytics;` to expose analytics handler
- **PATTERN**: Follow existing handler module pattern (line 6)
- **IMPORTS**: None needed
- **GOTCHA**: Must be added before handler can be used in main.rs
- **VALIDATE**: `cargo check --manifest-path amp/server/Cargo.toml`

### UPDATE amp/server/src/main.rs

- **IMPLEMENT**: Add analytics service to AppState and register route
- **PATTERN**: Follow existing service initialization and route registration (lines 80-100)
- **IMPORTS**: `services::analytics::AnalyticsService`
- **GOTCHA**: Initialize analytics service after database connection is established
- **VALIDATE**: `cargo run --manifest-path amp/server/Cargo.toml` (should start without errors)

### UPDATE amp/ui/src/hooks/useAnalytics.ts

- **IMPLEMENT**: Remove mock data fallback and use only real server data
- **PATTERN**: Keep existing fetch pattern but remove mock analytics generation
- **IMPORTS**: None needed
- **GOTCHA**: Handle server unavailable gracefully with proper error messages
- **VALIDATE**: `npm run type-check` in `amp/ui` directory

### UPDATE amp/ui/src/components/Analytics.tsx

- **IMPLEMENT**: Update to handle real data structure and add refresh functionality
- **PATTERN**: Keep existing UI layout but ensure data mapping works with real server response
- **IMPORTS**: None needed
- **GOTCHA**: Ensure all data fields are properly mapped from real analytics response
- **VALIDATE**: `npm run build` in `amp/ui` directory

---

## TESTING STRATEGY

### Unit Tests

Test analytics service data collection methods independently:
- Database query aggregation accuracy
- System metrics collection functionality
- Error handling for database connection failures
- Data model serialization/deserialization

### Integration Tests

Test full analytics endpoint workflow:
- HTTP endpoint returns proper JSON structure
- Database queries execute within timeout limits
- System metrics are collected successfully
- Error responses for various failure scenarios

### Edge Cases

- Empty database (no objects or relationships)
- Database connection timeout during analytics collection
- System metrics collection failure
- Large dataset performance (1000+ objects)
- Concurrent analytics requests

---

## VALIDATION COMMANDS

Execute every command to ensure zero regressions and 100% feature correctness.

### Level 1: Syntax & Style

```bash
# Rust formatting and linting
cd amp/server && cargo fmt --check
cd amp/server && cargo clippy -- -D warnings
cd amp/ui && npm run type-check
cd amp/ui && npm run lint
```

### Level 2: Unit Tests

```bash
# Rust unit tests
cd amp/server && cargo test

# TypeScript compilation
cd amp/ui && npm run build
```

### Level 3: Integration Tests

```bash
# Start server and test analytics endpoint
cd amp/server && cargo run &
sleep 5
curl -X GET http://localhost:8105/v1/analytics | jq .
curl -X GET http://localhost:8105/health | jq .
pkill -f "amp-server"
```

### Level 4: Manual Validation

```bash
# Full system test
cd amp/server && cargo run &
cd amp/ui && npm run dev &
# Open http://localhost:1420 and verify Analytics tab shows real data
# Verify refresh button updates data
# Check browser console for errors
```

### Level 5: Additional Validation (Optional)

```bash
# Performance test with multiple requests
for i in {1..10}; do curl -s http://localhost:8105/v1/analytics > /dev/null & done
wait
echo "All requests completed"
```

---

## ACCEPTANCE CRITERIA

- [ ] Analytics endpoint returns real object counts from database
- [ ] System metrics (CPU, memory, disk) are collected and displayed accurately
- [ ] Recent activity shows actual database operations with timestamps
- [ ] UI Analytics tab displays live data instead of mock data
- [ ] Refresh functionality updates data from server
- [ ] Error handling works when server is unavailable
- [ ] All validation commands pass with zero errors
- [ ] Analytics endpoint responds within 5-second timeout
- [ ] No performance degradation on existing endpoints
- [ ] UI maintains cyberpunk theme with real data

---

## COMPLETION CHECKLIST

- [ ] All tasks completed in order
- [ ] Each task validation passed immediately
- [ ] All validation commands executed successfully
- [ ] Full test suite passes (unit + integration)
- [ ] No linting or type checking errors
- [ ] Manual testing confirms feature works
- [ ] Acceptance criteria all met
- [ ] Code reviewed for quality and maintainability

---

## NOTES

**Design Decisions:**
- Using sysinfo crate for cross-platform system metrics collection
- Maintaining 5-second timeout pattern consistent with other endpoints
- Preserving existing UI layout and theme while adding real data
- Graceful degradation when server is unavailable

**Performance Considerations:**
- Analytics queries use efficient SurrealDB aggregation functions
- System metrics collection is lightweight and cached
- Analytics endpoint can handle concurrent requests

**Security Considerations:**
- Analytics endpoint follows same localhost-only binding as other endpoints
- No sensitive system information exposed beyond basic performance metrics
- Database queries use parameterized patterns to prevent injection
