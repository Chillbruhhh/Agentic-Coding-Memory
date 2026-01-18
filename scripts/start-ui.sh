#!/bin/bash

# AMP UI Test Script
echo "🚀 Starting AMP Console UI Test"

# Check if AMP server is running
echo "📡 Checking AMP server status..."
if curl -s http://localhost:8105/v1/objects > /dev/null 2>&1; then
    echo "✅ AMP server is running on port 8105"
else
    echo "⚠️  AMP server not detected - UI will use mock data"
fi

# Start the UI development server
echo "🎨 Starting UI development server..."
cd /mnt/c/Users/Joshc/source/repos/ACM/amp/ui

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Start development server
echo "🌐 Starting Vite dev server on http://localhost:8109"
npm run dev
