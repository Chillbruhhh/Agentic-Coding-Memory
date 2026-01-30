# API Overview

Complete REST API reference for AMP server.

## Base URL

```
http://localhost:8105
```

## Authentication

Currently, AMP uses localhost-only binding for security. No authentication required for local development.

For production deployment with remote access, implement authentication at the reverse proxy level.

## Request Format

All requests use JSON:

```http
POST /v1/objects
Content-Type: application/json

{
  "type": "symbol",
  "name": "my_function",
  ...
}
```

## Response Format

### Success Response

```json
{
  "id": "uuid-here",
  "type": "symbol",
  "created_at": "2026-01-27T10:30:00Z",
  ...
}
```

### Error Response

```json
{
  "error": "Object not found",
  "code": "NOT_FOUND",
  "details": {
    "object_id": "uuid-here"
  }
}
```

## HTTP Status Codes

- `200 OK` - Successful GET/PUT request
- `201 Created` - Successful POST request
- `204 No Content` - Successful DELETE request
- `400 Bad Request` - Invalid request data
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource conflict (e.g., lease already held)
- `500 Internal Server Error` - Server error

## API Endpoints

### Core Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/v1/objects` | Create object |
| POST | `/v1/objects/batch` | Batch create objects |
| GET | `/v1/objects/{id}` | Get object by ID |
| PUT | `/v1/objects/{id}` | Update object |
| DELETE | `/v1/objects/{id}` | Delete object |

### Query & Search

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/query` | Hybrid search (vector + graph + temporal) |
| GET | `/v1/trace/{id}` | Object provenance and relationships |

> **Note:** Hybrid queries use [Reciprocal Rank Fusion (RRF)](../concepts/hybrid-retrieval.md) to combine results from vector search, graph traversal, and temporal filtering into a unified ranking.

### Relationships

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/relationships` | Create relationship |
| GET | `/v1/relationships` | Query relationships |
| DELETE | `/v1/relationships/{type}/{id}` | Delete relationship |

### Codebase Intelligence

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/codebase/parse` | Parse entire codebase |
| POST | `/v1/codebase/parse-file` | Parse single file |
| POST | `/v1/codebase/delete` | Delete codebase data |
| POST | `/v1/codebase/sync` | Sync file (file_sync MCP tool) |
| GET | `/v1/codebase/file-logs` | List all file logs |
| GET | `/v1/codebase/file-logs/{path}` | Get file log by path |
| GET | `/v1/codebase/file-log-objects/{path}` | Get file log object |
| GET | `/v1/codebase/file-contents/{path}` | Get file content |
| POST | `/v1/codebase/update-file-log` | Update file log |
| POST | `/v1/codebase/ai-file-log` | Generate AI file log |

### Artifacts

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/artifacts` | Write artifact (decision, note, changeset) |
| GET | `/v1/artifacts` | List artifacts |
| DELETE | `/v1/artifacts/{id}` | Delete artifact |

### Cache Operations (Legacy)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/cache/pack` | Get cache pack |
| POST | `/v1/cache/write` | Write cache items |
| POST | `/v1/cache/gc` | Garbage collect cache |

### Cache Block (Episodic Memory)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/cache/block/write` | Write to block cache |
| POST | `/v1/cache/block/compact` | Compact block cache |
| POST | `/v1/cache/block/search` | Search block cache |
| GET | `/v1/cache/block/current/{scope_id}` | Get current block for scope |
| GET | `/v1/cache/block/{id}` | Get block by ID |

### Coordination (Leases)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/leases/acquire` | Acquire resource lease |
| POST | `/v1/leases/release` | Release resource lease |
| POST | `/v1/leases/renew` | Renew resource lease |

### Connections (Agent Tracking)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/connections/register` | Register agent connection |
| POST | `/v1/connections/heartbeat` | Send heartbeat |
| POST | `/v1/connections/disconnect` | Disconnect agent |
| GET | `/v1/connections` | List active connections |
| POST | `/v1/connections/cleanup` | Cleanup expired connections |

### System

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/v1/analytics` | System analytics and metrics |
| GET | `/v1/settings` | Get server settings |
| PUT | `/v1/settings` | Update server settings |

## Common Patterns

### Creating Objects

```bash
curl -X POST http://localhost:8105/v1/objects \
  -H "Content-Type: application/json" \
  -d '{
    "type": "symbol",
    "tenant_id": "my-org",
    "project_id": "my-project",
    "provenance": {
      "agent": "my-agent",
      "summary": "Created via API"
    },
    "name": "my_function",
    "kind": "function",
    "path": "src/main.py",
    "language": "python"
  }'
```

### Querying

```bash
curl -X POST http://localhost:8105/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "authentication",
    "hybrid": true,
    "limit": 10,
    "filters": {
      "object_types": ["symbol"],
      "project_id": "my-project"
    }
  }'
```

### Batch Operations

```bash
curl -X POST http://localhost:8105/v1/objects/batch \
  -H "Content-Type: application/json" \
  -d '{
    "objects": [
      {
        "type": "symbol",
        "name": "func1",
        ...
      },
      {
        "type": "symbol",
        "name": "func2",
        ...
      }
    ]
  }'
```

## Rate Limiting

Currently no rate limiting. For production:
- Implement rate limiting at reverse proxy
- Recommended: 100 requests/minute per IP

## Pagination

For endpoints returning lists:

```json
{
  "limit": 50,
  "offset": 0
}
```

Response includes:
```json
{
  "results": [...],
  "total": 150,
  "limit": 50,
  "offset": 0
}
```

## Filtering

Most query endpoints support filters:

```json
{
  "filters": {
    "object_types": ["symbol", "decision"],
    "project_id": "my-project",
    "tenant_id": "my-org",
    "created_after": "2026-01-01T00:00:00Z",
    "created_before": "2026-01-31T23:59:59Z"
  }
}
```

## Sorting

Specify sort order:

```json
{
  "sort": {
    "field": "created_at",
    "order": "desc"
  }
}
```

## Field Selection

Request specific fields only:

```json
{
  "fields": ["id", "name", "type", "created_at"]
}
```

## Timeouts

All operations have 5-second timeout. For long-running operations:
- Use batch endpoints
- Implement client-side retry logic
- Consider async processing

## Error Handling

### Client Errors (4xx)

```json
{
  "error": "Validation failed",
  "code": "VALIDATION_ERROR",
  "details": {
    "field": "name",
    "message": "Name is required"
  }
}
```

### Server Errors (5xx)

```json
{
  "error": "Internal server error",
  "code": "INTERNAL_ERROR",
  "request_id": "req-uuid"
}
```

## WebSocket Support

WebSocket support is planned for real-time updates in a future release.

## API Clients

### JavaScript/TypeScript

```typescript
const AMP_BASE_URL = 'http://localhost:8105';

async function createSymbol(data) {
  const response = await fetch(`${AMP_BASE_URL}/v1/objects`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data)
  });
  return response.json();
}
```

### Python

```python
import requests

AMP_BASE_URL = 'http://localhost:8105'

def create_symbol(data):
    response = requests.post(
        f'{AMP_BASE_URL}/v1/objects',
        json=data
    )
    return response.json()
```

### Rust

```rust
use reqwest::Client;

const AMP_BASE_URL: &str = "http://localhost:8105";

async fn create_symbol(data: serde_json::Value) -> Result<serde_json::Value> {
    let client = Client::new();
    let response = client
        .post(format!("{}/v1/objects", AMP_BASE_URL))
        .json(&data)
        .send()
        .await?;
    response.json().await
}
```
