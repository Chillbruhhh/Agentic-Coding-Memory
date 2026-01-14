#!/bin/bash
# AMP CRUD Test Script

BASE_URL="http://localhost:8105"

echo "=== AMP CRUD Operations Test ==="
echo ""

# Test 1: Health Check
echo "1. Testing health endpoint..."
curl -s "$BASE_URL/health" | jq .
echo ""

# Test 2: Create a Symbol object
echo "2. Creating a Symbol object..."
SYMBOL_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/v1/objects" \
  -H "Content-Type: application/json" \
  -d "{
    \"Symbol\": {
      \"base\": {
        \"id\": \"$SYMBOL_ID\",
        \"type\": \"symbol\",
        \"tenant_id\": \"test\",
        \"project_id\": \"amp_demo\",
        \"created_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"updated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"provenance\": {
          \"agent\": \"test_script\",
          \"summary\": \"Testing symbol creation\"
        },
        \"links\": [],
        \"embedding\": null
      },
      \"name\": \"main\",
      \"kind\": \"function\",
      \"path\": \"src/main.rs\",
      \"language\": \"rust\",
      \"content_hash\": null,
      \"signature\": \"fn main()\",
      \"documentation\": \"Entry point for AMP server\"
    }
  }")

echo "$CREATE_RESPONSE" | jq .
CREATED_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
echo "Created object with ID: $CREATED_ID"
echo ""

# Test 3: Retrieve the object
echo "3. Retrieving the created object..."
curl -s "$BASE_URL/v1/objects/$CREATED_ID" | jq .
echo ""

# Test 4: Create a Decision object
echo "4. Creating a Decision object..."
DECISION_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
curl -s -X POST "$BASE_URL/v1/objects" \
  -H "Content-Type: application/json" \
  -d "{
    \"Decision\": {
      \"base\": {
        \"id\": \"$DECISION_ID\",
        \"type\": \"decision\",
        \"tenant_id\": \"test\",
        \"project_id\": \"amp_demo\",
        \"created_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"updated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
        \"provenance\": {
          \"agent\": \"test_script\",
          \"summary\": \"Testing decision creation\"
        },
        \"links\": [],
        \"embedding\": null
      },
      \"title\": \"Use SurrealDB for storage\",
      \"problem\": \"Need embedded database with vector and graph support\",
      \"options\": null,
      \"rationale\": \"SurrealDB provides all required features in single package\",
      \"outcome\": \"Implemented with embedded mode\",
      \"status\": \"accepted\"
    }
  }" | jq .
echo ""

# Test 5: Batch create
echo "5. Testing batch creation..."
curl -s -X POST "$BASE_URL/v1/objects/batch" \
  -H "Content-Type: application/json" \
  -d "[
    {
      \"Symbol\": {
        \"base\": {
          \"id\": \"$(uuidgen | tr '[:upper:]' '[:lower:]')\",
          \"type\": \"symbol\",
          \"tenant_id\": \"test\",
          \"project_id\": \"amp_demo\",
          \"created_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
          \"updated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
          \"provenance\": {
            \"agent\": \"test_script\",
            \"summary\": \"Batch test 1\"
          },
          \"links\": [],
          \"embedding\": null
        },
        \"name\": \"config\",
        \"kind\": \"module\",
        \"path\": \"src/config.rs\",
        \"language\": \"rust\",
        \"content_hash\": null,
        \"signature\": null,
        \"documentation\": null
      }
    },
    {
      \"Symbol\": {
        \"base\": {
          \"id\": \"$(uuidgen | tr '[:upper:]' '[:lower:]')\",
          \"type\": \"symbol\",
          \"tenant_id\": \"test\",
          \"project_id\": \"amp_demo\",
          \"created_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
          \"updated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",
          \"provenance\": {
            \"agent\": \"test_script\",
            \"summary\": \"Batch test 2\"
          },
          \"links\": [],
          \"embedding\": null
        },
        \"name\": \"database\",
        \"kind\": \"module\",
        \"path\": \"src/database.rs\",
        \"language\": \"rust\",
        \"content_hash\": null,
        \"signature\": null,
        \"documentation\": null
      }
    }
  ]" | jq .
echo ""

echo "=== Test Complete ==="
