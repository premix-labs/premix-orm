param(
    [string]$Features = ""
)
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
    Write-Header "PREMIX ORM: CODE COVERAGE"
    Test-Command "cargo-tarpaulin"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Running Tarpaulin..."
    # Note: Doctests are currently disabled due to a known rustdoc issue on Windows with Tarpaulin.
    $FeatureArgs = @()
    if ($Features) {
        $FeatureArgs = @("--features", $Features)
    }
    cargo tarpaulin --out Html --output-dir coverage --all-targets --exclude-files benchmarks/* --exclude-files examples/* --exclude-files premix-macros/src/lib.rs @FeatureArgs

    $sw.Stop()
    Write-Header "COVERAGE REPORT GENERATED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
    Write-Host "   file:///$($pwd.Path)/coverage/tarpaulin-report.html" -ForegroundColor DarkGray
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

