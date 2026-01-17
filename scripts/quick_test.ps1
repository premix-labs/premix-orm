# Quick Test - Build and run basic-app
Write-Host "Quick Test..." -ForegroundColor Cyan

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray

Write-Host "`nBuilding workspace..." -ForegroundColor Yellow
cargo build --workspace

if ($LASTEXITCODE -ne 0) { 
    Write-Error "Build failed!"; exit 1 
}

Write-Host "`nRunning basic-app..." -ForegroundColor Yellow
cargo run -p basic-app

if ($LASTEXITCODE -ne 0) { 
    Write-Error "basic-app failed!"; exit 1 
}

Write-Host "`n[OK] Quick Test Passed!" -ForegroundColor Green
