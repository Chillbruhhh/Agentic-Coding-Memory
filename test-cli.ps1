# Quick test of AMP CLI without installation
# Run with: .\test-cli.ps1

Write-Host "🧪 Testing AMP CLI..." -ForegroundColor Green

Set-Location "amp\cli"

Write-Host "📋 Running: cargo run -- --help" -ForegroundColor Cyan
cargo run -- --help

Write-Host "`n🎯 To test with commands:" -ForegroundColor Yellow
Write-Host "   cargo run -- start 'kiro-cli'" -ForegroundColor Cyan
Write-Host "   cargo run -- status" -ForegroundColor Cyan
Write-Host "   cargo run -- tui" -ForegroundColor Cyan
