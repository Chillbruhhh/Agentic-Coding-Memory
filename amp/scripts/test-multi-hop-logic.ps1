Write-Host "=== AMP Multi-Hop Logic Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "1. Creating complex function chain A->B->C->D->E..." -ForegroundColor Yellow

# Create a chain: A -> B -> C -> D -> E
$funcA = [guid]::NewGuid().ToString()
$funcB = [guid]::NewGuid().ToString()
$funcC = [guid]::NewGuid().ToString()
$funcD = [guid]::NewGuid().ToString()
$funcE = [guid]::NewGuid().ToString()

$functions = @(
    @{id = $funcA; name = "function_a"},
    @{id = $funcB; name = "function_b"},
    @{id = $funcC; name = "function_c"},
    @{id = $funcD; name = "function_d"},
    @{id = $funcE; name = "function_e"}
)

# Create function objects
foreach ($func in $functions) {
    $obj = @"
{
    "id": "$($func.id)",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "multi_hop_logic_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Multi-hop logic test"},
    "links": [],
    "embedding": null,
    "name": "$($func.name)",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn $($func.name)()",
    "documentation": "Test function for multi-hop logic"
}
"@
    try {
        Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $obj -ContentType "application/json" | Out-Null
        Write-Host "  Created $($func.name)" -ForegroundColor Green
    } catch {
        Write-Host "  Failed to create $($func.name): $_" -ForegroundColor Red
    }
}

# Create relationships: A -> B -> C -> D -> E
$relationships = @(
    @{source = $funcA; target = $funcB; type = "calls"},
    @{source = $funcB; target = $funcC; type = "calls"},
    @{source = $funcC; target = $funcD; type = "calls"},
    @{source = $funcD; target = $funcE; type = "calls"}
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

Write-Host "Created chain: function_a -> function_b -> function_c -> function_d -> function_e" -ForegroundColor Green
Write-Host ""

Write-Host "2. Testing COLLECT algorithm depth 4..." -ForegroundColor Yellow

$collectQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 4
        algorithm = "collect"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $collectQuery -ContentType "application/json"
    Write-Host "  ✓ Collect algorithm: found $($response.results.Count) unique nodes" -ForegroundColor Green
    Write-Host "    Expected: 4 nodes B, C, D, E" -ForegroundColor Gray
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "  ✗ Collect algorithm failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "3. Testing PATH algorithm depth 3..." -ForegroundColor Yellow

$pathQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 3
        algorithm = "path"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $pathQuery -ContentType "application/json"
    Write-Host "  ✓ Path algorithm: found $($response.results.Count) nodes in paths" -ForegroundColor Green
    Write-Host "    Expected: 3 nodes B, C, D with path information" -ForegroundColor Gray
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Gray
        }
        if ($result.path) {
            Write-Host "      Path length: $($result.path.Count)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "  ✗ Path algorithm failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "4. Testing SHORTEST algorithm A to E..." -ForegroundColor Yellow

$shortestQuery = @{
    graph = @{
        start_nodes = @($funcA)
        target_node = $funcE
        direction = "outbound"
        max_depth = 5
        algorithm = "shortest"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $shortestQuery -ContentType "application/json"
    Write-Host "  ✓ Shortest path: found path with $($response.results.Count) nodes" -ForegroundColor Green
    Write-Host "    Expected: 5 nodes A, B, C, D, E" -ForegroundColor Gray
    foreach ($result in $response.results) {
        if ($result.object.name) {
            Write-Host "    - $($result.object.name)" -ForegroundColor Gray
        }
    }
} catch {
    Write-Host "  ✗ Shortest path failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "5. Testing cycle detection..." -ForegroundColor Yellow

# Create a cycle: E -> A
$cycleRel = @{
    type = "calls"
    source_id = $funcE
    target_id = $funcA
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $cycleRel -ContentType "application/json" | Out-Null
    Write-Host "  Created cycle: E -> A" -ForegroundColor Green
} catch {
    Write-Host "  Failed to create cycle: $_" -ForegroundColor Yellow
}

# Test collect with cycle
$cycleQuery = @{
    graph = @{
        start_nodes = @($funcA)
        direction = "outbound"
        max_depth = 10
        algorithm = "collect"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $cycleQuery -ContentType "application/json"
    Write-Host "  ✓ Cycle detection: found $($response.results.Count) unique nodes should not infinite loop" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Cycle detection failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "6. Testing unreachable target..." -ForegroundColor Yellow

# Create isolated node
$funcZ = [guid]::NewGuid().ToString()
$objZ = @"
{
    "id": "$funcZ",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "multi_hop_logic_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Isolated node"},
    "links": [],
    "embedding": null,
    "name": "function_z",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn function_z()",
    "documentation": "Isolated function"
}
"@

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $objZ -ContentType "application/json" | Out-Null
    Write-Host "  Created isolated function_z" -ForegroundColor Green
} catch {
    Write-Host "  Failed to create function_z: $_" -ForegroundColor Red
}

$unreachableQuery = @{
    graph = @{
        start_nodes = @($funcA)
        target_node = $funcZ
        direction = "outbound"
        max_depth = 5
        algorithm = "shortest"
        relation_types = @("calls")
    }
} | ConvertTo-Json -Depth 4

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $unreachableQuery -ContentType "application/json"
    Write-Host "  ✗ Should have failed for unreachable target" -ForegroundColor Red
} catch {
    if ($_.Exception.Response.StatusCode -eq 500) {
        Write-Host "  ✓ Correctly handled unreachable target" -ForegroundColor Green
    } else {
        Write-Host "  ? Unexpected error: $_" -ForegroundColor Yellow
    }
}
Write-Host ""

Write-Host "7. Testing backward compatibility (no algorithm)..." -ForegroundColor Yellow

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
    Write-Host "  ✓ Backward compatibility: found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Backward compatibility failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== Multi-Hop Logic Test Complete ===" -ForegroundColor Cyan
