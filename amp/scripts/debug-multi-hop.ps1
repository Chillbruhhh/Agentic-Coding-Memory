Write-Host "=== Debug Multi-Hop Implementation ===" -ForegroundColor Cyan

$baseUrl = "http://localhost:8105"
$funcA = [guid]::NewGuid().ToString()

Write-Host "1. Testing basic query (no algorithm)..." -ForegroundColor Yellow

$basicBody = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 2
    }
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $basicBody -ContentType "application/json"
    Write-Host "  Basic query works - found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "  Basic query failed: $_" -ForegroundColor Red
}

Write-Host "2. Testing with collect algorithm..." -ForegroundColor Yellow

$collectBody = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 2
        algorithm = "collect"
    }
} | ConvertTo-Json -Depth 3

Write-Host "Query JSON: $collectBody" -ForegroundColor Gray

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $collectBody -ContentType "application/json"
    Write-Host "  Collect query works - found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "  Collect query failed: $_" -ForegroundColor Red
    Write-Host "  Error details: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "=== Debug Complete ===" -ForegroundColor Cyan
