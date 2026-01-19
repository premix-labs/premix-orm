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

function Test-Command {
    param($Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Host "[ERROR] Error: '$Name' is not installed." -ForegroundColor Red
        exit 1
    }
}

function Get-DatabaseUrl {
    if ($env:POSTGRES_URL) {
        return $env:POSTGRES_URL
    }
    return "sqlite:premix.db"
}

try {
    Write-Header "PREMIX ORM: MIGRATION VALIDATION"
    Test-Command "cargo"

    # Navigate to Root
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    Write-Step "Building Premix CLI..."
    cargo build -p premix-cli --quiet

    Write-Step "Cleaning up old artifacts..."
    if (Test-Path "migrations") { Remove-Item "migrations" -Recurse -Force }
    if (Test-Path "premix.db") { Remove-Item "premix.db" -Force }

    Write-Step "Creating a Migration..."
    $output = cargo run -p premix-cli --quiet -- migrate create create_users
    Write-Host $output -ForegroundColor DarkGray

    $migrationFile = Get-ChildItem "migrations/*.sql" | Select-Object -First 1
    if (-not $migrationFile) { throw "No migration file created." }
    Write-Host "   -> File: $($migrationFile.Name)" -ForegroundColor Gray

    Write-Step "Writing SQL Content..."
    $sqlContent = @"
-- up
CREATE TABLE premix_migration_test_users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL
);

-- down
DROP TABLE premix_migration_test_users;
"@
    Set-Content -Path $migrationFile.FullName -Value $sqlContent
    Write-Host "   -> SQL injected." -ForegroundColor Gray

    Write-Step "Running Migrations (UP)..."
    $databaseUrl = Get-DatabaseUrl
    $cliFeatures = @()
    if ($databaseUrl -like "postgres*") {
        $cliFeatures += "--features"
        $cliFeatures += "postgres"
    }
    elseif ($databaseUrl -like "mysql*") {
        $cliFeatures += "--features"
        $cliFeatures += "mysql"
    }
    cargo run -p premix-cli --quiet @cliFeatures -- migrate up --database $databaseUrl
    if ($LASTEXITCODE -ne 0) {
        throw "Migration failed for $databaseUrl."
    }

    Write-Step "Verifying Database..."
    if ($databaseUrl -like "sqlite:*") {
        if (Test-Path "premix.db") {
            Write-Success "Database 'premix.db' created."
        }
        else {
            throw "Database check failed."
        }
    }
    Write-Success "Migration applied to $databaseUrl"

    Write-Step "Rolling Back Last Migration..."
    cargo run -p premix-cli --quiet @cliFeatures -- migrate down --database $databaseUrl
    if ($LASTEXITCODE -ne 0) {
        throw "Rollback failed for $databaseUrl."
    }
    Write-Success "Rollback succeeded for $databaseUrl"

    $sw.Stop()
    Write-Header "VALIDATION PASSED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

