param(
    [int]$Rounds = 3
)

$ErrorActionPreference = "Stop"

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

function Collect-Estimates {
    param(
        [string]$Root,
        [int]$Round
    )
    $rows = @()
    $items = Get-ChildItem $Root -Recurse -Filter estimates.json |
        Where-Object { $_.FullName -match "\\\\base\\\\estimates\\.json$" }
    foreach ($item in $items) {
        $bench = Split-Path -Leaf (Split-Path -Parent (Split-Path -Parent $item.FullName))
        $group = Split-Path -Leaf (Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $item.FullName)))
        $json = Get-Content $item.FullName -Raw | ConvertFrom-Json
        $rows += [pscustomobject]@{
            Round = $Round
            Group = $group
            Bench = $bench
            MedianNs = [double]$json.median.point_estimate
        }
    }
    return $rows
}

Write-Header "PREMIX ORM: REPEATABLE BENCHMARK RUNNER"

$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/../.."

$resultsRoot = "benchmarks/results"
New-Item -ItemType Directory -Path $resultsRoot -Force | Out-Null
$summaryPath = Join-Path $resultsRoot "summary.csv"

$allRows = @()

for ($i = 1; $i -le $Rounds; $i++) {
    Write-Step "Round $i/${Rounds}: SQLite benchmarks"
    . "scripts/bench/bench_orm.ps1"

    Write-Step "Round $i/${Rounds}: Postgres benchmarks"
    . "scripts/bench/bench_io.ps1"

    $criterionRoot = "target/criterion"
    $rows = Collect-Estimates -Root $criterionRoot -Round $i
    $allRows += $rows

    $stamp = (Get-Date).ToString("yyyyMMdd_HHmmss")
    $snapshot = Join-Path $resultsRoot "criterion_round_${i}_$stamp"
    Copy-Item -Recurse -Force $criterionRoot $snapshot
    Write-Success "Saved Criterion snapshot: $snapshot"
}

$allRows | Sort-Object Round, Group, Bench | Export-Csv -NoTypeInformation -Path $summaryPath
Write-Success "Wrote summary: $summaryPath"

Write-Header "REPEATABLE BENCHMARKS COMPLETE"
