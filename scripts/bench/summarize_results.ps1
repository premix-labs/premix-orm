#!/usr/bin/env powershell
param (
    [string]$OrmPattern = "bench_orm_r*_output.txt",
    [string]$IoPattern = "bench_io_r*_output.txt",
    [string]$OutDir = "benchmarks/results",
    [string]$OutCsv = "benchmarks/results/summary.csv",
    [string]$OutJson = "benchmarks/results/summary.json",
    [string]$OutMarkdown = "benchmarks/results/summary.md"
)

$ErrorActionPreference = "Stop"

function Get-Median([double[]]$values) {
    if (-not $values -or $values.Count -eq 0) { return $null }
    $sorted = $values | Sort-Object
    $n = $sorted.Count
    if ($n % 2 -eq 1) {
        return $sorted[($n - 1) / 2]
    }
    return ($sorted[$n / 2 - 1] + $sorted[$n / 2]) / 2
}

function Parse-BenchFile {
    param([string]$Path)
    $results = @{}
    $lastLabel = $null
    foreach ($line in Get-Content $Path) {
        $trimmed = $line.Trim()
        if (-not $trimmed) { continue }
        if ($trimmed -match "time:") {
            $prefix = $trimmed.Split("time:")[0].Trim()
            $label = if ($prefix -like "*/*") { $prefix } else { $lastLabel }
            if (-not $label) { continue }
            $match = [regex]::Match($trimmed, "\[\s*([0-9.]+)\s*([^\d]+)\s+([0-9.]+)\s*([^\d]+)\s+([0-9.]+)\s*([^\d]+)")
            if (-not $match.Success) { continue }
            $median = [double]$match.Groups[3].Value
            $unit = $match.Groups[4].Value.Trim()
            $medianUs = if ($unit -match "ms") { $median * 1000 } else { $median }
            $results[$label] = $medianUs
        } elseif ($trimmed -like "*/*" -and $trimmed -notmatch "change:" -and $trimmed -notmatch "Benchmarking") {
            $lastLabel = $trimmed
        }
    }
    return $results
}

function Aggregate-Pattern {
    param([string]$Pattern)
    $files = Get-ChildItem -Path . -Filter $Pattern -File -ErrorAction SilentlyContinue
    $collected = @{}
    foreach ($file in $files) {
        $parsed = Parse-BenchFile -Path $file.FullName
        foreach ($entry in $parsed.GetEnumerator()) {
            if (-not $collected.ContainsKey($entry.Key)) {
                $collected[$entry.Key] = New-Object System.Collections.Generic.List[Double]
            }
            $collected[$entry.Key].Add([double]$entry.Value)
        }
    }
    $summary = @{}
    foreach ($entry in $collected.GetEnumerator()) {
        $median = Get-Median $entry.Value.ToArray()
        if ($null -ne $median) {
            $summary[$entry.Key] = $median
        }
    }
    return $summary
}

if (-not (Test-Path $OutDir)) {
    New-Item -ItemType Directory -Path $OutDir | Out-Null
}

$orm = Aggregate-Pattern -Pattern $OrmPattern
$io = Aggregate-Pattern -Pattern $IoPattern

$csvLines = @("group,label,median_us")
foreach ($entry in ($orm.GetEnumerator() | Sort-Object Key)) {
    $csvLines += "orm,$($entry.Key),$([math]::Round($entry.Value, 3))"
}
foreach ($entry in ($io.GetEnumerator() | Sort-Object Key)) {
    $csvLines += "io,$($entry.Key),$([math]::Round($entry.Value, 3))"
}
Set-Content -Path $OutCsv -Value $csvLines -Encoding ASCII

$json = @{
    orm = $orm
    io = $io
}
($json | ConvertTo-Json -Depth 5) | Set-Content -Path $OutJson -Encoding ASCII

$md = @()
$md += "# Benchmark Summary (Median of Medians)"
$md += ""
$md += "Source files:"
$md += "- ORM: $OrmPattern"
$md += "- IO: $IoPattern"
$md += ""
$md += "## ORM"
foreach ($entry in ($orm.GetEnumerator() | Sort-Object Key)) {
    $md += "- $($entry.Key): $([math]::Round($entry.Value, 3)) us"
}
$md += ""
$md += "## IO"
foreach ($entry in ($io.GetEnumerator() | Sort-Object Key)) {
    $md += "- $($entry.Key): $([math]::Round($entry.Value, 3)) us"
}
Set-Content -Path $OutMarkdown -Value $md -Encoding ASCII

Write-Host "Wrote summary:"
Write-Host "  CSV: $OutCsv"
Write-Host "  JSON: $OutJson"
Write-Host "  Markdown: $OutMarkdown"
