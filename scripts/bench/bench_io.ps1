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

function Test-Psql {
    if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
        Write-Host "[WARN] psql not found. Cannot verify database existence automatically." -ForegroundColor DarkGray
        return $false
    }
    return $true
}

try {
    Write-Header "PREMIX ORM: POSTGRES I/O BENCHMARK"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

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

    Write-Step "Building & Running Benchmarks..."
    cargo bench --bench io_benchmark --features postgres

    $sw.Stop()
    Write-Header "BENCHMARK SUITE COMPLETE in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

