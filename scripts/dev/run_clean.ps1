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

try {
    Write-Header "PREMIX ORM: CLEANUP UTILITY"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Running cargo clean..."
    cargo clean

    Write-Step "Removing Database Artifacts..."
    Get-ChildItem -Path . -Filter "*.db" -Recurse | Remove-Item -Force -ErrorAction SilentlyContinue
    Get-ChildItem -Path . -Filter "*.sqlite" -Recurse | Remove-Item -Force -ErrorAction SilentlyContinue
    Write-Success "Allocated DB files removed."

    $removeBenchmarks = Read-Host "`nRemove benchmark results? (y/N)"
    if ($removeBenchmarks -eq "y") {
        Write-Step "Removing Benchmark Results..."
        Remove-Item -Path "target/criterion" -Recurse -Force -ErrorAction SilentlyContinue
        Write-Success "Criterion results cleaned."
    }

    $sw.Stop()
    Write-Header "CLEANUP COMPLETE in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n❌ FAILED: $_" -ForegroundColor Red
    exit 1
}

