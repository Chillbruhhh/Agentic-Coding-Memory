#!/usr/bin/env pwsh

# Simple test script for AMP codebase parser

Write-Host "Testing AMP Codebase Parser..." -ForegroundColor Cyan

# Check if server is running
$serverUrl = "http://localhost:8105"
try {
    $health = Invoke-RestMethod -Uri "$serverUrl/health" -Method GET
    Write-Host "Server is running: $($health.service)" -ForegroundColor Green
} catch {
    Write-Host "Server is not running. Start with: cd amp/server; cargo run" -ForegroundColor Red
    exit 1
}

# Create test files
Write-Host "Creating test files..." -ForegroundColor Yellow

$testDir = "test-codebase"
if (Test-Path $testDir) {
    Remove-Item -Recurse -Force $testDir
}
New-Item -ItemType Directory -Path $testDir | Out-Null

# Create Python test file
$pythonCode = @"
def hello_world():
    print("Hello, world!")

class MyClass:
    def method(self):
        pass

import os
"@

$pythonCode | Out-File -FilePath "$testDir/test.py" -Encoding UTF8

Write-Host "Created test.py" -ForegroundColor Green

# Test parsing Python file
Write-Host "Testing Python file parsing..." -ForegroundColor Yellow

$parseRequest = @{
    file_path = (Resolve-Path "$testDir/test.py").Path
    language = "python"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$serverUrl/v1/codebase/parse-file" -Method POST -Body $parseRequest -ContentType "application/json"
    
    Write-Host "SUCCESS: Python file parsed!" -ForegroundColor Green
    Write-Host "Symbols found: $($response.file_log.symbols.Count)" -ForegroundColor Gray
    
    foreach ($symbol in $response.file_log.symbols) {
        Write-Host "  - $($symbol.symbol_type): $($symbol.name)" -ForegroundColor Gray
    }
    
    Write-Host "Generated Markdown:" -ForegroundColor Cyan
    Write-Host $response.markdown -ForegroundColor White
    
} catch {
    Write-Host "FAILED: $($_.Exception.Message)" -ForegroundColor Red
}

# Cleanup
Write-Host "Cleaning up..." -ForegroundColor Yellow
Remove-Item -Recurse -Force $testDir

Write-Host "Test complete!" -ForegroundColor Green
