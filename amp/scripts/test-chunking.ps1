# Test FileChunk Creation
# Tests chunking of large files and embedding generation

$ErrorActionPreference = "Stop"

Write-Host "Testing FileChunk functionality..." -ForegroundColor Green

# Create a large test file (>1000 tokens)
$testContent = @"
def function_one():
    '''First function with documentation'''
    result = []
    for i in range(100):
        result.append(i * 2)
    return result

def function_two():
    '''Second function'''
    data = function_one()
    return sum(data)

class MyClass:
    '''A test class'''
    def __init__(self):
        self.value = 0
    
    def method_one(self):
        return self.value * 2
    
    def method_two(self):
        return self.value + 10

def function_three():
    '''Third function'''
    obj = MyClass()
    return obj.method_one()
"@ * 10  # Repeat to make it large

$testFile = "test_large_file.py"
$testContent | Out-File -FilePath $testFile -Encoding UTF8

Write-Host "Created test file: $testFile" -ForegroundColor Cyan

# Index the file (this should create FileChunks)
Write-Host "`nIndexing file..." -ForegroundColor Yellow
amp index --path . --exclude "target,node_modules"

# Query for FileChunk objects
Write-Host "`nQuerying for FileChunk objects..." -ForegroundColor Yellow
$response = Invoke-RestMethod -Uri "http://localhost:8105/v1/query" -Method Post -ContentType "application/json" -Body (@{
    filters = @{
        type = @("FileChunk")
    }
    limit = 10
} | ConvertTo-Json)

Write-Host "Found $($response.results.Count) FileChunk objects" -ForegroundColor Green

if ($response.results.Count -gt 0) {
    Write-Host "`nFirst chunk details:" -ForegroundColor Cyan
    $chunk = $response.results[0]
    Write-Host "  File: $($chunk.file_path)"
    Write-Host "  Chunk Index: $($chunk.chunk_index)"
    Write-Host "  Token Count: $($chunk.token_count)"
    Write-Host "  Lines: $($chunk.start_line)-$($chunk.end_line)"
    Write-Host "  Has Embedding: $($null -ne $chunk.embedding)"
}

# Cleanup
Remove-Item $testFile -ErrorAction SilentlyContinue

Write-Host "`n✓ FileChunk test complete!" -ForegroundColor Green
