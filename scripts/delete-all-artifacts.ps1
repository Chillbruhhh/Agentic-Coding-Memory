#!/usr/bin/env pwsh
param(
    [string]$ServerUrl = "http://localhost:8105",
    [switch]$DryRun
)

# Delete all artifacts from AMP (decisions, notes, changesets, filelogs)

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  AMP Delete All Artifacts" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Checking AMP server status..." -ForegroundColor Yellow
try {
    $null = Invoke-RestMethod -Uri "$ServerUrl/health" -Method Get -TimeoutSec 5
    Write-Host " Server is running" -ForegroundColor Green
} catch {
    Write-Host "[FAIL] Server is not running at $ServerUrl" -ForegroundColor Red
    exit 1
}

$typeFilters = @("decision", "filelog", "note", "changeset", "Decision", "FileLog", "Note", "ChangeSet", "Changeset")

Write-Host ""
Write-Host "Fetching artifacts..." -ForegroundColor Yellow

$payload = @{
    limit = 2000
    filters = @{
        type = $typeFilters
    }
} | ConvertTo-Json -Depth 5

try {
    $response = Invoke-RestMethod -Uri "$ServerUrl/v1/query" -Method Post -ContentType "application/json" -Body $payload
} catch {
    Write-Host "[FAIL] Failed to query artifacts: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

$objects = @()
if ($response.results -and $response.results.Count -gt 0) {
    $objects = $response.results | ForEach-Object {
        if ($_.object) { $_.object } else { $_ }
    }
} elseif ($response -is [System.Array]) {
    $objects = $response
}

$artifacts = $objects | Where-Object { $typeFilters -contains $_.type }

if (-not $artifacts -or $artifacts.Count -eq 0) {
    Write-Host "No artifacts found." -ForegroundColor Gray
    exit 0
}

Write-Host "Found $($artifacts.Count) artifacts." -ForegroundColor Gray

if ($DryRun) {
    Write-Host "Dry run enabled. No deletions performed." -ForegroundColor Yellow
    exit 0
}

$deleted = 0
$failed = 0

foreach ($artifact in $artifacts) {
    $id = $artifact.id
    if (-not $id) {
        $failed++
        Write-Host "[FAIL] Missing id for artifact: $($artifact.title)" -ForegroundColor Red
        continue
    }
    try {
        Invoke-RestMethod -Uri "$ServerUrl/v1/artifacts/$id" -Method Delete -TimeoutSec 10 | Out-Null
        $deleted++
        Write-Host "Deleted: $($artifact.title)" -ForegroundColor Green
    } catch {
        $failed++
        Write-Host "[FAIL] $($artifact.title): $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Delete Complete" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Deleted: $deleted" -ForegroundColor Green
Write-Host "Failed:  $failed" -ForegroundColor $(if ($failed -gt 0) { "Red" } else { "Gray" })
Write-Host ""

if ($failed -gt 0) {
    exit 1
}

exit 0
