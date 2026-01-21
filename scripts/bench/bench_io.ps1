#!/usr/bin/env powershell
param (
    [int]$Core = 2,               # Pin to Core 2 (mask 4) by default
    [string]$Priority = "High",   # High priority class
    [int]$Warmup = 5,             # Warmup time in seconds
    [int]$Measurement = 10        # Measurement time in seconds
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

try {
    Write-Header "PREMIX ORM: POSTGRES I/O BENCHMARK (Pinned Core $Core)"
    Write-Host "Configuration:" -ForegroundColor Gray
    Write-Host "  - CPU Core: $Core" -ForegroundColor Gray
    Write-Host "  - Priority: $Priority" -ForegroundColor Gray

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

    # Build argument list for cargo bench
    $argList = @(
        "bench", 
        "--bench", "io_benchmark", 
        "--features", "postgres",
        "--", 
        "--noplot",
        "--warm-up-time", $Warmup, 
        "--measurement-time", $Measurement
    )

    # Start process with captured output path
    $process = Start-Process -FilePath "cargo" -ArgumentList $argList -NoNewWindow -PassThru -RedirectStandardOutput "bench_io_output.txt" -RedirectStandardError "bench_io_errors.txt"

    # Apply Affinity & Priority
    try {
        $affinityMask = [IntPtr](1 -shl $Core)
        $process.ProcessorAffinity = $affinityMask
        $process.PriorityClass = $Priority
        Write-Success "Pinned to Core $Core with $Priority priority."
    }
    catch {
        Write-Warning "Failed to set affinity/priority: $_"
    }

    $process.WaitForExit()

    if ($process.ExitCode -ne 0) { 
        throw "Bench failed with exit code $($process.ExitCode)" 
    }

    Write-Success "Benchmark Execution Successful."
    Write-Host "   View results in: target/criterion/report/index.html" -ForegroundColor DarkGray
    Write-Host "   Raw output saved to: bench_io_output.txt" -ForegroundColor DarkGray
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    if (Test-Path "bench_io_errors.txt") {
        Write-Host "Errors:" -ForegroundColor Red
        Get-Content "bench_io_errors.txt" | Select-Object -First 10
    }
    exit 1
}

