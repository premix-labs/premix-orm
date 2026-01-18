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

function Test-Command {
    param($Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Command '$Name' not found. Please run: cargo install $Name"
    }
}

try {
    Write-Header "PREMIX ORM: SECURITY AUDIT"
    Test-Command "cargo-audit"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Running cargo audit..."
    cargo audit

    $sw.Stop()
    Write-Header "AUDIT PASSED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

