#!/usr/bin/env powershell
$ErrorActionPreference = "Stop"

$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/../.."

Write-Host "Starting Zero-Overhead Comparison Benchmark..." -ForegroundColor Cyan

$process = Start-Process -FilePath "cargo" -ArgumentList "bench", "--manifest-path", "benchmarks/Cargo.toml", "--bench", "premix_vs_sqlx", "--features", "sqlite" -NoNewWindow -PassThru -RedirectStandardOutput "benchmark_output.txt" -RedirectStandardError "benchmark_errors.txt"
$process.WaitForExit()

Write-Host "Benchmark completed. Summary (Head 20):" -ForegroundColor Green
Get-Content "benchmark_output.txt" -Head 20

Write-Host "`nFull output saved to: benchmark_output.txt" -ForegroundColor DarkGray
if (Test-Path "benchmark_errors.txt") {
    $errors = Get-Content "benchmark_errors.txt"
    if ($errors) {
        Write-Host "`nErrors detected:" -ForegroundColor Red
        $errors | Select-Object -First 10
    }
}
