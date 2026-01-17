#!/bin/bash
# AMP CLI Installation Script

set -e

echo "🚀 Installing AMP CLI..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build and install the CLI
cd amp/cli
cargo install --path . --force

echo "✅ AMP CLI installed successfully!"
echo "📋 Usage: amp --help"
echo "🎯 Start a session: amp start 'your-agent-command'"
echo "📊 Check status: amp status"
echo "🖥️  Launch TUI: amp tui"
