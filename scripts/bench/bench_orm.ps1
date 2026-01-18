$ErrorActionPreference = "Stop"
$sw = [Diagnostics.Stopwatch]::StartNew()

function Write-Header {
    param($Text)
    Write-Host "`n========================================" -ForegroundColor DarkCyan
    Write-Host "   $Text" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor DarkCyan
}

function Write-Step {
    param($Text)
    Write-Host "`n>> $Text" -ForegroundColor Yellow
}

function Write-Success {
    param($Text)
    Write-Host "[OK] $Text" -ForegroundColor Green
}

try {
    Write-Header "PREMIX ORM: SQLITE BENCHMARKS"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Building & Running 'orm_comparison'..."
    cargo bench --bench orm_comparison
    if ($LASTEXITCODE -ne 0) { throw "Bench failed with exit code $LASTEXITCODE" }

    Write-Success "Benchmark Execution Successful."
    Write-Host "   View results in: target/criterion/report/index.html" -ForegroundColor DarkGray

    $sw.Stop()
    Write-Header "BENCHMARK SUITE COMPLETE in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

