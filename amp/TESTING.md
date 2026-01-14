# AMP Testing Guide

## Quick Test (No Rust Installation Required)

If you don't have Rust installed, you can still test the API design and schemas:

### 1. Validate OpenAPI Spec
```bash
# Install swagger-codegen or openapi-generator
npm install -g @openapitools/openapi-generator-cli

# Validate the spec
cd amp
openapi-generator validate -i spec/openapi.yaml
```

### 2. Test JSON Schemas
```bash
# Install ajv-cli for JSON schema validation
npm install -g ajv-cli

# Test a sample symbol object
echo '{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "type": "symbol",
  "tenant_id": "test",
  "project_id": "amp",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z",
  "provenance": {
    "agent": "test",
    "summary": "test symbol"
  },
  "name": "main",
  "kind": "function",
  "path": "src/main.rs",
  "language": "rust"
}' | ajv validate -s spec/schemas/symbol.json
```

## Full Test (With Rust)

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. Build and Run Server
```bash
cd amp/server
cargo build
cargo run
```

The server will:
- Start on `http://localhost:8080`
- Create an embedded SurrealDB instance (in memory by default)
- Automatically run the schema initialization from `spec/schema.surql`
- Be ready to accept API requests

### 3. Test API Endpoints

**Health Check:**
```bash
curl http://localhost:8080/health
```

**Create a Symbol:**
```bash
curl -X POST http://localhost:8080/v1/objects \
  -H "Content-Type: application/json" \
  -d '{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "amp_demo",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "provenance": {
      "agent": "curl_test",
      "summary": "Testing symbol creation"
    },
    "name": "main",
    "kind": "function",
    "path": "src/main.rs",
    "language": "rust"
  }'
```

**Query Objects:**
```bash
curl -X POST http://localhost:8080/v1/query \
  -H "Content-Type: application/json" \
  -d '{
    "text": "rust functions",
    "limit": 10
  }'
```

### 4. Run the Demo Script
```bash
cd amp
chmod +x scripts/demo.sh
./scripts/demo.sh
```

## Database Details

### SurrealDB Configuration
- **Mode**: Embedded (no separate server needed)
- **Storage**: Memory by default (data lost on restart)
- **Schema**: Auto-initialized from `spec/schema.surql`

### Persistent Storage
To use file-based storage instead of memory:
```bash
export DATABASE_URL="file://amp.db"
cargo run
```

### Manual Schema Testing
If you want to test SurrealDB queries manually:
```bash
# Install SurrealDB CLI
curl -sSf https://install.surrealdb.com | sh

# Connect to the embedded database (while server is running)
surreal sql --conn http://localhost:8080 --user root --pass root --ns amp --db main

# Run test queries
SELECT * FROM objects;
SELECT * FROM symbols;
```

## Expected Behavior

### What Works ✅
- Server starts and binds to port 8080
- Health endpoint returns JSON response
- Schema initialization runs without errors
- API endpoints accept requests

### What's Not Implemented Yet ⚠️
- Object creation returns placeholder data (doesn't actually store)
- Object retrieval returns 501 Not Implemented
- Query endpoint returns empty results
- No actual database operations yet

This is expected for the hackathon prototype - the foundation is solid and ready for implementation!

## Troubleshooting

### Port Already in Use
```bash
# Kill any process using port 8080
lsof -ti:8080 | xargs kill -9
```

### Compilation Errors
```bash
# Update Rust toolchain
rustup update
# Clean and rebuild
cargo clean && cargo build
```

### Schema Errors
Check the server logs - schema warnings are expected and don't prevent startup.
