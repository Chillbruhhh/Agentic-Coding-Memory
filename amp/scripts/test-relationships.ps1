Write-Host "=== AMP Relationship Management Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "1. Creating test objects..." -ForegroundColor Yellow

$symbol1Id = [guid]::NewGuid().ToString()
$symbol2Id = [guid]::NewGuid().ToString()

$symbol1 = @"
{
    "id": "$symbol1Id",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "graph_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Test"},
    "links": [],
    "embedding": null,
    "name": "authenticate_user",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn authenticate_user()",
    "documentation": "Authenticates a user"
}
"@

$symbol2 = @"
{
    "id": "$symbol2Id",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "graph_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Test"},
    "links": [],
    "embedding": null,
    "name": "hash_password",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn hash_password()",
    "documentation": "Hashes a password"
}
"@

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol1 -ContentType "application/json" | Out-Null
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol2 -ContentType "application/json" | Out-Null
    Write-Host "Created 2 symbols" -ForegroundColor Green
    Write-Host "  - authenticate_user: $symbol1Id" -ForegroundColor Gray
    Write-Host "  - hash_password: $symbol2Id" -ForegroundColor Gray
} catch {
    Write-Host "Failed to create: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "2. Creating 'calls' relationship..." -ForegroundColor Yellow

$relationship = @{
    type = "calls"
    source_id = $symbol1Id
    target_id = $symbol2Id
    metadata = @{
        line_number = 42
    }
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/relationships" -Method Post -Body $relationship -ContentType "application/json"
    Write-Host "Relationship created" -ForegroundColor Green
} catch {
    Write-Host "Failed to create relationship: $_" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "3. Querying relationships..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/relationships?source_id=$symbol1Id" -Method Get
    Write-Host "Found $($response.Count) relationships" -ForegroundColor Green
} catch {
    Write-Host "Query failed: $_" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
