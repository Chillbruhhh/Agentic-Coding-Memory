#!/bin/bash

echo "Testing AMP Analytics Implementation"
echo "===================================="

# Test 1: Check if server compiles
echo "1. Checking server compilation..."
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server
if command -v cargo &> /dev/null; then
    cargo check
    if [ $? -eq 0 ]; then
        echo "✅ Server compiles successfully"
    else
        echo "❌ Server compilation failed"
        exit 1
    fi
else
    echo "⚠️  Cargo not available, skipping compilation check"
fi

# Test 2: Check if analytics endpoint would be accessible
echo "2. Checking if server can start..."
if command -v cargo &> /dev/null; then
    timeout 10s cargo run &
    SERVER_PID=$!
    sleep 5
    
    # Test health endpoint
    if curl -s http://localhost:8105/health > /dev/null 2>&1; then
        echo "✅ Server started successfully"
        
        # Test analytics endpoint
        if curl -s http://localhost:8105/v1/analytics > /dev/null 2>&1; then
            echo "✅ Analytics endpoint accessible"
        else
            echo "❌ Analytics endpoint not accessible"
        fi
    else
        echo "❌ Server failed to start or health check failed"
    fi
    
    # Clean up
    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
else
    echo "⚠️  Cargo not available, skipping server start test"
fi

echo "3. Checking UI TypeScript compilation..."
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/ui
if command -v npm &> /dev/null; then
    # Only check our specific analytics files
    echo "Checking analytics hook and component..."
    if npx tsc --noEmit src/hooks/useAnalytics.ts src/components/Analytics.tsx 2>/dev/null; then
        echo "✅ Analytics TypeScript files compile successfully"
    else
        echo "❌ Analytics TypeScript files have compilation errors"
    fi
else
    echo "⚠️  npm not available, skipping TypeScript check"
fi

echo ""
echo "Analytics Implementation Test Complete"
echo "====================================="
