$ErrorActionPreference = "Stop"
$sw = [Diagnostics.Stopwatch]::StartNew()

param([switch]$Fix)

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

try {
    Write-Header "PREMIX ORM: CODE FORMATTER"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Running cargo fmt..."
    cargo fmt --all

    if ($Fix) {
        Write-Step "Running cargo clippy (auto-fix)..."
        cargo clippy --fix --workspace --allow-dirty --allow-staged
        Write-Success "Auto-fixes applied."
    }
    else {
        Write-Step "Running cargo clippy (check only)..."
        cargo clippy --workspace --all-features -- -W clippy::all
    }

    $sw.Stop()
    Write-Header "FORMAT COMPLETE in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n❌ FAILED: $_" -ForegroundColor Red
    exit 1
}

