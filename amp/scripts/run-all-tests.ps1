# Run All AMP Tests
Write-Host "=== AMP Test Suite ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"

# Check if server is running
Write-Host "Checking server status..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$baseUrl/health" -Method Get -TimeoutSec 10
    Write-Host "[OK] Server is running" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Host "[ERROR] Server is not running on $baseUrl" -ForegroundColor Red
    Write-Host "Please start the server first: cd server; cargo run" -ForegroundColor Yellow
    exit 1
}

$tests = @(
    @{Name = "CRUD Operations"; Script = "test-crud.ps1"},
    @{Name = "Update/Delete"; Script = "test-update-delete.ps1"},
    @{Name = "Lease Coordination"; Script = "test-leases.ps1"},
    @{Name = "Query Endpoint"; Script = "test-query.ps1"},
    @{Name = "Embeddings"; Script = "test-embeddings.ps1"},
    @{Name = "Vector Search"; Script = "test-vector-search.ps1"},
    @{Name = "Relationships"; Script = "test-relationships.ps1"},
    @{Name = "Graph Traversal"; Script = "test-graph-traversal.ps1"}
)

$passed = 0
$failed = 0

foreach ($test in $tests) {
    Write-Host "Running: $($test.Name)" -ForegroundColor Cyan
    Write-Host ("=" * 60) -ForegroundColor Gray
    
    try {
        & ".\$($test.Script)"
        $passed++
        Write-Host ""
        Write-Host "[PASS] $($test.Name)" -ForegroundColor Green
    } catch {
        $failed++
        Write-Host ""
        Write-Host "[FAIL] $($test.Name): $_" -ForegroundColor Red
    }
    
    Write-Host ""
    Write-Host ""
}

Write-Host ("=" * 60) -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host ("=" * 60) -ForegroundColor Cyan
Write-Host "Passed: $passed" -ForegroundColor Green
if ($failed -gt 0) {
    Write-Host "Failed: $failed" -ForegroundColor Red
} else {
    Write-Host "Failed: $failed" -ForegroundColor Green
}
Write-Host ""

if ($failed -eq 0) {
    Write-Host "All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some tests failed. Please review the output above." -ForegroundColor Yellow
    exit 1
}
