#!/bin/bash
# Test AMP CLI npm package locally

set -e

echo "🧪 Testing AMP CLI npm package..."

# Step 1: Create the package
echo ""
echo "📦 Step 1: Creating npm package..."
npm pack

PACKAGE_FILE=$(ls amp-protocol-cli-*.tgz | head -n 1)

if [ -z "$PACKAGE_FILE" ]; then
    echo "❌ Failed to create package"
    exit 1
fi

echo "✅ Package created: $PACKAGE_FILE"

# Step 2: Install globally
echo ""
echo "📥 Step 2: Installing package globally..."
npm install -g "$PACKAGE_FILE"

# Step 3: Test the command
echo ""
echo "🧪 Step 3: Testing amp command..."

echo ""
echo "Testing: amp --help"
if amp --help; then
    echo "✅ amp --help works!"
else
    echo "❌ amp --help failed"
fi

echo ""
echo "Testing: amp status"
amp status || true

# Step 4: Cleanup
echo ""
echo "🧹 Step 4: Cleanup..."
read -p "Uninstall the package? (y/n) " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    npm uninstall -g @amp-protocol/cli
    rm "$PACKAGE_FILE"
    echo "✅ Cleanup complete"
else
    echo "⚠️  Package still installed. Uninstall with: npm uninstall -g @amp-protocol/cli"
    echo "⚠️  Package file: $PACKAGE_FILE"
fi

echo ""
echo "✅ Test complete!"
