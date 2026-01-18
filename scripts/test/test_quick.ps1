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
    Write-Header "PREMIX ORM: QUICK TEST SUITE"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Building Workspace..."
    cargo build --workspace

    Write-Step "Running basic-app smoke test..."
    cargo run -p basic-app

    $sw.Stop()
    Write-Header "QUICK TEST PASSED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

