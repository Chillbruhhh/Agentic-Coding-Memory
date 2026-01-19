#!/bin/bash

# Build AMP MCP Server
# Usage: ./build-mcp-server.sh

set -e

echo "Building AMP MCP Server..."

cd "$(dirname "$0")/amp/mcp-server"

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Build release binary
echo "Building release binary..."
cargo build --release

echo "✓ Build complete!"
echo "Binary location: $(pwd)/target/release/amp-mcp-server"
echo ""
echo "To run:"
echo "  cd amp/mcp-server"
echo "  ./target/release/amp-mcp-server"
echo ""
echo "Or install globally:"
echo "  cargo install --path ."
