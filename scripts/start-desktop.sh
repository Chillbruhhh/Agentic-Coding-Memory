#!/bin/bash

echo "🚀 Starting AMP Desktop Application..."

# Navigate to UI directory
cd "$(dirname "$0")/../amp/ui"

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
fi

# Start the desktop app
echo "🖥️  Launching desktop app..."
npm run tauri:dev
