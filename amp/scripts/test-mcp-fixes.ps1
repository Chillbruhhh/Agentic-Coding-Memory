#!/usr/bin/env pwsh
# Test script for MCP integration fixes

$ErrorActionPreference = "Stop"
$SERVER_URL = "http://localhost:8105"

Write-Host "=== Testing MCP Integration Fixes ===" -ForegroundColor Cyan

# Test 1: Lease Acquire
Write-Host "`n[Test 1] Testing lease acquisition..." -ForegroundColor Yellow
$leasePayload = @{
    resource = "test-resource"
    agent_id = "test-agent"
    duration = 60
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$SERVER_URL/v1/leases/acquire" -Method Post -Body $leasePayload -ContentType "application/json"
    Write-Host "✓ Lease acquired successfully" -ForegroundColor Green
    Write-Host "  Lease ID: $($response.lease_id)" -ForegroundColor Gray
    $leaseId = $response.lease_id
} catch {
    Write-Host "✗ Lease acquisition failed: $($_.Exception.Message)" -ForegroundColor Red
    $leaseId = $null
}

# Test 2: Lease Release
if ($leaseId) {
    Write-Host "`n[Test 2] Testing lease release..." -ForegroundColor Yellow
    $releasePayload = @{
        lease_id = $leaseId
    } | ConvertTo-Json

    try {
        Invoke-RestMethod -Uri "$SERVER_URL/v1/leases/release" -Method Post -Body $releasePayload -ContentType "application/json"
        Write-Host "✓ Lease released successfully" -ForegroundColor Green
    } catch {
        Write-Host "✗ Lease release failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 3: File Log Get (with path resolution)
Write-Host "`n[Test 3] Testing file log retrieval..." -ForegroundColor Yellow
$testPath = "amp/server/src/main.rs"
try {
    $response = Invoke-RestMethod -Uri "$SERVER_URL/v1/codebase/file-logs/$testPath" -Method Get
    Write-Host "✓ File log retrieved successfully" -ForegroundColor Green
    Write-Host "  File: $($response.file_log.file_path)" -ForegroundColor Gray
    Write-Host "  Symbols: $($response.file_log.symbols.Count)" -ForegroundColor Gray
} catch {
    Write-Host "✗ File log retrieval failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Run Update (partial update)
Write-Host "`n[Test 4] Testing run update..." -ForegroundColor Yellow

# First create a run
$runPayload = @{
    type = "run"
    agent_name = "test-agent"
    goal = "Test run update"
    repo_id = "test-repo"
    tenant_id = "test-tenant"
    project_id = "test-project"
} | ConvertTo-Json

try {
    $createResponse = Invoke-RestMethod -Uri "$SERVER_URL/v1/objects" -Method Post -Body $runPayload -ContentType "application/json"
    $runId = $createResponse.id
    Write-Host "  Created run: $runId" -ForegroundColor Gray

    # Now update it with partial data
    $updatePayload = @{
        status = "completed"
        summary = "Test completed successfully"
        outputs = @("output1", "output2")
    } | ConvertTo-Json

    Invoke-RestMethod -Uri "$SERVER_URL/v1/objects/$runId" -Method Put -Body $updatePayload -ContentType "application/json"
    Write-Host "✓ Run updated successfully" -ForegroundColor Green
} catch {
    Write-Host "✗ Run update failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n=== Test Summary ===" -ForegroundColor Cyan
Write-Host "All critical MCP integration fixes have been tested." -ForegroundColor White
