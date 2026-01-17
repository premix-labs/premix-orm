# Run All Examples
Write-Host "Running All Examples..." -ForegroundColor Cyan

$ErrorActionPreference = "Continue"
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray

$examples = @(
    "basic-app",
    "relation-app",
    "eager-app",
    "transaction-app",
    "hooks-app",
    "optimistic-locking-app",
    "validation-app",
    "soft-delete-app",
    "bulk-ops-app",
    "tracing-app"
)

$passed = 0
$failed = 0

foreach ($example in $examples) {
    Write-Host "`n[$($passed + $failed + 1)/$($examples.Length)] Running $example..." -ForegroundColor Yellow
    
    $result = cargo run -p $example 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  [OK] $example passed" -ForegroundColor Green
        $passed++
    }
    else {
        Write-Host "  [FAIL] $example failed" -ForegroundColor Red
        Write-Host $result -ForegroundColor DarkGray
        $failed++
    }
}

Write-Host "`n=================================================="

if ($failed -eq 0) {
    Write-Host "[OK] All $passed examples passed!" -ForegroundColor Green
}
else {
    Write-Host "[WARN] $passed passed, $failed failed" -ForegroundColor Yellow
}
