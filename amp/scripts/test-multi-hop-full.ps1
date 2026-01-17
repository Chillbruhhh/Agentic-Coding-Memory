Write-Host "=== AMP Multi-Hop Full Test ===" -ForegroundColor Cyan

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "1. Creating function chain A->B->C->D..." -ForegroundColor Yellow

# Create functions A, B, C, D
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

foreach ($func in $functions) {
    $obj = @"
{
    "id": "$($func.id)",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "multi_hop_full_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Multi-hop full test"},
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
        Write-Host "  Created $($func.name)" -ForegroundColor Green
    } catch {
        Write-Host "  Failed to create $($func.name): $_" -ForegroundColor Red
    }
}

Write-Host "2. Creating relationships A->B->C->D..." -ForegroundColor Yellow

# Create relationships: A calls B, B calls C, C calls D
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
        Write-Host "  Created relationship" -ForegroundColor Green
    } catch {
        Write-Host "  Failed to create relationship: $_" -ForegroundColor Yellow
    }
}

Write-Host "Created chain: function_a -> function_b -> function_c -> function_d" -ForegroundColor Green
Write-Host ""

Write-Host "3. Testing COLLECT algorithm depth 3..." -ForegroundColor Yellow

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
    Write-Host "  COLLECT: found $($response.results.Count) unique nodes" -ForegroundColor Green
    Write-Host "  Expected: 3 nodes B, C, D" -ForegroundColor Gray
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Cyan
        }
    }
} catch {
    Write-Host "  COLLECT failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "4. Testing PATH algorithm depth 2..." -ForegroundColor Yellow

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
    Write-Host "  PATH: found $($response.results.Count) nodes in paths" -ForegroundColor Green
    Write-Host "  Expected: nodes with path information" -ForegroundColor Gray
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Cyan
            if ($result.path) {
                Write-Host "      Path length: $($result.path.Count)" -ForegroundColor Gray
            }
        }
    }
} catch {
    Write-Host "  PATH failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "5. Testing SHORTEST algorithm A to D..." -ForegroundColor Yellow

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
    Write-Host "  SHORTEST: found path with $($response.results.Count) nodes" -ForegroundColor Green
    Write-Host "  Expected: 4 nodes A, B, C, D" -ForegroundColor Gray
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Cyan
        }
    }
} catch {
    Write-Host "  SHORTEST failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "6. Testing backward compatibility no algorithm..." -ForegroundColor Yellow

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
    Write-Host "  BACKWARD COMPAT: found $($response.results.Count) results" -ForegroundColor Green
    Write-Host "  Expected: single-hop results" -ForegroundColor Gray
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Cyan
        }
    }
} catch {
    Write-Host "  BACKWARD COMPAT failed: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Multi-Hop Full Test Complete ===" -ForegroundColor Cyan
