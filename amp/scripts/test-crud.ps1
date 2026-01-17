# AMP CRUD Test Script for Windows PowerShell

$BASE_URL = "http://localhost:8105"

Write-Host "=== AMP CRUD Operations Test ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Health Check
Write-Host "1. Testing health endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/health" -Method Get
    $response | ConvertTo-Json
    Write-Host "Success: Health check passed" -ForegroundColor Green
} catch {
    Write-Host "Failed: Health check failed: $_" -ForegroundColor Red
}
Write-Host ""

# Test 2: Create a Symbol object
Write-Host "2. Creating a Symbol object..." -ForegroundColor Yellow
$symbolId = [guid]::NewGuid().ToString()
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$symbolData = @"
{
    "id": "$symbolId",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "amp_demo",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "powershell_test",
        "summary": "Testing symbol creation"
    },
    "links": [],
    "embedding": null,
    "name": "main",
    "kind": "function",
    "path": "src/main.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn main()",
    "documentation": "Entry point for AMP server"
}
"@

try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/objects" -Method Post -Body $symbolData -ContentType "application/json"
    $response | ConvertTo-Json
    Write-Host "Success: Symbol created with ID: $($response.id)" -ForegroundColor Green
    $createdId = $response.id
} catch {
    Write-Host "Failed: Symbol creation failed" -ForegroundColor Red
    Write-Host $_.Exception.Message
}
Write-Host ""

# Test 3: Retrieve the object
if ($createdId) {
    Write-Host "3. Retrieving the created object..." -ForegroundColor Yellow
    try {
        $response = Invoke-RestMethod -Uri "$BASE_URL/v1/objects/$createdId" -Method Get
        $response | ConvertTo-Json -Depth 5
        Write-Host "Success: Object retrieved" -ForegroundColor Green
    } catch {
        Write-Host "Failed: Object retrieval failed" -ForegroundColor Red
        Write-Host $_.Exception.Message
    }
    Write-Host ""
}

# Test 4: Create a Decision object
Write-Host "4. Creating a Decision object..." -ForegroundColor Yellow
$decisionId = [guid]::NewGuid().ToString()

$decisionData = @"
{
    "id": "$decisionId",
    "type": "decision",
    "tenant_id": "test",
    "project_id": "amp_demo",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "powershell_test",
        "summary": "Testing decision creation"
    },
    "links": [],
    "embedding": null,
    "title": "Use SurrealDB for storage",
    "problem": "Need embedded database with vector and graph support",
    "options": null,
    "rationale": "SurrealDB provides all required features in single package",
    "outcome": "Implemented with embedded mode",
    "status": "accepted"
}
"@

try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/objects" -Method Post -Body $decisionData -ContentType "application/json"
    $response | ConvertTo-Json
    Write-Host "Success: Decision created with ID: $($response.id)" -ForegroundColor Green
} catch {
    Write-Host "Failed: Decision creation failed" -ForegroundColor Red
    Write-Host $_.Exception.Message
}
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
