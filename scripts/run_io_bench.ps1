# Run I/O Benchmark (Postgres Required)
Write-Host "Running I/O Benchmark (Postgres)..." -ForegroundColor Cyan

$ErrorActionPreference = "Stop"
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/.."
Write-Host "[DIR] Working Directory: $(Get-Location)" -ForegroundColor Gray

# Configuration
$PG_HOST = "localhost"
$PG_PORT = "5432"
$PG_USER = "postgres"
$PG_PASS = "admin123"
$PG_DB = "premix_bench"

# Set DATABASE_URL
if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://${PG_USER}:${PG_PASS}@${PG_HOST}:${PG_PORT}/${PG_DB}"
    Write-Host "[INFO] DATABASE_URL: $env:DATABASE_URL" -ForegroundColor Gray
}

# Try to create database if it doesn't exist
Write-Host "`nChecking if database '$PG_DB' exists..." -ForegroundColor Yellow
$env:PGPASSWORD = $PG_PASS

try {
    # Check if psql is available
    $null = Get-Command psql -ErrorAction Stop
    
    # Try to connect to the target database
    $null = psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d $PG_DB -c "SELECT 1;" 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[INFO] Database '$PG_DB' not found. Creating..." -ForegroundColor Yellow
        
        # Connect to default 'postgres' database to create new one
        psql -h $PG_HOST -p $PG_PORT -U $PG_USER -d postgres -c "CREATE DATABASE $PG_DB;"
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] Database '$PG_DB' created successfully!" -ForegroundColor Green
        }
        else {
            Write-Error "Failed to create database '$PG_DB'"
            exit 1
        }
    }
    else {
        Write-Host "[OK] Database '$PG_DB' exists." -ForegroundColor Green
    }
}
catch {
    Write-Host "[WARN] psql not found in PATH. Assuming database exists..." -ForegroundColor Yellow
    Write-Host "       If benchmark fails, manually create: CREATE DATABASE $PG_DB;" -ForegroundColor Gray
}

# Run benchmark
Write-Host "`nBuilding and running I/O benchmarks..." -ForegroundColor Yellow
cargo bench --bench io_benchmark --features postgres

if ($LASTEXITCODE -ne 0) { 
    Write-Error "I/O Benchmark failed! Check Postgres connection."
    exit 1 
}

Write-Host "`n[OK] I/O Benchmark Complete!" -ForegroundColor Green
