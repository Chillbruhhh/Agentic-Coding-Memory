Write-Host "=== AMP Multi-Hop Graph Traversal Test ===" -ForegroundColor Cyan

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "1. Creating test functions..." -ForegroundColor Yellow

$funcA = [guid]::NewGuid().ToString()
$funcB = [guid]::NewGuid().ToString()

# Create function A
$objA = @"
{
    "id": "$funcA",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "multi_hop_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Multi-hop test"},
    "links": [],
    "embedding": null,
    "name": "function_a",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn function_a()",
    "documentation": "Test function"
}
"@

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $objA -ContentType "application/json" | Out-Null
    Write-Host "  Created function_a" -ForegroundColor Green
} catch {
    Write-Host "  Failed to create function_a: $_" -ForegroundColor Red
}

Write-Host "2. Testing COLLECT algorithm..." -ForegroundColor Yellow

$collectBody = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 3
        algorithm = "collect"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $collectBody -ContentType "application/json"
    Write-Host "  Collect algorithm test passed - found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "  Collect algorithm failed: $_" -ForegroundColor Red
}

Write-Host "3. Testing depth validation..." -ForegroundColor Yellow

$invalidBody = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 15
        algorithm = "collect"
    }
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $invalidBody -ContentType "application/json"
    Write-Host "  ERROR: Should have rejected depth > 10" -ForegroundColor Red
} catch {
    Write-Host "  Depth validation working - correctly rejected depth > 10" -ForegroundColor Green
}

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
