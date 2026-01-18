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

function Install-Tool {
    param($Name, $Package = $Name)
    Write-Step "Checking for $Name..."
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Host "   -> $Name not found. Installing..." -ForegroundColor Gray
        cargo install $Package
        Write-Success "$Name installed."
    }
    else {
        Write-Success "$Name is already installed."
    }
}

try {
    Write-Header "PREMIX ORM: DEV ENVIRONMENT SETUP"
    
    # 1. Check Rustup Components
    Write-Step "Checking Rust Components..."
    rustup component add rustfmt clippy
    Write-Success "Rustfmt and Clippy are ready."

    # 2. Install Cargo Tools
    Install-Tool "cargo-audit"
    Install-Tool "cargo-tarpaulin"
    Install-Tool "mdbook"
    
    # 3. Optional: Install cargo-edit for dependency management
    Install-Tool "cargo-upgrade" "cargo-edit"

    $sw.Stop()
    Write-Header "SETUP COMPLETE in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
    Write-Host "   You can now run CI/CD scripts in ./scripts/ci/" -ForegroundColor Gray
}
catch {
    Write-Host "`n[FAILED] Setup failed: $_" -ForegroundColor Red
    exit 1
}
