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

try {
    Write-Header "PREMIX ORM: CRATES.IO PUBLISHER"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    # 1. Pre-flight Checks
    Write-Step "Running Pre-publication Checks..."
    & "$ScriptRoot/../ci/check_all.ps1"
    if ($LASTEXITCODE -ne 0) { throw "Pre-check failed." }

    Write-Step "Checking Git Status..."
    $status = git status --porcelain
    if ($status) {
        throw "Uncommitted changes detected. Commit your code before publishing."
    }
    Write-Success "Git tree is clean."

    # 2. Dry Run
    Write-Step "Performing Dry Run..."
    Write-Host "   [premix-core]..." -ForegroundColor Gray
    cargo publish -p premix-core --dry-run
    
    Write-Host "   [premix-macros]..." -ForegroundColor Gray
    cargo publish -p premix-macros --dry-run

    Write-Host "   [premix-orm]..." -ForegroundColor Gray
    cargo publish -p premix-orm --dry-run

    Write-Host "   [premix-cli]..." -ForegroundColor Gray
    cargo publish -p premix-cli --dry-run
    Write-Success "Dry run completed successfully."

    # 3. Confirmation
    Write-Header "CONFIRM PUBLICATION"
    $version = (Select-String -Pattern 'version = "(.*)"' -Path premix-core/Cargo.toml | Select-Object -First 1 | ForEach-Object { $_.Matches.Groups[1].Value })
    $prompt = "Are you sure you want to publish v" + $version + "? (y/N)"
    $confirm = Read-Host $prompt
    if ($confirm -ne "y") {
        Write-Host "Cancelled." -ForegroundColor Yellow
        exit 0
    }

    # 4. Publish
    Write-Step "Publishing premix-core..."
    cargo publish -p premix-core
    
    Write-Step "Waiting 30s for Indexing..."
    Start-Sleep -Seconds 30
    
    Write-Step "Publishing premix-macros..."
    cargo publish -p premix-macros

    Write-Step "Waiting 30s for Indexing..."
    Start-Sleep -Seconds 30

    Write-Step "Publishing premix-orm..."
    cargo publish -p premix-orm

    Write-Step "Waiting 30s for Indexing..."
    Start-Sleep -Seconds 30

    Write-Step "Publishing premix-cli..."
    cargo publish -p premix-cli

    $sw.Stop()
    Write-Header "PUBLICATION SUCCESSFUL in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
    Write-Host "   https://crates.io/crates/premix-core"
    Write-Host "   https://crates.io/crates/premix-macros"
    Write-Host "   https://crates.io/crates/premix-orm"
    Write-Host "   https://crates.io/crates/premix-cli"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}
