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
    Write-Header "PREMIX ORM: CI CHECK SUITE"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Checking Workspace..."
    cargo check

    Write-Step "Checking Premix Macros..."
    cargo check -p premix-macros

    Write-Step "Checking Premix Core..."
    cargo check -p premix-core --all-features

    Write-Step "Checking Benchmarks (Postgres)..."
    cargo check -p benchmarks --features postgres

    Write-Step "Running Unit Tests..."
    cargo test --workspace --exclude benchmarks

    $sw.Stop()
    Write-Header "ALL CHECKS PASSED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

