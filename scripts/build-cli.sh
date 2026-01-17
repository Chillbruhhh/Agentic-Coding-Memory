#!/bin/bash
# Build AMP CLI for distribution

set -e

echo "🔨 Building AMP CLI for release..."

cd amp/cli

# Build optimized release binary
cargo build --release

# Copy binary to project root for easy access
cp target/release/amp ../../amp-cli

echo "✅ AMP CLI built successfully!"
echo "📦 Binary location: ./amp-cli"
echo "🚀 Run: ./amp-cli --help"

# Optional: Create tarball for distribution
if [ "$1" = "--package" ]; then
    cd ../..
    tar -czf amp-cli-$(uname -s)-$(uname -m).tar.gz amp-cli README.md
    echo "📦 Package created: amp-cli-$(uname -s)-$(uname -m).tar.gz"
fi
