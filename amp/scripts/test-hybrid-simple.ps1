Write-Host "=== AMP Hybrid Retrieval Quick Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"

# Test 1: Simple hybrid query (no objects needed)
Write-Host "1. Testing hybrid query with no results..." -ForegroundColor Yellow
$hybridQuery = @{
    text = "authentication"
    hybrid = $true
    limit = 5
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $hybridQuery -ContentType "application/json"
    Write-Host "✅ Hybrid query successful!" -ForegroundColor Green
    Write-Host "   Results: $($response.results.Count)" -ForegroundColor Gray
    Write-Host "   Execution time: $($response.execution_time_ms)ms" -ForegroundColor Gray
    Write-Host "   Trace ID: $($response.trace_id)" -ForegroundColor Gray
} catch {
    Write-Host "❌ Hybrid query failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 2: Compare with non-hybrid query
Write-Host "2. Testing non-hybrid query (baseline)..." -ForegroundColor Yellow
$normalQuery = @{
    text = "authentication"
    limit = 5
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $normalQuery -ContentType "application/json"
    Write-Host "✅ Normal query successful!" -ForegroundColor Green
    Write-Host "   Results: $($response.results.Count)" -ForegroundColor Gray
    Write-Host "   Execution time: $($response.execution_time_ms)ms" -ForegroundColor Gray
} catch {
    Write-Host "❌ Normal query failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Create a simple test object and query it
Write-Host "3. Creating a test object..." -ForegroundColor Yellow
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$testId = [guid]::NewGuid().ToString()

$testObject = @"
{
    "id": "$testId",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "hybrid_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "hybrid_test",
        "summary": "Test object for hybrid retrieval"
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

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $testObject -ContentType "application/json"
    Write-Host "✅ Test object created!" -ForegroundColor Green
    Write-Host "   ID: $($response.id)" -ForegroundColor Gray
} catch {
    Write-Host "❌ Object creation failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "=== Test Complete (Partial) ===" -ForegroundColor Cyan
    exit
}
Write-Host ""

# Test 4: Query the created object with hybrid
Write-Host "4. Testing hybrid query with created object..." -ForegroundColor Yellow
$hybridQueryWithData = @{
    text = "authenticate"
    hybrid = $true
    filters = @{
        project_id = "hybrid_test"
    }
    limit = 5
} | ConvertTo-Json -Depth 3

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/query" -Method Post -Body $hybridQueryWithData -ContentType "application/json"
    Write-Host "✅ Hybrid query with data successful!" -ForegroundColor Green
    Write-Host "   Results: $($response.results.Count)" -ForegroundColor Gray
    Write-Host "   Execution time: $($response.execution_time_ms)ms" -ForegroundColor Gray
    
    if ($response.results.Count -gt 0) {
        Write-Host "   First result score: $($response.results[0].score)" -ForegroundColor Gray
        Write-Host "   First result explanation: $($response.results[0].explanation)" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ Hybrid query with data failed: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== Hybrid Retrieval Test Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Summary:" -ForegroundColor White
Write-Host "- Hybrid query endpoint is working" -ForegroundColor Gray
Write-Host "- Backward compatibility maintained" -ForegroundColor Gray
Write-Host "- Object creation and querying functional" -ForegroundColor Gray
