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
    Write-Header "PREMIX ORM: DOCUMENTATION GENERATOR"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."
    
    Write-Step "Generating Rust Documentation..."
    cargo doc --no-deps

    $docPath = Join-Path $ScriptRoot "..\..\target\doc\premix_orm\index.html"
    if (Test-Path $docPath) {
        Write-Host "     Opening $docPath"
        Start-Process $docPath
    }
    else {
        Write-Host "[WARN] premix_orm docs not found at $docPath" -ForegroundColor DarkGray
    }

    if (Get-Command mdbook -ErrorAction SilentlyContinue) {
        Write-Step "Building The Book (mdBook)..."
        if (Test-Path "orm-book") {
            Push-Location "orm-book"
            mdbook build
            Pop-Location
            Write-Success "Book built in orm-book/book"
        }
        else {
            Write-Host "[WARN] 'orm-book' directory not found." -ForegroundColor DarkGray
        }
    }
    else {
        Write-Host "[WARN] mdBook not installed (Skipping)." -ForegroundColor DarkGray
    }

    $sw.Stop()
    Write-Header "DOCS GENERATED in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

