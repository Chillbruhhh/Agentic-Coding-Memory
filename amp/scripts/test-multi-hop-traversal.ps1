Write-Host "=== AMP Multi-Hop Graph Traversal Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "1. Creating multi-level function chain..." -ForegroundColor Yellow

# Create a chain: A -> B -> C -> D
$funcA = [guid]::NewGuid().ToString()
$funcB = [guid]::NewGuid().ToString()
$funcC = [guid]::NewGuid().ToString()
$funcD = [guid]::NewGuid().ToString()

$functions = @(
    @{id = $funcA; name = "function_a"},
    @{id = $funcB; name = "function_b"},
    @{id = $funcC; name = "function_c"},
    @{id = $funcD; name = "function_d"}
)

# Create function objects
foreach ($func in $functions) {
    $obj = @"
{
    "id": "$($func.id)",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "multi_hop_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Multi-hop test"},
    "links": [],
    "embedding": null,
    "name": "$($func.name)",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn $($func.name)()",
    "documentation": "Test function for multi-hop traversal"
}
"@
    try {
        Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $obj -ContentType "application/json" | Out-Null
        Write-Host "  Created $($func.name)" -ForegroundColor Green
    } catch {
        Write-Host "  Failed to create $($func.name): $_" -ForegroundColor Red
    }
}

# Create relationships: A -> B -> C -> D
$relationships = @(
    @{source = $funcA; target = $funcB; type = "calls"},
    @{source = $funcB; target = $funcC; type = "calls"},
    @{source = $funcC; target = $funcD; type = "calls"}
)

foreach ($rel in $relationships) {
    $relObj = @{
        type = $rel.type
        source_id = $rel.source
        target_id = $rel.target
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $relObj -ContentType "application/json" | Out-Null
    } catch {
        Write-Host "Failed to create relationship: $_" -ForegroundColor Yellow
    }
}

Write-Host "Created chain: function_a -> function_b -> function_c -> function_d" -ForegroundColor Green
Write-Host ""

Write-Host "2. Testing COLLECT algorithm (depth 3)..." -ForegroundColor Yellow

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
    Write-Host "  Found $($response.results.Count) unique nodes in collect traversal" -ForegroundColor Green
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "  Collect traversal failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "3. Testing PATH algorithm (depth 2)..." -ForegroundColor Yellow

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
    Write-Host "  Found $($response.results.Count) paths in path traversal" -ForegroundColor Green
    foreach ($result in $response.results) {
        if ($result.path) {
            $pathNames = $result.path | ForEach-Object { if ($_.name) { $_.name } else { "unknown" } }
            Write-Host "    Path: $($pathNames -join ' -> ')" -ForegroundColor Gray
        } elseif ($result.object.name) {
            Write-Host "    Node: $($result.object.name)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "  Path traversal failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "4. Testing SHORTEST algorithm..." -ForegroundColor Yellow

$shortestQuery = @{
    graph = @{
        start_nodes = @($funcA)
        target_node = $funcD
        direction = "outbound"
        max_depth = 5
        algorithm = "shortest"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $shortestQuery -ContentType "application/json"
    Write-Host "  Found shortest path with $($response.results.Count) steps" -ForegroundColor Green
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "  Shortest path traversal failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "5. Testing depth validation (should fail with depth > 10)..." -ForegroundColor Yellow

$invalidDepthQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 15
        algorithm = "collect"
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $invalidDepthQuery -ContentType "application/json"
    Write-Host "  ERROR: Depth validation failed - should have rejected depth > 10" -ForegroundColor Red
} catch {
    if ($_.Exception.Response.StatusCode -eq 400) {
        Write-Host "  ✓ Depth validation working - correctly rejected depth > 10" -ForegroundColor Green
    } else {
        Write-Host "  Unexpected error: $_" -ForegroundColor Yellow
    }
}
Write-Host ""

Write-Host "6. Testing backward compatibility (no algorithm specified)..." -ForegroundColor Yellow

$compatQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 2
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $compatQuery -ContentType "application/json"
    Write-Host "  ✓ Backward compatibility working - found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "  Backward compatibility failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== Multi-Hop Test Complete ===" -ForegroundColor Cyan
