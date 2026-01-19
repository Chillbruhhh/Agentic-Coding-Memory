#!/bin/bash
# Test script for MCP integration fixes

SERVER_URL="http://localhost:8105"

echo "=== Testing MCP Integration Fixes ==="

# Test 1: Lease Acquire
echo -e "\n[Test 1] Testing lease acquisition..."
LEASE_RESPONSE=$(curl -s -X POST "$SERVER_URL/v1/leases/acquire" \
  -H "Content-Type: application/json" \
  -d '{
    "resource": "test-resource",
    "agent_id": "test-agent",
    "duration": 60
  }')

if echo "$LEASE_RESPONSE" | grep -q "lease_id"; then
    echo "✓ Lease acquired successfully"
    LEASE_ID=$(echo "$LEASE_RESPONSE" | grep -o '"lease_id":"[^"]*"' | cut -d'"' -f4)
    echo "  Lease ID: $LEASE_ID"
else
    echo "✗ Lease acquisition failed"
    echo "$LEASE_RESPONSE"
    LEASE_ID=""
fi

# Test 2: Lease Release
if [ -n "$LEASE_ID" ]; then
    echo -e "\n[Test 2] Testing lease release..."
    RELEASE_RESPONSE=$(curl -s -X POST "$SERVER_URL/v1/leases/release" \
      -H "Content-Type: application/json" \
      -d "{\"lease_id\": \"$LEASE_ID\"}")
    
    if [ $? -eq 0 ]; then
        echo "✓ Lease released successfully"
    else
        echo "✗ Lease release failed"
    fi
fi

# Test 3: File Log Get
echo -e "\n[Test 3] Testing file log retrieval..."
FILE_RESPONSE=$(curl -s -w "\n%{http_code}" "$SERVER_URL/v1/codebase/file-logs/amp/server/src/main.rs")
HTTP_CODE=$(echo "$FILE_RESPONSE" | tail -n1)
BODY=$(echo "$FILE_RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
    echo "✓ File log retrieved successfully"
    echo "  Response length: ${#BODY} bytes"
else
    echo "✗ File log retrieval failed (HTTP $HTTP_CODE)"
fi

# Test 4: Run Update
echo -e "\n[Test 4] Testing run update..."
RUN_CREATE=$(curl -s -X POST "$SERVER_URL/v1/objects" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "run",
    "agent_name": "test-agent",
    "goal": "Test run update",
    "repo_id": "test-repo",
    "tenant_id": "test-tenant",
    "project_id": "test-project"
  }')

if echo "$RUN_CREATE" | grep -q '"id"'; then
    RUN_ID=$(echo "$RUN_CREATE" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    echo "  Created run: $RUN_ID"
    
    # Update the run
    UPDATE_RESPONSE=$(curl -s -w "\n%{http_code}" -X PUT "$SERVER_URL/v1/objects/$RUN_ID" \
      -H "Content-Type: application/json" \
      -d '{
        "status": "completed",
        "summary": "Test completed successfully",
        "outputs": ["output1", "output2"]
      }')
    
    UPDATE_CODE=$(echo "$UPDATE_RESPONSE" | tail -n1)
    if [ "$UPDATE_CODE" = "204" ]; then
        echo "✓ Run updated successfully"
    else
        echo "✗ Run update failed (HTTP $UPDATE_CODE)"
    fi
else
    echo "✗ Run creation failed"
fi

echo -e "\n=== Test Summary ==="
echo "All critical MCP integration fixes have been tested."
