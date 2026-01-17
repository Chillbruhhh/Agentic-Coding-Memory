# Test External SurrealDB Connection
Write-Host "=== Testing External SurrealDB Connection ===" -ForegroundColor Cyan
Write-Host ""

# Check if SurrealDB is running
Write-Host "1. Checking if SurrealDB is running on port 7505..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:7505/health" -Method Get -TimeoutSec 2 -ErrorAction Stop
    Write-Host "   ✓ SurrealDB is running" -ForegroundColor Green
} catch {
    Write-Host "   ✗ SurrealDB is not responding on port 7505" -ForegroundColor Red
    Write-Host "   Please start SurrealDB with: surreal start --bind 0.0.0.0:7505 --user root --pass root" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "2. Checking AMP server configuration..." -ForegroundColor Yellow
$envFile = "../server/.env"
if (Test-Path $envFile) {
    $dbUrl = Get-Content $envFile | Where-Object { $_ -match "^DATABASE_URL=" } | Select-Object -First 1
    Write-Host "   Current DATABASE_URL: $dbUrl" -ForegroundColor Gray
    
    if ($dbUrl -match "ws://localhost:7505") {
        Write-Host "   ✓ Configured for WebSocket connection" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Not configured for external DB" -ForegroundColor Red
        Write-Host "   Update .env to: DATABASE_URL=ws://localhost:7505" -ForegroundColor Yellow
    }
} else {
    Write-Host "   ✗ .env file not found" -ForegroundColor Red
}

Write-Host ""
Write-Host "3. Testing AMP server connection..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8105/health" -Method Get -TimeoutSec 2 -ErrorAction Stop
    Write-Host "   ✓ AMP server is running" -ForegroundColor Green
    Write-Host "   Database: $($response.database)" -ForegroundColor Gray
} catch {
    Write-Host "   ✗ AMP server is not responding" -ForegroundColor Red
    Write-Host "   Please restart the server to apply new configuration" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
