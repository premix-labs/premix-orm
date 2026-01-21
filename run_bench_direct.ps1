#!/usr/bin/env powershell
cd "D:\code\RustroverProjects\Premix Labs\premix-orm"
Write-Host "Running benchmark directly..."
&cargo bench --manifest-path benchmarks/Cargo.toml --bench premix_vs_sqlx --features sqlite 2>&1 | Tee-Object -FilePath benchmark_direct.txt
