# Run ORM Comparison Benchmark (SQLite)
Write-Host "Running ORM Comparison Benchmark..." -ForegroundColor Cyan

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray

Write-Host "`nBuilding and running benchmarks..." -ForegroundColor Yellow
cargo bench --bench orm_comparison

if ($LASTEXITCODE -ne 0) { 
    Write-Error "Benchmark failed!"; exit 1 
}

Write-Host "`n[OK] Benchmark Complete!" -ForegroundColor Green
Write-Host "Results saved to: target/criterion/" -ForegroundColor Gray
