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

try {
    Write-Header "PREMIX ORM: SQLITE BENCHMARKS (Pinned Core $Core)"
    Write-Host "Configuration:" -ForegroundColor Gray
    Write-Host "  - CPU Core: $Core" -ForegroundColor Gray
    Write-Host "  - Priority: $Priority" -ForegroundColor Gray
    
    Write-Step "Building & Running 'orm_comparison'..."
    
    # Build argument list for cargo bench
    $argList = @(
        "bench", 
        "--bench", "orm_comparison", 
        "--", 
        "--noplot",
        "--warm-up-time", $Warmup, 
        "--measurement-time", $Measurement
    )

    # Start process with captured output path
    $process = Start-Process -FilePath "cargo" -ArgumentList $argList -NoNewWindow -PassThru -RedirectStandardOutput "bench_orm_output.txt" -RedirectStandardError "bench_orm_errors.txt"

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
    Write-Host "   Raw output saved to: bench_orm_output.txt" -ForegroundColor DarkGray
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    if (Test-Path "bench_orm_errors.txt") {
        Write-Host "Errors:" -ForegroundColor Red
        Get-Content "bench_orm_errors.txt" | Select-Object -First 10
    }
    exit 1
}

