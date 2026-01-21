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

Write-Host "Starting Zero-Overhead Comparison Benchmark..." -ForegroundColor Cyan
Write-Host "Configuration:" -ForegroundColor Gray
Write-Host "  - CPU Core: $Core" -ForegroundColor Gray
Write-Host "  - Priority: $Priority" -ForegroundColor Gray
Write-Host "  - Warmup: ${Warmup}s" -ForegroundColor Gray
Write-Host "  - Measurement: ${Measurement}s" -ForegroundColor Gray

# Build argument list
$argList = @(
    "bench", "--manifest-path", "benchmarks/Cargo.toml",
    "--bench", "premix_vs_sqlx",
    "--features", "sqlite",
    "--",
    "--noplot",
    "--warm-up-time", $Warmup,
    "--measurement-time", $Measurement
)

# Start process
$process = Start-Process -FilePath "cargo" -ArgumentList $argList -NoNewWindow -PassThru -RedirectStandardOutput "benchmark_output.txt" -RedirectStandardError "benchmark_errors.txt"

# Apply Affinity & Priority
try {
    # Calculate affinity mask (1 << Core)
    $affinityMask = [IntPtr](1 -shl $Core)
    $process.ProcessorAffinity = $affinityMask
    $process.PriorityClass = $Priority
    Write-Host "Successfully pinned to Core $Core with $Priority priority." -ForegroundColor Green
}
catch {
    Write-Warning "Failed to set affinity/priority: $_"
}

$process.WaitForExit()

Write-Host "`nBenchmark completed. Summary (Time Results):" -ForegroundColor Green
Get-Content "benchmark_output.txt" | Select-String "time:" | ForEach-Object {
    Write-Host $_.Line.Trim()
}

Write-Host "`nFull output saved to: benchmark_output.txt" -ForegroundColor DarkGray
if (Test-Path "benchmark_errors.txt") {
    $errors = Get-Content "benchmark_errors.txt"
    if ($errors) {
        Write-Host "`nErrors/Warnings detected (First 5):" -ForegroundColor Yellow
        $errors | Select-Object -First 5
    }
}

