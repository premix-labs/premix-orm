# Format and Lint All Code
# Usage:
#   .\fmt.ps1        - Format + check clippy (warnings only)
#   .\fmt.ps1 -Fix   - Format + auto-fix clippy warnings

param(
    [switch]$Fix
)

Write-Host "Formatting All Code..." -ForegroundColor Cyan

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray

# Step 1: Format
Write-Host "`nRunning cargo fmt..." -ForegroundColor Yellow
cargo fmt --all

if ($LASTEXITCODE -ne 0) { 
    Write-Error "Format failed!"; exit 1 
}

# Step 2: Clippy
if ($Fix) {
    Write-Host "`nRunning cargo clippy --fix (auto-fixing)..." -ForegroundColor Yellow
    cargo clippy --fix --workspace --allow-dirty --allow-staged
    
    if ($LASTEXITCODE -ne 0) { 
        Write-Error "Clippy fix failed!"; exit 1 
    }
    Write-Host "[OK] Auto-fix applied!" -ForegroundColor Green
}
else {
    Write-Host "`nRunning cargo clippy (check only)..." -ForegroundColor Yellow
    cargo clippy --workspace --all-features -- -W clippy::all
}

Write-Host "`n[OK] Format Complete!" -ForegroundColor Green
