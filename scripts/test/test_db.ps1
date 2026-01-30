$ErrorActionPreference = "Stop"
$sw = [Diagnostics.Stopwatch]::StartNew()

param(
    [string]$PostgresUrl,
    [string]$MysqlUrl
)

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

function Resolve-EnvUrl {
    param($Value, $EnvKey)
    if ($Value) { return $Value }
    if ($env:$EnvKey) { return $env:$EnvKey }
    return ""
}

try {
    Write-Header "PREMIX ORM: DB INTEGRATION TESTS"
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "SQLite integration tests (default features)"
    cargo test -p premix-orm --tests
    Write-Success "SQLite tests passed"

    $pgUrl = Resolve-EnvUrl -Value $PostgresUrl -EnvKey "PREMIX_POSTGRES_URL"
    if (-not $pgUrl) { $pgUrl = Resolve-EnvUrl -Value "" -EnvKey "DATABASE_URL" }
    if ($pgUrl -and ($pgUrl -like "postgres*")) {
        Write-Step "Postgres integration tests"
        $oldDb = $env:DATABASE_URL
        $env:DATABASE_URL = $pgUrl
        cargo test -p premix-orm --tests --features postgres
        cargo test -p premix-core --features postgres
        if ($null -ne $oldDb) { $env:DATABASE_URL = $oldDb } else { Remove-Item Env:\DATABASE_URL }
        Write-Success "Postgres tests passed"
    }
    else {
        Write-Host "[SKIP] Postgres tests (set PREMIX_POSTGRES_URL or DATABASE_URL)" -ForegroundColor DarkYellow
    }

    $myUrl = Resolve-EnvUrl -Value $MysqlUrl -EnvKey "PREMIX_MYSQL_URL"
    if (-not $myUrl) { $myUrl = Resolve-EnvUrl -Value "" -EnvKey "DATABASE_URL" }
    if ($myUrl -and ($myUrl -like "mysql*")) {
        Write-Step "MySQL integration tests"
        $oldDb = $env:DATABASE_URL
        $env:DATABASE_URL = $myUrl
        cargo test -p premix-orm --tests --features mysql
        if ($null -ne $oldDb) { $env:DATABASE_URL = $oldDb } else { Remove-Item Env:\DATABASE_URL }
        Write-Success "MySQL tests passed"
    }
    else {
        Write-Host "[SKIP] MySQL tests (set PREMIX_MYSQL_URL or DATABASE_URL)" -ForegroundColor DarkYellow
    }

    $sw.Stop()
    Write-Header "DB TESTS COMPLETE in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}
