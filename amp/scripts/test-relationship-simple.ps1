Write-Host "=== Testing Relationship Creation ===" -ForegroundColor Cyan

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# Create two simple functions
$funcA = [guid]::NewGuid().ToString()
$funcB = [guid]::NewGuid().ToString()

Write-Host "Creating function A: $funcA" -ForegroundColor Yellow

$objA = @{
    id = $funcA
    type = "symbol"
    tenant_id = "test"
    project_id = "rel_test"
    created_at = $now
    updated_at = $now
    provenance = @{agent = "test"; summary = "Relationship test"}
    links = @()
    embedding = $null
    name = "function_a"
    kind = "function"
    path = "src/lib.rs"
    language = "rust"
    signature = "fn function_a()"
} | ConvertTo-Json -Depth 3

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $objA -ContentType "application/json" | Out-Null
    Write-Host "  Created function A" -ForegroundColor Green
} catch {
    Write-Host "  Failed to create function A: $_" -ForegroundColor Red
}

Write-Host "Creating function B: $funcB" -ForegroundColor Yellow

$objB = @{
    id = $funcB
    type = "symbol"
    tenant_id = "test"
    project_id = "rel_test"
    created_at = $now
    updated_at = $now
    provenance = @{agent = "test"; summary = "Relationship test"}
    links = @()
    embedding = $null
    name = "function_b"
    kind = "function"
    path = "src/lib.rs"
    language = "rust"
    signature = "fn function_b()"
} | ConvertTo-Json -Depth 3

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $objB -ContentType "application/json" | Out-Null
    Write-Host "  Created function B" -ForegroundColor Green
} catch {
    Write-Host "  Failed to create function B: $_" -ForegroundColor Red
}

Write-Host "Creating relationship A calls B..." -ForegroundColor Yellow

$relObj = @{
    type = "calls"
    source_id = $funcA
    target_id = $funcB
} | ConvertTo-Json

Write-Host "Relationship JSON: $relObj" -ForegroundColor Gray

try {
    $result = Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $relObj -ContentType "application/json"
    Write-Host "  Created relationship successfully" -ForegroundColor Green
} catch {
    Write-Host "  Failed to create relationship: $_" -ForegroundColor Red
}

Write-Host "=== Relationship Test Complete ===" -ForegroundColor Cyan
