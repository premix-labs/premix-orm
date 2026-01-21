#!/usr/bin/env powershell
param (
    [int]$Core = 2,               # Pin to Core 2 (mask 4) by default
    [string]$Priority = "High"    # High priority class
)

$ErrorActionPreference = "Stop"

$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/../.."

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

function Invoke-Pinned {
    param(
        [string]$Command,
        [string[]]$Arguments
    )
    
    $process = Start-Process -FilePath $Command -ArgumentList $Arguments -NoNewWindow -PassThru
    
    try {
        $affinityMask = [IntPtr](1 -shl $Core)
        $process.ProcessorAffinity = $affinityMask
        $process.PriorityClass = $Priority
    }
    catch {
        Write-Warning "Failed to set affinity/priority: $_"
    }
    
    $process.WaitForExit()
    
    if ($process.ExitCode -ne 0) {
        throw "$Command failed with exit code $($process.ExitCode)"
    }
}

try {
    Write-Header "PREMIX ORM: QUICK TEST SUITE (Pinned Core $Core)"
    Write-Host "Configuration:" -ForegroundColor Gray
    Write-Host "  - CPU Core: $Core" -ForegroundColor Gray
    Write-Host "  - Priority: $Priority" -ForegroundColor Gray
    
    Write-Step "Building Workspace..."
    Invoke-Pinned "cargo" @("build", "--workspace")

    Write-Step "Running basic-app smoke test..."
    Invoke-Pinned "cargo" @("run", "-p", "basic-app")

    Write-Success "QUICK TEST PASSED."
}
catch {
    Write-Host "`n[FAILED] $_" -ForegroundColor Red
    exit 1
}

