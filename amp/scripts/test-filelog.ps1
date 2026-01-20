# Test FileLog Creation
# Tests semantic summary generation for files

$ErrorActionPreference = "Stop"

Write-Host "Testing FileLog functionality..." -ForegroundColor Green

# Create test file with multiple symbols
$testContent = @"
import os
from typing import List

def calculate_sum(numbers: List[int]) -> int:
    '''Calculate sum of numbers'''
    return sum(numbers)

class DataProcessor:
    '''Process data efficiently'''
    def __init__(self):
        self.data = []
    
    def add_item(self, item):
        self.data.append(item)
    
    def process(self):
        return calculate_sum(self.data)
"@

$testFile = "test_filelog.py"
$testContent | Out-File -FilePath $testFile -Encoding UTF8

Write-Host "Created test file: $testFile" -ForegroundColor Cyan

# Index the file
Write-Host "`nIndexing file..." -ForegroundColor Yellow
amp index --path . --exclude "target,node_modules"

# Query for FileLog objects
Write-Host "`nQuerying for FileLog objects..." -ForegroundColor Yellow
$response = Invoke-RestMethod -Uri "http://localhost:8105/v1/query" -Method Post -ContentType "application/json" -Body (@{
    filters = @{
        type = @("FileLog")
    }
    limit = 10
} | ConvertTo-Json)

Write-Host "Found $($response.results.Count) FileLog objects" -ForegroundColor Green

if ($response.results.Count -gt 0) {
    Write-Host "`nFileLog details:" -ForegroundColor Cyan
    $log = $response.results[0]
    Write-Host "  File: $($log.file_path)"
    Write-Host "  Purpose: $($log.purpose)"
    Write-Host "  Key Symbols: $($log.key_symbols.Count)"
    Write-Host "  Dependencies: $($log.dependencies.Count)"
    Write-Host "  Has Embedding: $($null -ne $log.embedding)"
    Write-Host "`n  Summary Preview:"
    Write-Host "  $($log.summary.Substring(0, [Math]::Min(200, $log.summary.Length)))..."
}

# Cleanup
Remove-Item $testFile -ErrorAction SilentlyContinue

Write-Host "`n✓ FileLog test complete!" -ForegroundColor Green
