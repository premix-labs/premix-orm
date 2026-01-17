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
    Write-Host "`n➜ $Text" -ForegroundColor Yellow
}

function Write-Success {
    param($Text)
    Write-Host "✅ $Text" -ForegroundColor Green
}

function Test-Command {
    param($Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Command '$Name' not found. Please run: cargo install $Name"
    }
}

try {
    Write-Header "PREMIX ORM: CODE COVERAGE"
    Test-Command "cargo-tarpaulin"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Running Tarpaulin..."
    cargo tarpaulin --out Html --output-dir coverage --exclude-files benchmarks/* --exclude-files examples/*

    $sw.Stop()
    Write-Header "COVERAGE REPORT GENERATED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
    Write-Host "   file:///$($pwd.Path)/coverage/tarpaulin-report.html" -ForegroundColor DarkGray
}
catch {
    Write-Host "`n❌ FAILED: $_" -ForegroundColor Red
    exit 1
}

