# AMP Lease Coordination Test Script
$BASE_URL = "http://localhost:8105"

Write-Host "=== AMP Lease Coordination Test ===" -ForegroundColor Cyan

# 1. Acquire a lease
Write-Host "`n1. Acquiring lease on resource 'test-file.rs'..." -ForegroundColor Yellow
$acquireData = @"
{
    "resource": "test-file.rs",
    "holder": "agent-1",
    "ttl_seconds": 60
}
"@

try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/leases/acquire" -Method Post -Body $acquireData -ContentType "application/json"
    $leaseId = $response.lease_id
    Write-Host "Lease acquired: $leaseId" -ForegroundColor Green
    Write-Host "Expires at: $($response.expires_at)" -ForegroundColor Gray
} catch {
    Write-Host "Failed to acquire lease: $_" -ForegroundColor Red
    exit 1
}

# 2. Try to acquire same resource (should fail)
Write-Host "`n2. Attempting to acquire same resource (should fail)..." -ForegroundColor Yellow
$conflictData = @"
{
    "resource": "test-file.rs",
    "holder": "agent-2",
    "ttl_seconds": 60
}
"@

try {
    Invoke-RestMethod -Uri "$BASE_URL/v1/leases/acquire" -Method Post -Body $conflictData -ContentType "application/json"
    Write-Host "ERROR: Should have gotten 409 Conflict!" -ForegroundColor Red
} catch {
    if ($_.Exception.Response.StatusCode -eq 409) {
        Write-Host "Correctly rejected (409 Conflict)" -ForegroundColor Green
    } else {
        Write-Host "Unexpected error: $_" -ForegroundColor Red
    }
}

# 3. Renew the lease
Write-Host "`n3. Renewing lease..." -ForegroundColor Yellow
$renewData = @"
{
    "lease_id": "$leaseId",
    "ttl_seconds": 120
}
"@

try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/leases/renew" -Method Post -Body $renewData -ContentType "application/json"
    Write-Host "Lease renewed, new expiration: $($response.expires_at)" -ForegroundColor Green
} catch {
    Write-Host "Renew failed: $_" -ForegroundColor Yellow
}

# 4. Release the lease
Write-Host "`n4. Releasing lease..." -ForegroundColor Yellow
$releaseData = @"
{
    "lease_id": "$leaseId"
}
"@

try {
    Invoke-RestMethod -Uri "$BASE_URL/v1/leases/release" -Method Post -Body $releaseData -ContentType "application/json"
    Write-Host "Lease released successfully" -ForegroundColor Green
} catch {
    Write-Host "Failed to release: $_" -ForegroundColor Red
    exit 1
}

# 5. Acquire again (should succeed now)
Write-Host "`n5. Acquiring same resource again (should succeed)..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/v1/leases/acquire" -Method Post -Body $acquireData -ContentType "application/json"
    Write-Host "Successfully acquired: $($response.lease_id)" -ForegroundColor Green
    
    # Clean up
    $cleanupData = @"
{
    "lease_id": "$($response.lease_id)"
}
"@
    Invoke-RestMethod -Uri "$BASE_URL/v1/leases/release" -Method Post -Body $cleanupData -ContentType "application/json" | Out-Null
} catch {
    Write-Host "Failed to re-acquire: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
