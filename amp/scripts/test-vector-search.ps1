Write-Host "=== AMP Vector Search Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

Write-Host "1. Creating test objects..." -ForegroundColor Yellow

$objects = @(
    @{ name = "authenticate_user"; doc = "Authenticates a user with username and password" },
    @{ name = "hash_password"; doc = "Hashes a password using bcrypt for secure storage" },
    @{ name = "send_email"; doc = "Sends an email notification to a user" }
)

foreach ($obj in $objects) {
    $symbol = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "vector_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test vector search"
    },
    "links": [],
    "embedding": null,
    "name": "$($obj.name)",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn $($obj.name)()",
    "documentation": "$($obj.doc)"
}
"@
    try {
        Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol -ContentType "application/json" | Out-Null
    } catch {
        Write-Host "Failed to create $($obj.name): $_" -ForegroundColor Red
    }
}

Write-Host "Created 3 test objects" -ForegroundColor Green
Write-Host ""

Write-Host "2. Testing semantic search for 'user login security'..." -ForegroundColor Yellow
$query = @{
    text = "user login security"
    filters = @{
        project_id = "vector_test"
    }
    limit = 5
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $query -ContentType "application/json"
    Write-Host "Found $($response.results.Count) results" -ForegroundColor Green
    foreach ($result in $response.results) {
        Write-Host "  - $($result.name): $($result.documentation)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Search failed: $_" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
