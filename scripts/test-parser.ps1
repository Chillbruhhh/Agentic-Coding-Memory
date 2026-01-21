#!/usr/bin/env pwsh
# Test AMP's multi-language parser against the test repository

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "AMP Multi-Language Parser Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Get the script directory and project root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$TestRepoPath = Join-Path $ProjectRoot "test-repo"
$ServerPath = Join-Path $ProjectRoot "amp\server"

# Check if test-repo exists
if (-not (Test-Path $TestRepoPath)) {
    Write-Host "Error: test-repo directory not found at $TestRepoPath" -ForegroundColor Red
    exit 1
}

Write-Host "Test Repository: $TestRepoPath" -ForegroundColor Green
Write-Host "Server Path: $ServerPath" -ForegroundColor Green
Write-Host ""

# Count files by language
Write-Host "Test Repository Contents:" -ForegroundColor Yellow
Write-Host "-------------------------" -ForegroundColor Yellow

$languages = @{
    "Python" = "*.py"
    "TypeScript" = "*.ts"
    "JavaScript" = "*.js"
    "Rust" = "*.rs"
    "Go" = "*.go"
    "C#" = "*.cs"
    "Java" = "*.java"
    "C" = "*.c"
    "C++" = "*.cpp"
    "Ruby" = "*.rb"
}

$totalFiles = 0
foreach ($lang in $languages.Keys) {
    $pattern = $languages[$lang]
    $files = Get-ChildItem -Path $TestRepoPath -Filter $pattern -Recurse -File
    $count = $files.Count
    $totalFiles += $count
    
    if ($count -gt 0) {
        Write-Host "  $lang : $count file(s)" -ForegroundColor White
        foreach ($file in $files) {
            $relativePath = $file.FullName.Substring($TestRepoPath.Length + 1)
            Write-Host "    - $relativePath" -ForegroundColor Gray
        }
    }
}

Write-Host ""
Write-Host "Total files: $totalFiles" -ForegroundColor Green
Write-Host ""

# Build the server
Write-Host "Building AMP server..." -ForegroundColor Yellow
Push-Location $ServerPath
try {
    $buildOutput = cargo build --release 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        Write-Host $buildOutput
        exit 1
    }
    Write-Host "Build successful!" -ForegroundColor Green
} finally {
    Pop-Location
}

Write-Host ""

# Run parser tests
Write-Host "Running parser unit tests..." -ForegroundColor Yellow
Push-Location $ServerPath
try {
    $testOutput = cargo test codebase_parser --release -- --nocapture 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Tests failed!" -ForegroundColor Red
        Write-Host $testOutput
        exit 1
    }
    Write-Host $testOutput
    Write-Host "Tests passed!" -ForegroundColor Green
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Parser Test Complete!" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "1. Start the AMP server: cd amp/server && cargo run --release" -ForegroundColor White
Write-Host "2. Index the test repository via API:" -ForegroundColor White
Write-Host "   POST http://localhost:3000/v1/index" -ForegroundColor Gray
Write-Host "   Body: { `"path`": `"$TestRepoPath`" }" -ForegroundColor Gray
Write-Host "3. View indexed symbols in the UI: http://localhost:5173" -ForegroundColor White
Write-Host ""
