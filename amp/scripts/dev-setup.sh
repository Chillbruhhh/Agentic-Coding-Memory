#!/bin/bash

# AMP Development Scripts

echo "=== AMP Development Environment ==="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Install from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust/Cargo found"

# Build the server
echo "🔨 Building AMP server..."
cd server && cargo build

if [ $? -eq 0 ]; then
    echo "✅ Server build successful"
else
    echo "❌ Server build failed"
    exit 1
fi

# Run tests
echo "🧪 Running tests..."
cargo test

echo "🚀 Ready to start AMP server with: cargo run"
