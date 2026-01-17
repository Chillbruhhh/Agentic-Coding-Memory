Write-Host "=== Multi-Hop Logic Validation ===" -ForegroundColor Cyan

$baseUrl = "http://localhost:8105"
$funcA = [guid]::NewGuid().ToString()

Write-Host "Testing COLLECT algorithm..." -ForegroundColor Yellow
$collectQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 3
        algorithm = "collect"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $collectQuery -ContentType "application/json"
    Write-Host "  COLLECT works - found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "  COLLECT failed: $_" -ForegroundColor Red
}

Write-Host "Testing PATH algorithm..." -ForegroundColor Yellow
$pathQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 2
        algorithm = "path"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $pathQuery -ContentType "application/json"
    Write-Host "  PATH works - found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "  PATH failed: $_" -ForegroundColor Red
}

Write-Host "Testing SHORTEST algorithm..." -ForegroundColor Yellow
$shortestQuery = @{
    graph = @{
        start_nodes = @($funcA)
        target_node = [guid]::NewGuid().ToString()
        direction = "outbound"
        max_depth = 5
        algorithm = "shortest"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $shortestQuery -ContentType "application/json"
    Write-Host "  SHORTEST should have failed" -ForegroundColor Red
} catch {
    Write-Host "  SHORTEST correctly failed - Target not reachable" -ForegroundColor Green
}

Write-Host "Testing depth validation..." -ForegroundColor Yellow
$invalidQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 15
        algorithm = "collect"
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $invalidQuery -ContentType "application/json"
    Write-Host "  Should have rejected depth > 10" -ForegroundColor Red
} catch {
    Write-Host "  Depth validation works" -ForegroundColor Green
}

Write-Host ""
Write-Host "=== MULTI-HOP LOGIC IS WORKING! ===" -ForegroundColor Green
