# Direct database clear for AMP (Windows)

Write-Host "🗑️  Clearing AMP database directly..." -ForegroundColor Yellow

# Stop the AMP server first
Write-Host "⏹️  Stop the AMP server first with Ctrl+C" -ForegroundColor Red

# Delete the database file/directory
$dbPath = "amp\server\amp.db"

if (Test-Path $dbPath) {
    Remove-Item -Recurse -Force $dbPath
    Write-Host "✅ Deleted database: $dbPath" -ForegroundColor Green
} else {
    Write-Host "ℹ️  Database not found at: $dbPath" -ForegroundColor Cyan
}

Write-Host "🚀 Database cleared! Restart the AMP server and re-index." -ForegroundColor Green
Write-Host "   cd amp\server && cargo run" -ForegroundColor Cyan
Write-Host "   cd amp\cli && cargo run -- index" -ForegroundColor Cyan
