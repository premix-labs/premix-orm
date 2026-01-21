#!/usr/bin/env powershell
cd "D:\code\RustroverProjects\Premix Labs\premix-orm"
Write-Host "Starting benchmark run..."
$process = Start-Process -FilePath "cargo" -ArgumentList "bench", "--manifest-path", "benchmarks/Cargo.toml", "--bench", "premix_vs_sqlx", "--features", "sqlite" -NoNewWindow -PassThru -RedirectStandardOutput "benchmark_output.txt" -RedirectStandardError "benchmark_errors.txt"
$process.WaitForExit()
Write-Host "Benchmark completed. Output:"
Get-Content "benchmark_output.txt" -Head 200
Write-Host "`nErrors (if any):"
Get-Content "benchmark_errors.txt" -Head 50
