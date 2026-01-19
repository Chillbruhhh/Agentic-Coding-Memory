# Build AMP MCP Server
# Usage: .\build-mcp-server.ps1

$ErrorActionPreference = "Stop"

Write-Host "Building AMP MCP Server..." -ForegroundColor Green

$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$mcpServerPath = Join-Path $scriptPath "..\amp\mcp-server"

Set-Location $mcpServerPath

# Check if cargo is available
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: cargo not found. Please install Rust: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Build release binary
Write-Host "Building release binary..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Build complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Binary location: $(Get-Location)\target\release\amp-mcp-server.exe"
    Write-Host ""
    Write-Host "To run:"
    Write-Host "  cd amp\mcp-server"
    Write-Host "  .\target\release\amp-mcp-server.exe"
    Write-Host ""
    Write-Host "Or install globally:"
    Write-Host "  cargo install --path ."
} else {
    Write-Host "✗ Build failed!" -ForegroundColor Red
    exit 1
}
