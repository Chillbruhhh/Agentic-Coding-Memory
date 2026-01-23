#!/usr/bin/env pwsh
# Import test artifacts into AMP for UI testing

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  AMP Test Artifacts Import Script" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Configuration
$SERVER_URL = "http://localhost:8105"
$ARTIFACTS_DIR = "test-repo/artifacts"

# Check if server is running
Write-Host "Checking AMP server status..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$SERVER_URL/health" -Method Get -TimeoutSec 5
    Write-Host " Server is running" -ForegroundColor Green
} catch {
    Write-Host "[FAIL] Server is not running at $SERVER_URL" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please start the AMP server first:" -ForegroundColor Yellow
    Write-Host "  cd amp/server" -ForegroundColor White
    Write-Host "  cargo run --release" -ForegroundColor White
    Write-Host ""
    exit 1
}

# Artifact files to import
$artifactFiles = @(
    @{ Path = "$ARTIFACTS_DIR/decisions.json"; Type = "decision"; Color = "Yellow" },
    @{ Path = "$ARTIFACTS_DIR/notes.json"; Type = "note"; Color = "Green" },
    @{ Path = "$ARTIFACTS_DIR/changesets.json"; Type = "changeset"; Color = "Magenta" },
    @{ Path = "$ARTIFACTS_DIR/filelogs.json"; Type = "filelog"; Color = "Cyan" }
)

$totalImported = 0
$totalFailed = 0

foreach ($file in $artifactFiles) {
    $filePath = $file.Path
    $artifactType = $file.Type
    $color = $file.Color
    
    if (-not (Test-Path $filePath)) {
        Write-Host "[FAIL] File not found: $filePath" -ForegroundColor Red
        continue
    }
    
    Write-Host ""
    Write-Host "Importing $artifactType artifacts from $filePath..." -ForegroundColor $color
    
    try {
        $content = Get-Content $filePath -Raw | ConvertFrom-Json
        $count = $content.Count
        
        Write-Host "  Found $count artifacts to import" -ForegroundColor Gray
        
        $imported = 0
        $failed = 0
        
        foreach ($artifact in $content) {
            try {
                # Convert to JSON with proper depth
                $json = $artifact | ConvertTo-Json -Depth 10 -Compress
                
                # Send to API
                $response = Invoke-RestMethod `
                    -Uri "$SERVER_URL/v1/artifacts" `
                    -Method Post `
                    -ContentType "application/json" `
                    -Body $json `
                    -TimeoutSec 10
                
                $imported++
                Write-Host "   Imported: $($artifact.title)" -ForegroundColor Green
                
            } catch {
                $failed++
                Write-Host "  [FAIL] Failed: $($artifact.title)" -ForegroundColor Red
                Write-Host "    Error: $($_.Exception.Message)" -ForegroundColor DarkRed
            }
        }
        
        $totalImported += $imported
        $totalFailed += $failed
        
        Write-Host "  Summary: $imported imported, $failed failed" -ForegroundColor Gray
        
    } catch {
        Write-Host "[FAIL] Failed to read file: $filePath" -ForegroundColor Red
        Write-Host "  Error: $($_.Exception.Message)" -ForegroundColor DarkRed
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Import Complete" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Total Imported: $totalImported" -ForegroundColor Green
Write-Host "Total Failed:   $totalFailed" -ForegroundColor $(if ($totalFailed -gt 0) { "Red" } else { "Gray" })
Write-Host ""

if ($totalImported -gt 0) {
    Write-Host "Next Steps:" -ForegroundColor Yellow
    Write-Host "1. Open the AMP UI at http://localhost:5173" -ForegroundColor White
    Write-Host "2. Navigate to the Artifacts tab" -ForegroundColor White
    Write-Host "3. Verify all $totalImported artifacts are visible" -ForegroundColor White
    Write-Host "4. Test filtering by type (Decision, Note, ChangeSet, FileLog)" -ForegroundColor White
    Write-Host "5. Click artifacts to view detailed information" -ForegroundColor White
    Write-Host ""
}

if ($totalFailed -gt 0) {
    Write-Host "Some artifacts failed to import. Check the errors above." -ForegroundColor Red
    exit 1
}

exit 0


