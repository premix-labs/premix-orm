#!/usr/bin/env powershell
param (
    [int]$Core = 2,               # Pin to Core 2 (mask 4) by default
    [string]$Priority = "High",   # High priority class
    [int]$Warmup = 5,             # Warmup time in seconds
    [int]$Measurement = 10,       # Measurement time in seconds
    [int]$TimeoutSec = 900,       # Hard timeout for the bench process
    [string]$OutPrefix = "bench_io"
)

$ErrorActionPreference = "Stop"

$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/../.."

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

function Test-Psql {
    if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
        Write-Host "[WARN] psql not found. Cannot verify database existence automatically." -ForegroundColor DarkGray
        return $false
    }
    return $true
}

function Read-Head {
    param($Path, [int]$Lines = 10)
    if (Test-Path $Path) {
        Get-Content $Path | Select-Object -First $Lines
    }
}

function Read-Tail {
    param($Path, [int]$Lines = 20)
    if (Test-Path $Path) {
        Get-Content $Path | Select-Object -Last $Lines
    }
}

try {
    Write-Header "PREMIX ORM: POSTGRES I/O BENCHMARK (Pinned Core $Core)"
    Write-Host "Configuration:" -ForegroundColor Gray
    Write-Host "  - CPU Core: $Core" -ForegroundColor Gray
    Write-Host "  - Priority: $Priority" -ForegroundColor Gray
    Write-Host "  - Timeout: ${TimeoutSec}s" -ForegroundColor Gray

    # Configuration
    $PG_HOST = "localhost"
    $PG_PORT = "5432"
    $PG_USER = "postgres"
    $PG_PASS = "admin123"
    $PG_DB = "premix_bench"

    # Set Environment
    if (-not $env:DATABASE_URL) {
        $env:DATABASE_URL = "postgres://${PG_USER}:${PG_PASS}@${PG_HOST}:${PG_PORT}/${PG_DB}"
        Write-Step "Configured Environment"
        Write-Host "   DATABASE_URL: $env:DATABASE_URL" -ForegroundColor Gray
    }

    # DB Check
    if (Test-Psql) {
        Write-Step "Checking Postgres Database..."
        $env:PGPASSWORD = $PG_PASS
        $null = psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB -c "SELECT 1;" 2>&1

        if ($LASTEXITCODE -ne 0) {
            Write-Host "   Database '$PG_DB' not found. Creating..." -ForegroundColor DarkGray
            psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d postgres -c "CREATE DATABASE $PG_DB;"
            if ($LASTEXITCODE -eq 0) { Write-Success "Database created." }
            else { throw "Failed to create database." }
        }
        else {
            Write-Success "Database '$PG_DB' exists."
        }
    }

    Write-Step "Building & Running 'io_benchmark'..."

    $argList = @(
        "bench",
        "--bench", "io_benchmark",
        "--features", "postgres",
        "--",
        "--noplot",
        "--warm-up-time", $Warmup,
        "--measurement-time", $Measurement
    )

    $outFile = "${OutPrefix}_output.txt"
    $errFile = "${OutPrefix}_errors.txt"
    $process = Start-Process -FilePath "cargo" -ArgumentList $argList -NoNewWindow -PassThru -RedirectStandardOutput $outFile -RedirectStandardError $errFile

    try {
        $affinityMask = [IntPtr](1 -shl $Core)
        $process.ProcessorAffinity = $affinityMask
        $process.PriorityClass = $Priority
        Write-Success "Pinned to Core $Core with $Priority priority."
    }
    catch {
        Write-Warning "Failed to set affinity/priority: $_"
    }

    if (-not $process.WaitForExit($TimeoutSec * 1000)) {
        try { $process.Kill() } catch {}
        throw "Bench timed out after ${TimeoutSec}s"
    }

    if ($null -eq $process.ExitCode) {
        Write-Warning "Bench terminated without exit code"
    } elseif ($process.ExitCode -ne 0) {
        throw "Bench failed with exit code $($process.ExitCode)"
    }

    Write-Success "Benchmark Execution Successful."
    Write-Host "   View results in: target/criterion/report/index.html" -ForegroundColor DarkGray
    Write-Host "   Raw output saved to: $outFile" -ForegroundColor DarkGray
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    if (Test-Path $errFile) {
        Write-Host "Errors:" -ForegroundColor Red
        Read-Head $errFile 10
    }
    if (Test-Path $outFile) {
        Write-Host "Output (tail):" -ForegroundColor DarkGray
        Read-Tail $outFile 20
    }
    exit 1
}
