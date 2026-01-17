# Publish to Crates.io
Write-Host "Publishing Premix ORM to Crates.io..." -ForegroundColor Cyan

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray

# Pre-flight checks
Write-Host "`n1. Running pre-publish checks..." -ForegroundColor Yellow
& "$ScriptRoot/check_all.ps1"
if ($LASTEXITCODE -ne 0) { 
    Write-Error "Pre-publish checks failed!"; exit 1 
}

Write-Host "`n2. Checking for uncommitted changes..." -ForegroundColor Yellow
$status = git status --porcelain
if ($status) {
    Write-Error "Uncommitted changes detected. Please commit first."
    git status
    exit 1
}

# Dry run first
Write-Host "`n3. Dry run publish..." -ForegroundColor Yellow
Write-Host "`n   [premix-core]" -ForegroundColor Gray
cargo publish -p premix-core --dry-run
if ($LASTEXITCODE -ne 0) { Write-Error "premix-core dry-run failed!"; exit 1 }

Write-Host "`n   [premix-macros]" -ForegroundColor Gray
cargo publish -p premix-macros --dry-run
if ($LASTEXITCODE -ne 0) { Write-Error "premix-macros dry-run failed!"; exit 1 }

# Confirmation
Write-Host "`n==================================================" -ForegroundColor Yellow
$confirm = Read-Host "Ready to publish! Continue? (y/N)"
if ($confirm -ne "y") {
    Write-Host "Cancelled." -ForegroundColor Yellow
    exit 0
}

# Actual publish (order matters: core first, then macros)
Write-Host "`n4. Publishing premix-core..." -ForegroundColor Yellow
cargo publish -p premix-core
if ($LASTEXITCODE -ne 0) { Write-Error "premix-core publish failed!"; exit 1 }

Write-Host "`nWaiting 30s for crates.io to index premix-core..." -ForegroundColor Gray
Start-Sleep -Seconds 30

Write-Host "`n5. Publishing premix-macros..." -ForegroundColor Yellow
cargo publish -p premix-macros
if ($LASTEXITCODE -ne 0) { Write-Error "premix-macros publish failed!"; exit 1 }

Write-Host "`n[OK] Published Successfully!" -ForegroundColor Green
Write-Host "  - https://crates.io/crates/premix-core"
Write-Host "  - https://crates.io/crates/premix-macros"
