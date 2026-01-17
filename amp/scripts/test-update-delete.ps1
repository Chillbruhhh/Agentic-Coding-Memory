# AMP Update/Delete Test Script
$BASE_URL = "http://localhost:8105"

Write-Host "=== AMP Update/Delete Test ===" -ForegroundColor Cyan

# 1. Create a test object
Write-Host "`n1. Creating test object..." -ForegroundColor Yellow
$testId = [guid]::NewGuid().ToString()
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$createData = @"
{
    "id": "$testId",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "test_update_delete",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Test object"},
    "links": [],
    "embedding": null,
    "name": "test_function",
    "kind": "function",
    "path": "test.rs",
    "language": "rust",
    "content_hash": null,
    "signature": null,
    "documentation": null
}
"@

try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/objects" -Method Post -Body $createData -ContentType "application/json"
    Write-Host "Created object: $testId" -ForegroundColor Green
} catch {
    Write-Host "Failed to create: $_" -ForegroundColor Red
    exit 1
}

# 2. Update the object
Write-Host "`n2. Updating object..." -ForegroundColor Yellow
$updateData = @"
{
    "id": "$testId",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "test_update_delete",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {"agent": "test", "summary": "Updated object"},
    "links": [],
    "embedding": null,
    "name": "updated_function",
    "kind": "function",
    "path": "test.rs",
    "language": "rust",
    "content_hash": null,
    "signature": null,
    "documentation": null
}
"@

try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/objects/$testId" -Method Put -Body $updateData -ContentType "application/json"
    Write-Host "Updated successfully" -ForegroundColor Green
} catch {
    Write-Host "Failed to update: $_" -ForegroundColor Red
    exit 1
}

# 3. Verify update
Write-Host "`n3. Verifying update..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/objects/$testId" -Method Get
    if ($response.name -eq "updated_function") {
        Write-Host "Update verified!" -ForegroundColor Green
    } else {
        Write-Host "Update failed - name not changed" -ForegroundColor Red
    }
} catch {
    Write-Host "Failed to verify: $_" -ForegroundColor Red
}

# 4. Delete the object
Write-Host "`n4. Deleting object..." -ForegroundColor Yellow
try {
    Invoke-RestMethod -Uri "$BASE_URL/v1/objects/$testId" -Method Delete
    Write-Host "Deleted successfully" -ForegroundColor Green
} catch {
    Write-Host "Failed to delete: $_" -ForegroundColor Red
    exit 1
}

# 5. Verify deletion
Write-Host "`n5. Verifying deletion..." -ForegroundColor Yellow
try {
    Invoke-RestMethod -Uri "$BASE_URL/v1/objects/$testId" -Method Get
    Write-Host "ERROR: Object still exists!" -ForegroundColor Red
} catch {
    if ($_.Exception.Response.StatusCode -eq 404) {
        Write-Host "Deletion verified (404 Not Found)" -ForegroundColor Green
    } else {
        Write-Host "Unexpected error: $_" -ForegroundColor Red
    }
}

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
