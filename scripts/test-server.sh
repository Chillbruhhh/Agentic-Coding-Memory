#!/bin/bash

echo "Testing Analytics Server Compilation"
echo "==================================="

cd /mnt/c/Users/Joshc/source/repos/ACM/amp/server

echo "Attempting to compile server..."
if cargo check 2>&1; then
    echo "✅ Server compiles successfully!"
    
    echo "Attempting to start server for 10 seconds..."
    timeout 10s cargo run &
    SERVER_PID=$!
    sleep 5
    
    echo "Testing health endpoint..."
    if curl -s http://localhost:8105/health | jq . 2>/dev/null; then
        echo "✅ Health endpoint working!"
        
        echo "Testing analytics endpoint..."
        if curl -s http://localhost:8105/v1/analytics | jq . 2>/dev/null; then
            echo "✅ Analytics endpoint working!"
        else
            echo "❌ Analytics endpoint failed"
        fi
    else
        echo "❌ Health endpoint failed"
    fi
    
    kill $SERVER_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
else
    echo "❌ Server compilation failed"
fi
