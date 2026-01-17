# Build AMP CLI for Windows distribution
# Run with: .\build-cli.ps1

param(
    [switch]$Package
)

Write-Host "🔨 Building AMP CLI for release..." -ForegroundColor Green

Set-Location "amp\cli"

# Build optimized release binary
cargo build --release

# Copy binary to project root for easy access
Copy-Item "target\release\amp.exe" "..\..\amp-cli.exe"

Write-Host "✅ AMP CLI built successfully!" -ForegroundColor Green
Write-Host "📦 Binary location: .\amp-cli.exe" -ForegroundColor Cyan
Write-Host "🚀 Run: .\amp-cli.exe --help" -ForegroundColor Cyan

# Optional: Create zip for distribution
if ($Package) {
    Set-Location "..\..\"
    $archName = "amp-cli-windows-$(if ([Environment]::Is64BitOperatingSystem) { 'x64' } else { 'x86' }).zip"
    Compress-Archive -Path "amp-cli.exe", "README.md", "CLI-USAGE.md" -DestinationPath $archName -Force
    Write-Host "📦 Package created: $archName" -ForegroundColor Green
}
