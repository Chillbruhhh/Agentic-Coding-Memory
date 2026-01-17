#!/bin/bash

# AMP Hybrid Retrieval Validation Script
# This script validates the hybrid retrieval implementation

echo "=== AMP Hybrid Retrieval Validation ==="
echo ""

echo "1. Checking file structure..."
if [ -f "amp/server/src/services/hybrid.rs" ]; then
    echo "✅ Hybrid service created"
else
    echo "❌ Hybrid service missing"
fi

if [ -f "amp/scripts/test-hybrid-retrieval.ps1" ]; then
    echo "✅ Test script created"
else
    echo "❌ Test script missing"
fi

if [ -f "amp/examples/hybrid_query_examples.surql" ]; then
    echo "✅ Documentation examples created"
else
    echo "❌ Documentation examples missing"
fi

echo ""
echo "2. Checking code integration..."
if grep -q "pub mod hybrid;" amp/server/src/services/mod.rs; then
    echo "✅ Hybrid module exported"
else
    echo "❌ Hybrid module not exported"
fi

if grep -q "hybrid_service:" amp/server/src/main.rs; then
    echo "✅ Hybrid service integrated in AppState"
else
    echo "❌ Hybrid service not integrated"
fi

if grep -q "hybrid.*Option<bool>" amp/server/src/handlers/query.rs; then
    echo "✅ Hybrid field added to QueryRequest"
else
    echo "❌ Hybrid field missing from QueryRequest"
fi

echo ""
echo "3. Manual validation steps:"
echo "   - Start server: cd amp/server && cargo run"
echo "   - Run tests: cd amp && powershell -ExecutionPolicy Bypass -File scripts/test-hybrid-retrieval.ps1"
echo "   - Test hybrid query: curl -X POST http://localhost:8105/v1/query -H 'Content-Type: application/json' -d '{\"text\": \"test\", \"hybrid\": true}'"
echo ""
echo "=== Validation Complete ==="
