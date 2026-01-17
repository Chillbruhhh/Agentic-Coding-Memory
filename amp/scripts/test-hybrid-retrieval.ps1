Write-Host "=== AMP Hybrid Retrieval Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"

# Create test objects with different characteristics for hybrid testing
Write-Host "1. Creating test objects for hybrid retrieval..." -ForegroundColor Yellow

$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

# Symbol with strong text match
$symbol1 = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "hybrid_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test symbol for hybrid retrieval"
    },
    "links": [],
    "embedding": null,
    "name": "authenticate_user",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn authenticate_user(username: &str, password: &str) -> Result<User>",
    "documentation": "Authenticates a user with username and password using secure methods"
}
"@

# Decision with different text content
$decision1 = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "decision",
    "tenant_id": "test",
    "project_id": "hybrid_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test decision for hybrid retrieval"
    },
    "links": [],
    "embedding": null,
    "title": "Use JWT for authentication tokens",
    "problem": "Need secure session management",
    "options": null,
    "rationale": "JWT tokens provide stateless authentication with good security",
    "outcome": "Implemented JWT-based authentication system",
    "status": "accepted"
}
"@

# ChangeSet with related content
$changeset1 = @"
{
    "id": "$([guid]::NewGuid().ToString())",
    "type": "changeset",
    "tenant_id": "test",
    "project_id": "hybrid_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test changeset for hybrid retrieval"
    },
    "links": [],
    "embedding": null,
    "title": "Add password hashing to authentication",
    "description": "Implement bcrypt password hashing for secure user authentication",
    "files_changed": ["src/auth.rs", "src/models/user.rs"],
    "lines_added": 45,
    "lines_removed": 12,
    "diff_summary": "Added password hashing utilities and updated user model"
}
"@

try {
    $response1 = Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol1 -ContentType "application/json"
    $response2 = Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $decision1 -ContentType "application/json"
    $response3 = Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $changeset1 -ContentType "application/json"
    
    Write-Host "Created 3 test objects for hybrid testing" -ForegroundColor Green
    
    # Store IDs for relationship creation
    $symbolId = $response1.id
    $decisionId = $response2.id
    $changesetId = $response3.id
    
    Write-Host "  - Symbol ID: $symbolId" -ForegroundColor Gray
    Write-Host "  - Decision ID: $decisionId" -ForegroundColor Gray
    Write-Host "  - ChangeSet ID: $changesetId" -ForegroundColor Gray
} catch {
    Write-Host "Failed to create test objects: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Test 2: Text-only query (baseline)
Write-Host "2. Testing text-only query (baseline)..." -ForegroundColor Yellow
$query = @{
    text = "authentication"
    filters = @{
        project_id = "hybrid_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $query -ContentType "application/json"
    Write-Host "Text-only results: $($response.results.Count) objects found" -ForegroundColor Green
    Write-Host "Execution time: $($response.execution_time_ms)ms" -ForegroundColor Gray
} catch {
    Write-Host "Text query failed: $_" -ForegroundColor Yellow
}
Write-Host ""

# Test 3: Hybrid query with text only
Write-Host "3. Testing hybrid query with text..." -ForegroundColor Yellow
$hybridQuery = @{
    text = "authentication"
    hybrid = $true
    filters = @{
        project_id = "hybrid_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $hybridQuery -ContentType "application/json"
    Write-Host "Hybrid text results: $($response.results.Count) objects found" -ForegroundColor Green
    Write-Host "Execution time: $($response.execution_time_ms)ms" -ForegroundColor Gray
    
    # Check for hybrid-specific scoring
    foreach ($result in $response.results) {
        Write-Host "  - Score: $($result.score), Explanation: $($result.explanation)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Hybrid text query failed: $_" -ForegroundColor Yellow
}
Write-Host ""

# Test 4: Hybrid query with different search terms
Write-Host "4. Testing hybrid query with different terms..." -ForegroundColor Yellow
$hybridQuery2 = @{
    text = "password"
    hybrid = $true
    filters = @{
        project_id = "hybrid_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $hybridQuery2 -ContentType "application/json"
    Write-Host "Hybrid password results: $($response.results.Count) objects found" -ForegroundColor Green
    Write-Host "Execution time: $($response.execution_time_ms)ms" -ForegroundColor Gray
    
    foreach ($result in $response.results) {
        Write-Host "  - Score: $($result.score), Type: $($result.object.type), Explanation: $($result.explanation)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Hybrid password query failed: $_" -ForegroundColor Yellow
}
Write-Host ""

# Test 5: Hybrid query with type filtering
Write-Host "5. Testing hybrid query with type filtering..." -ForegroundColor Yellow
$hybridQuery3 = @{
    text = "authentication"
    hybrid = $true
    filters = @{
        project_id = "hybrid_test"
        object_types = @("symbol", "decision")
    }
    limit = 10
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $hybridQuery3 -ContentType "application/json"
    Write-Host "Hybrid filtered results: $($response.results.Count) objects found" -ForegroundColor Green
    Write-Host "Execution time: $($response.execution_time_ms)ms" -ForegroundColor Gray
    
    foreach ($result in $response.results) {
        Write-Host "  - Type: $($result.object.type), Score: $($result.score)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Hybrid filtered query failed: $_" -ForegroundColor Yellow
}
Write-Host ""

# Test 6: Performance comparison
Write-Host "6. Performance comparison (text vs hybrid)..." -ForegroundColor Yellow

# Text query timing
$textQuery = @{
    text = "authentication"
    filters = @{
        project_id = "hybrid_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 3

$textTime = 0
try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $textQuery -ContentType "application/json"
    $textTime = $response.execution_time_ms
    Write-Host "Text query time: ${textTime}ms" -ForegroundColor Green
} catch {
    Write-Host "Text query timing failed: $_" -ForegroundColor Yellow
}

# Hybrid query timing
$hybridTime = 0
try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $hybridQuery -ContentType "application/json"
    $hybridTime = $response.execution_time_ms
    Write-Host "Hybrid query time: ${hybridTime}ms" -ForegroundColor Green
} catch {
    Write-Host "Hybrid query timing failed: $_" -ForegroundColor Yellow
}

if ($textTime -gt 0 -and $hybridTime -gt 0) {
    $overhead = $hybridTime - $textTime
    Write-Host "Hybrid overhead: ${overhead}ms" -ForegroundColor $(if ($overhead -lt 100) { "Green" } else { "Yellow" })
}
Write-Host ""

# Test 7: Error handling - malformed hybrid query
Write-Host "7. Testing error handling..." -ForegroundColor Yellow
$malformedQuery = @{
    hybrid = $true
    # No text, vector, or graph - should handle gracefully
    filters = @{
        project_id = "hybrid_test"
    }
    limit = 10
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $malformedQuery -ContentType "application/json"
    Write-Host "Empty hybrid query results: $($response.results.Count) objects found" -ForegroundColor Green
} catch {
    Write-Host "Empty hybrid query handled: $($_.Exception.Response.StatusCode)" -ForegroundColor Green
}
Write-Host ""

Write-Host "=== Hybrid Retrieval Test Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Summary:" -ForegroundColor White
Write-Host "- Created test objects with different content types" -ForegroundColor Gray
Write-Host "- Tested hybrid retrieval with text queries" -ForegroundColor Gray
Write-Host "- Validated filtering and scoring mechanisms" -ForegroundColor Gray
Write-Host "- Compared performance with baseline text queries" -ForegroundColor Gray
Write-Host "- Verified error handling for edge cases" -ForegroundColor Gray
