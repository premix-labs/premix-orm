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
    Write-Host "`n➜ $Text" -ForegroundColor Yellow
}

function Write-Success {
    param($Text)
    Write-Host "✅ $Text" -ForegroundColor Green
}

function Test-Command {
    param($Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Error: '$Name' is not installed." -ForegroundColor Red
        exit 1
    }
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
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT NOT NULL
);

-- down
DROP TABLE users;
"@
    Set-Content -Path $migrationFile.FullName -Value $sqlContent
    Write-Host "   -> SQL injected." -ForegroundColor Gray

    Write-Step "Running Migrations (UP)..."
    # Force SQLite for local validation
    cargo run -p premix-cli --quiet -- migrate up --database "sqlite:premix.db"

    Write-Step "Verifying Database..."
    if (Test-Path "premix.db") {
        Write-Success "Database 'premix.db' created."
    }
    else {
        throw "Database check failed."
    }

    $sw.Stop()
    Write-Header "VALIDATION PASSED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n❌ FAILED: $_" -ForegroundColor Red
    exit 1
}

