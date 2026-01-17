#!/bin/bash
# Direct database clear for AMP

echo "🗑️  Clearing AMP database directly..."

# Stop the AMP server first (if running)
echo "⏹️  Stop the AMP server first with Ctrl+C"

# Delete the database file
if [ -f "amp/server/amp.db" ]; then
    rm -rf amp/server/amp.db
    echo "✅ Deleted database file: amp/server/amp.db"
fi

if [ -d "amp/server/amp.db" ]; then
    rm -rf amp/server/amp.db
    echo "✅ Deleted database directory: amp/server/amp.db"
fi

echo "🚀 Database cleared! Restart the AMP server and re-index."
echo "   cd amp/server && cargo run"
echo "   cd amp/cli && cargo run -- index"
