$ErrorActionPreference = "Stop"
$sw = [Diagnostics.Stopwatch]::StartNew()

param(
    [switch]$RunTutorial
)

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
    Write-Header "PREMIX ORM: E2E SMOKE RUNS"
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "basic-app"
    cargo run -p basic-app
    Write-Success "basic-app OK"

    Write-Step "tracing-app"
    cargo run -p tracing-app
    Write-Success "tracing-app OK"

    if ($RunTutorial) {
        Write-Step "premix-axum-tutorial"
        cargo run -p premix-axum-tutorial
        Write-Success "premix-axum-tutorial OK"
    }
    else {
        Write-Host "[SKIP] premix-axum-tutorial (use -RunTutorial)" -ForegroundColor DarkYellow
    }

    $sw.Stop()
    Write-Header "E2E RUNS COMPLETE in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}
