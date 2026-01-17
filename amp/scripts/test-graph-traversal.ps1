Write-Host "=== AMP Graph Traversal Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "1. Creating function chain..." -ForegroundColor Yellow

$funcA = [guid]::NewGuid().ToString()
$funcB = [guid]::NewGuid().ToString()
$funcC = [guid]::NewGuid().ToString()

$functions = @(
    @{id = $funcA; name = "function_a"},
    @{id = $funcB; name = "function_b"},
    @{id = $funcC; name = "function_c"}
)

foreach ($func in $functions) {
    $obj = @"
{
    "id": "$($func.id)",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "graph_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Test"},
    "links": [],
    "embedding": null,
    "name": "$($func.name)",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn $($func.name)()",
    "documentation": "Test function"
}
"@
    try {
        Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $obj -ContentType "application/json" | Out-Null
    } catch {
        Write-Host "Failed to create $($func.name): $_" -ForegroundColor Red
    }
}

# Create relationships: A -> B -> C
$rel1 = @{type = "calls"; source_id = $funcA; target_id = $funcB} | ConvertTo-Json
$rel2 = @{type = "calls"; source_id = $funcB; target_id = $funcC} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $rel1 -ContentType "application/json" | Out-Null
    Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $rel2 -ContentType "application/json" | Out-Null
    Write-Host "Created chain: function_a -> function_b -> function_c" -ForegroundColor Green
} catch {
    Write-Host "Failed to create relationships: $_" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "2. Testing graph traversal from function_a (outbound)..." -ForegroundColor Yellow

$query = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        edge_types = @("calls")
        max_depth = 3
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $query -ContentType "application/json"
    Write-Host "Found $($response.results.Count) nodes in traversal" -ForegroundColor Green
    foreach ($result in $response.results) {
        Write-Host "  - $($result.name)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Traversal failed: $_" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
