Write-Host "=== AMP Comprehensive Embedding Test ===" -ForegroundColor Cyan
Write-Host "Tests embedding generation with configured provider" -ForegroundColor Gray
Write-Host ""

$baseUrl = "http://localhost:8105"
$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

function Create-TestObject {
    param(
        [string]$Name,
        [string]$Doc,
        [string]$ProjectId
    )
    
    $id = [guid]::NewGuid().ToString()
    $symbol = @"
{
    "id": "$id",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "$ProjectId",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Comprehensive embedding test"
    },
    "links": [],
    "embedding": null,
    "name": "$Name",
    "kind": "function",
    "path": "src/lib.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn $Name()",
    "documentation": "$Doc"
}
"@
    
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol -ContentType "application/json"
    return $response.id
}

function Test-EmbeddingGeneration {
    Write-Host ""
    Write-Host "=== Testing Embedding Generation ===" -ForegroundColor Yellow
    Write-Host ""
    
    Write-Host "1. Creating test object..." -ForegroundColor Cyan
    try {
        $objectId = Create-TestObject -Name "test_auth" -Doc "Authenticates users securely" -ProjectId "embed_test"
        Write-Host "   Created: $objectId" -ForegroundColor Green
    } catch {
        Write-Host "   Failed to create: $_" -ForegroundColor Red
        return $false
    }
    
    Write-Host "2. Checking embedding generation..." -ForegroundColor Cyan
    try {
        $obj = Invoke-RestMethod -Uri "$baseUrl/v1/objects/$objectId" -Method Get
        
        if ($obj.embedding -ne $null) {
            $dim = $obj.embedding.Length
            Write-Host "   [OK] Embedding generated! Dimension: $dim" -ForegroundColor Green
            Write-Host "   First 5 values: $($obj.embedding[0..4] -join ', ')" -ForegroundColor DarkGray
            return $true
        } else {
            Write-Host "   [WARN] No embedding generated" -ForegroundColor Yellow
            return $false
        }
    } catch {
        Write-Host "   Failed to retrieve: $_" -ForegroundColor Red
        return $false
    }
}

# Run test
$result = Test-EmbeddingGeneration

Write-Host ""
if ($result) {
    Write-Host "Embedding generation is working!" -ForegroundColor Green
} else {
    Write-Host "Embedding generation not available or failed" -ForegroundColor Yellow
    Write-Host "Check EMBEDDING_PROVIDER environment variable" -ForegroundColor Gray
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
