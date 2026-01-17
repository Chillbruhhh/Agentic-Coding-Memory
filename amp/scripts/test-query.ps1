Write-Host "=== AMP Query Endpoint Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"

# Create test objects
Write-Host "1. Creating test objects..." -ForegroundColor Yellow

$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$symbol1 = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "query_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test symbol for query"
    },
    "links": [],
    "embedding": null,
    "name": "authenticate_user",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn authenticate_user(username: &str, password: &str) -> Result<User>",
    "documentation": "Authenticates a user with username and password"
}
"@

$symbol2 = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "query_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test symbol for query"
    },
    "links": [],
    "embedding": null,
    "name": "hash_password",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn hash_password(password: &str) -> String",
    "documentation": "Hashes a password using bcrypt"
}
"@

$decision1 = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "decision",
    "tenant_id": "test",
    "project_id": "query_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test decision for query"
    },
    "links": [],
    "embedding": null,
    "title": "Use bcrypt for password hashing",
    "problem": "Need secure password storage",
    "options": null,
    "rationale": "bcrypt is industry standard and resistant to rainbow tables",
    "outcome": "Implemented bcrypt hashing in auth module",
    "status": "accepted"
}
"@

try {
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol1 -ContentType "application/json" | Out-Null
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol2 -ContentType "application/json" | Out-Null
    Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $decision1 -ContentType "application/json" | Out-Null
    Write-Host "Created 3 test objects" -ForegroundColor Green
} catch {
    Write-Host "Failed to create test objects: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Test 2: Query by type
Write-Host "2. Querying symbols only..." -ForegroundColor Yellow
$query = @{
    filters = @{
        type = @("symbol")  # Array, not string
        project_id = "query_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $query -ContentType "application/json"
    Write-Host "Found $($response.results.Count) symbols" -ForegroundColor Green
} catch {
    Write-Host "Query failed: $_" -ForegroundColor Yellow
}
Write-Host ""

# Test 3: Text search
Write-Host "3. Searching for 'password'..." -ForegroundColor Yellow
$query = @{
    text = "password"
    filters = @{
        project_id = "query_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $query -ContentType "application/json"
    Write-Host "Found $($response.results.Count) results" -ForegroundColor Green
} catch {
    Write-Host "Search failed: $_" -ForegroundColor Yellow
}
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
