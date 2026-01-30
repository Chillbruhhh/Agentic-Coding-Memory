# Test nuclear delete endpoint

Write-Host "Testing Nuclear Delete Endpoint" -ForegroundColor Yellow
Write-Host "================================" -ForegroundColor Yellow
Write-Host ""

# Check server health
Write-Host "1. Checking server health..." -ForegroundColor Cyan
try {
    $health = Invoke-RestMethod -Uri "http://localhost:8105/health" -Method Get
    Write-Host "   Server is healthy: $($health.status)" -ForegroundColor Green
} catch {
    Write-Host "   ERROR: Server is not running!" -ForegroundColor Red
    Write-Host "   Please start the server first: cd amp/server && cargo run" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "2. Testing nuclear delete endpoint..." -ForegroundColor Cyan
Write-Host "   WARNING: This will delete ALL data!" -ForegroundColor Red
Write-Host ""

$confirm = Read-Host "   Type 'YES' to proceed with test"
if ($confirm -ne "YES") {
    Write-Host "   Test cancelled." -ForegroundColor Yellow
    exit 0
}

try {
    $result = Invoke-RestMethod -Uri "http://localhost:8105/v1/settings/nuclear-delete" -Method Post
    Write-Host "   SUCCESS: Nuclear delete completed" -ForegroundColor Green
    Write-Host "   Queries executed: $($result.queries_executed)" -ForegroundColor Green
    Write-Host "   Message: $($result.message)" -ForegroundColor Green
} catch {
    Write-Host "   ERROR: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "   Status: $($_.Exception.Response.StatusCode.value__)" -ForegroundColor Red
}

Write-Host ""
Write-Host "Test complete!" -ForegroundColor Yellow
