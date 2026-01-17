Write-Host "=== AMP Embedding Generation Test ===" -ForegroundColor Cyan
Write-Host ""

$baseUrl = "http://localhost:8105"

Write-Host "Testing embedding generation..." -ForegroundColor Yellow
Write-Host "Note: Set EMBEDDING_PROVIDER=openai or ollama to enable embeddings" -ForegroundColor Gray
Write-Host ""

$now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$objectId = [guid]::NewGuid().ToString()

$symbol = @"
{
    "id": "$objectId",
    "type": "symbol",
    "tenant_id": "test",
    "project_id": "embedding_test",
    "created_at": "$now",
    "updated_at": "$now",
    "provenance": {
        "agent": "test_script",
        "summary": "Test embedding generation"
    },
    "links": [],
    "embedding": null,
    "name": "authenticate_user",
    "kind": "function",
    "path": "src/auth.rs",
    "language": "rust",
    "content_hash": null,
    "signature": "fn authenticate_user(username: &str, password: &str) -> Result<User>",
    "documentation": "Authenticates a user with username and password using bcrypt"
}
"@

Write-Host "1. Creating object (embedding should be auto-generated)..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "$baseUrl/v1/objects" -Method Post -Body $symbol -ContentType "application/json"
    Write-Host "Created object: $($response.id)" -ForegroundColor Green
} catch {
    Write-Host "Failed to create: $_" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "2. Retrieving object to check embedding..." -ForegroundColor Yellow
try {
    $retrieved = Invoke-RestMethod -Uri "$baseUrl/v1/objects/$objectId" -Method Get

    if ($retrieved.embedding -ne $null) {
        $embeddingLength = $retrieved.embedding.Length
        Write-Host "[OK] Embedding generated! Dimension: $embeddingLength" -ForegroundColor Green
        Write-Host "First 5 values: $($retrieved.embedding[0..4] -join ', ')" -ForegroundColor Gray
    } else {
        Write-Host "[WARN] No embedding generated (provider may be disabled)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "Failed to retrieve: $_" -ForegroundColor Red
}
Write-Host ""

Write-Host "=== Test Complete ===" -ForegroundColor Cyan
