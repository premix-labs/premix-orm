$ErrorActionPreference = "Continue" # Continue on error to run all examples
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

function Write-Fail {
    param($Text)
    Write-Host "[FAIL] $Text" -ForegroundColor Red
}

try {
    Write-Header "PREMIX ORM: EXAMPLE APP SUITE"
    
    $ScriptRoot = $PSScriptRoot
    Set-Location "$ScriptRoot/../.."

    $examples = @(
        "basic-app",
        "axum-app",
        "premix-axum-tutorial",
        "relation-app",
        "eager-app",
        "transaction-app",
        "hooks-app",
        "optimistic-locking-app",
        "validation-app",
        "soft-delete-app",
        "bulk-ops-app",
        "tracing-app"
    )

    $passed = 0
    $failed = 0

    foreach ($example in $examples) {
        Write-Step "Running $example..."
        $result = cargo run -p $example 2>&1
        
        if ($LASTEXITCODE -eq 0) {
            Write-Success "$example passed"
            $passed++
        }
        else {
            Write-Fail "$example failed"
            $result | Write-Host -ForegroundColor DarkGray
            $failed++
        }
    }

    $sw.Stop()
    
    Write-Header "SUMMARY: $passed Passed, $failed Failed in $($sw.Elapsed.TotalSeconds.ToString("N2"))s"
    
    if ($failed -gt 0) {
        exit 1
    }
}
catch {
    Write-Host "`n[FATAL ERROR]: $_" -ForegroundColor Red
    exit 1
}

