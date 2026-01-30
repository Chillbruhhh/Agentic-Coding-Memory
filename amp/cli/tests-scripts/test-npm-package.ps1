# Test AMP CLI npm package locally
# Run with: .\test-npm-package.ps1

Write-Host "🧪 Testing AMP CLI npm package..." -ForegroundColor Green

# Step 1: Create the package
Write-Host "`n📦 Step 1: Creating npm package..." -ForegroundColor Cyan
npm pack

$packageFile = Get-ChildItem -Filter "amp-protocol-cli-*.tgz" | Select-Object -First 1

if (-not $packageFile) {
    Write-Host "❌ Failed to create package" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Package created: $($packageFile.Name)" -ForegroundColor Green

# Step 2: Install globally
Write-Host "`n📥 Step 2: Installing package globally..." -ForegroundColor Cyan
npm install -g $packageFile.FullName

# Step 3: Test the command
Write-Host "`n🧪 Step 3: Testing amp command..." -ForegroundColor Cyan

Write-Host "`nTesting: amp --help" -ForegroundColor Yellow
amp --help

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ amp --help works!" -ForegroundColor Green
} else {
    Write-Host "❌ amp --help failed" -ForegroundColor Red
}

Write-Host "`nTesting: amp status" -ForegroundColor Yellow
amp status

# Step 4: Cleanup
Write-Host "`n🧹 Step 4: Cleanup..." -ForegroundColor Cyan
$response = Read-Host "Uninstall the package? (y/n)"

if ($response -eq 'y') {
    npm uninstall -g @amp-protocol/cli
    Remove-Item $packageFile.FullName
    Write-Host "✅ Cleanup complete" -ForegroundColor Green
} else {
    Write-Host "⚠️  Package still installed. Uninstall with: npm uninstall -g @amp-protocol/cli" -ForegroundColor Yellow
    Write-Host "⚠️  Package file: $($packageFile.FullName)" -ForegroundColor Yellow
}

Write-Host "`n✅ Test complete!" -ForegroundColor Green
