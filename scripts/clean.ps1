# Clean All Build Artifacts
Write-Host "Cleaning All Build Artifacts..." -ForegroundColor Cyan

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray

Write-Host "`nRunning cargo clean..." -ForegroundColor Yellow
cargo clean

# Remove SQLite database files
Write-Host "`nRemoving SQLite database files..." -ForegroundColor Yellow
Get-ChildItem -Path . -Filter "*.db" -Recurse | Remove-Item -Force -ErrorAction SilentlyContinue
Get-ChildItem -Path . -Filter "*.sqlite" -Recurse | Remove-Item -Force -ErrorAction SilentlyContinue

# Remove Criterion benchmark results (optional)
$removeBenchmarks = Read-Host "Remove benchmark results? (y/N)"
if ($removeBenchmarks -eq "y") {
    Write-Host "Removing benchmark results..." -ForegroundColor Yellow
    Remove-Item -Path "target/criterion" -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "`n[OK] Clean Complete!" -ForegroundColor Green
