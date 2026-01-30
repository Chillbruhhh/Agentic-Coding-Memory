#!/usr/bin/env pwsh

Write-Host "Building AMP server..." -ForegroundColor Cyan
Set-Location "C:\Users\Joshc\source\repos\ACM\amp\server"

cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "`nBuild successful! Starting server..." -ForegroundColor Green

# Start server in background
$serverJob = Start-Job -ScriptBlock {
    Set-Location "C:\Users\Joshc\source\repos\ACM\amp\server"
    cargo run
}

Write-Host "Waiting for server to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

# Test health endpoint
Write-Host "`nTesting health endpoint..." -ForegroundColor Cyan
try {
    $health = Invoke-RestMethod -Uri "http://localhost:8105/health" -Method Get
    Write-Host "Health check: $($health | ConvertTo-Json)" -ForegroundColor Green
} catch {
    Write-Host "Health check failed: $_" -ForegroundColor Red
    Stop-Job $serverJob
    Remove-Job $serverJob
    exit 1
}

# Test object creation
Write-Host "`nTesting object creation..." -ForegroundColor Cyan
$testObject = @{
    id = "550e8400-e29b-41d4-a716-446655440000"
    type = "symbol"
    tenant_id = "test-tenant"
    project_id = "test-project"
    created_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
    updated_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
    provenance = @{
        agent = "test-agent"
        model = "test-model"
        tools = @("tool1")
        summary = "Test symbol"
    }
    links = @()
    name = "test_function"
    kind = "function"
    path = "/test/path.rs"
    language = "rust"
}

try {
    $response = Invoke-RestMethod -Uri "http://localhost:8105/v1/objects" -Method Post -Body ($testObject | ConvertTo-Json -Depth 10) -ContentType "application/json"
    Write-Host "Object created successfully!" -ForegroundColor Green
    Write-Host ($response | ConvertTo-Json) -ForegroundColor Green
} catch {
    Write-Host "Object creation failed: $_" -ForegroundColor Red
    Write-Host "Response: $($_.Exception.Response)" -ForegroundColor Red
}

# Cleanup
Write-Host "`nStopping server..." -ForegroundColor Yellow
Stop-Job $serverJob
Remove-Job $serverJob

Write-Host "`nTest complete!" -ForegroundColor Cyan
