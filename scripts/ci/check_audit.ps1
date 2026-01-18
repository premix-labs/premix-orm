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
    # We ignore:
    # 1. RUSTSEC-2023-0071: RSA Marvin attack (sqlx-mysql/rsa). No fix for rsa 0.9.x.
    # 2. RUSTSEC-2025-0134: rustls-pemfile unmaintained (benchmarks/rbatis).
    # 3. RUSTSEC-2026-0002: lru unsoundness (benchmarks/rbatis).
    cargo audit --ignore RUSTSEC-2023-0071 --ignore RUSTSEC-2025-0134 --ignore RUSTSEC-2026-0002
    if ($LASTEXITCODE -ne 0) { throw "Security vulnerabilities found." }

    $sw.Stop()
    Write-Host "[OK] Audit passed successfully." -ForegroundColor Green
    Write-Header "AUDIT PASSED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

