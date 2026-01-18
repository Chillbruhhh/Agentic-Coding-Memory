# AMP CLI Installation Script for Windows
# Run with: .\install.ps1

Write-Host "🚀 Installing AMP CLI..." -ForegroundColor Green

# Check if Rust is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust/Cargo not found. Please install Rust first:" -ForegroundColor Red
    Write-Host "   Visit: https://rustup.rs/" -ForegroundColor Yellow
    Write-Host "   Or run: winget install Rustlang.Rustup" -ForegroundColor Yellow
    exit 1
}

# Navigate to repository root if we're in scripts directory
$currentDir = Get-Location
if ($currentDir.Path -like "*\scripts") {
    Set-Location ".."
}

# Build and install the CLI
Set-Location "amp\cli"
cargo install --path . --force

# Return to original directory
Set-Location $currentDir

Write-Host "✅ AMP CLI installed successfully!" -ForegroundColor Green
Write-Host "📋 Usage: amp --help" -ForegroundColor Cyan
Write-Host "🎯 Start a session: amp start 'kiro-cli'" -ForegroundColor Cyan
Write-Host "📊 Check status: amp status" -ForegroundColor Cyan
Write-Host "🖥️  Launch TUI: amp tui" -ForegroundColor Cyan
