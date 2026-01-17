Write-Host " Running Premix ORM Validation Suite..." -ForegroundColor Cyan

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot
# Navigate to the project root (parent of 'scripts' dir)
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray


Write-Host "`n1. Checking Workspace..." -ForegroundColor Yellow
cargo check
if ($LASTEXITCODE -ne 0) { Write-Error "Workspace check failed!"; exit 1 }

Write-Host "`n2. Checking Premix Macros..." -ForegroundColor Yellow
cargo check -p premix-macros
if ($LASTEXITCODE -ne 0) { Write-Error "Macros check failed!"; exit 1 }

Write-Host "`n3. Checking Premix Core..." -ForegroundColor Yellow
cargo check -p premix-core --all-features
if ($LASTEXITCODE -ne 0) { Write-Error "Core check failed!"; exit 1 }

Write-Host "`n4. Checking Benchmarks (Postgres)..." -ForegroundColor Yellow
cargo check -p benchmarks --features postgres
if ($LASTEXITCODE -ne 0) { Write-Error "Benchmarks check failed!"; exit 1 }

Write-Host "`n5. Running Unit Tests (excluding benchmarks)..." -ForegroundColor Yellow
# Exclude benchmarks from generic test run as they might require specific DB setup usually
cargo test --workspace --exclude benchmarks
if ($LASTEXITCODE -ne 0) { Write-Error "Unit tests failed!"; exit 1 }

Write-Host "`n All Checks Passed!" -ForegroundColor Green
