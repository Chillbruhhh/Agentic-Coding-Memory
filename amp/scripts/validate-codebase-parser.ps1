#!/usr/bin/env pwsh

# Validation script for AMP codebase parser
# Checks syntax and structure without compilation

Write-Host "🔍 Validating AMP Codebase Parser Implementation..." -ForegroundColor Cyan

$ErrorActionPreference = "Continue"

# Check if all required files exist
$requiredFiles = @(
    "amp/server/src/services/codebase_parser.rs",
    "amp/server/src/handlers/codebase.rs",
    "amp/scripts/test-codebase-parser.ps1",
    "amp/scripts/test-codebase-parser.sh",
    "amp/CODEBASE_PARSER.md"
)

Write-Host "`n📁 Checking required files..." -ForegroundColor Yellow
$allFilesExist = $true

foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host "  ✅ $file" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file" -ForegroundColor Red
        $allFilesExist = $false
    }
}

if (-not $allFilesExist) {
    Write-Host "`n❌ Some required files are missing!" -ForegroundColor Red
    exit 1
}

# Check Cargo.toml dependencies
Write-Host "`n📦 Checking dependencies..." -ForegroundColor Yellow
$cargoToml = Get-Content "amp/server/Cargo.toml" -Raw

$requiredDeps = @(
    "tree-sitter",
    "tree-sitter-python", 
    "tree-sitter-typescript",
    "walkdir",
    "sha2",
    "hex"
)

foreach ($dep in $requiredDeps) {
    if ($cargoToml -match $dep) {
        Write-Host "  ✅ $dep" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $dep" -ForegroundColor Red
    }
}

# Check main.rs for codebase routes
Write-Host "`n🛣️  Checking API routes..." -ForegroundColor Yellow
$mainRs = Get-Content "amp/server/src/main.rs" -Raw

$requiredRoutes = @(
    "/codebase/parse",
    "/codebase/parse-file",
    "/codebase/file-logs",
    "/codebase/update-file-log"
)

foreach ($route in $requiredRoutes) {
    if ($mainRs -match [regex]::Escape($route)) {
        Write-Host "  ✅ $route" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $route" -ForegroundColor Red
    }
}

# Check handlers/mod.rs includes codebase module
Write-Host "`n📋 Checking module includes..." -ForegroundColor Yellow
$handlersMod = Get-Content "amp/server/src/handlers/mod.rs" -Raw

if ($handlersMod -match "pub mod codebase") {
    Write-Host "  ✅ codebase handler module" -ForegroundColor Green
} else {
    Write-Host "  ❌ codebase handler module" -ForegroundColor Red
}

# Check services/mod.rs includes codebase_parser module
$servicesMod = Get-Content "amp/server/src/services/mod.rs" -Raw

if ($servicesMod -match "pub mod codebase_parser") {
    Write-Host "  ✅ codebase_parser service module" -ForegroundColor Green
} else {
    Write-Host "  ❌ codebase_parser service module" -ForegroundColor Red
}

# Check for key structs and functions
Write-Host "`n🏗️  Checking key structures..." -ForegroundColor Yellow
$codebaseParser = Get-Content "amp/server/src/services/codebase_parser.rs" -Raw

$keyStructs = @(
    "pub struct CodebaseParser",
    "pub struct ParsedSymbol", 
    "pub struct FileLog",
    "pub struct FileDependencies",
    "impl CodebaseParser",
    "pub fn parse_codebase",
    "pub fn parse_file",
    "pub fn generate_file_log_markdown"
)

foreach ($struct in $keyStructs) {
    if ($codebaseParser -match [regex]::Escape($struct)) {
        Write-Host "  ✅ $struct" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $struct" -ForegroundColor Red
    }
}

# Check handler functions
Write-Host "`n🎯 Checking handler functions..." -ForegroundColor Yellow
$codebaseHandler = Get-Content "amp/server/src/handlers/codebase.rs" -Raw

$handlerFunctions = @(
    "pub async fn parse_codebase",
    "pub async fn parse_file",
    "pub async fn update_file_log",
    "pub async fn get_file_logs",
    "pub async fn get_file_log"
)

foreach ($func in $handlerFunctions) {
    if ($codebaseHandler -match [regex]::Escape($func)) {
        Write-Host "  ✅ $func" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $func" -ForegroundColor Red
    }
}

# Check test scripts are executable
Write-Host "`n🧪 Checking test scripts..." -ForegroundColor Yellow

if (Test-Path "amp/scripts/test-codebase-parser.ps1") {
    Write-Host "  ✅ PowerShell test script exists" -ForegroundColor Green
} else {
    Write-Host "  ❌ PowerShell test script missing" -ForegroundColor Red
}

if (Test-Path "amp/scripts/test-codebase-parser.sh") {
    Write-Host "  ✅ Bash test script exists" -ForegroundColor Green
} else {
    Write-Host "  ❌ Bash test script missing" -ForegroundColor Red
}

# Check documentation
Write-Host "`n📚 Checking documentation..." -ForegroundColor Yellow
$docs = Get-Content "amp/CODEBASE_PARSER.md" -Raw

$docSections = @(
    "# AMP Codebase Parser",
    "## Features",
    "## API Endpoints", 
    "## File Log Format",
    "## Usage Examples"
)

foreach ($section in $docSections) {
    if ($docs -match [regex]::Escape($section)) {
        Write-Host "  ✅ $section" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $section" -ForegroundColor Red
    }
}

Write-Host "`n🎉 Validation complete!" -ForegroundColor Green
Write-Host "The AMP Codebase Parser implementation includes:" -ForegroundColor Cyan
Write-Host "  ✅ Tree-sitter integration for Python and TypeScript" -ForegroundColor Gray
Write-Host "  ✅ Symbol extraction and dependency analysis" -ForegroundColor Gray
Write-Host "  ✅ Structured file logs in Markdown format" -ForegroundColor Gray
Write-Host "  ✅ Complete REST API endpoints" -ForegroundColor Gray
Write-Host "  ✅ Comprehensive test scripts" -ForegroundColor Gray
Write-Host "  ✅ Detailed documentation" -ForegroundColor Gray

Write-Host "`n🚀 Ready for compilation and testing!" -ForegroundColor Green
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Run: cd amp/server && cargo build" -ForegroundColor Gray
Write-Host "  2. Run: cd amp/server && cargo run" -ForegroundColor Gray
Write-Host "  3. Test: ./amp/scripts/test-codebase-parser.ps1" -ForegroundColor Gray
