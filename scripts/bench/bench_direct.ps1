#!/usr/bin/env powershell
$ErrorActionPreference = "Stop"

$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/../.."

Write-Host "Running Zero-Overhead Benchmark Directly..." -ForegroundColor Cyan

&cargo bench --manifest-path benchmarks/Cargo.toml --bench premix_vs_sqlx --features sqlite 2>&1 | Tee-Object -FilePath benchmark_direct.txt

Write-Host "`nBenchmark finished. Results saved to benchmark_direct.txt" -ForegroundColor Green
