$ErrorActionPreference = "Stop"

function Write-Step {
    param($Text)
    Write-Host "`n> $Text" -ForegroundColor Yellow
}

function Write-Success {
    param($Text)
    Write-Host "[OK] $Text" -ForegroundColor Green
}

function Write-TestResult {
    param($Name, $Passed)
    if ($Passed) {
        Write-Host "  [PASS] $Name" -ForegroundColor Green
    }
    else {
        Write-Host "  [FAIL] $Name" -ForegroundColor Red
    }
}

$ProgressPreference = 'SilentlyContinue' # suppress progress bars

$ServerProcess = $null
$ScriptRoot = $PSScriptRoot
Set-Location "$ScriptRoot/../.."
$BaseUrl = "http://127.0.0.1:3000"
$TestsPassed = 0
$TestsFailed = 0

try {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "   PREMIX ORM: COMPREHENSIVE E2E TEST" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    # 0. Clean up old database
    $DbPath = "premix_axum_demo.db"
    if (Test-Path $DbPath) {
        Write-Step "Cleaning up old database..."
        Remove-Item $DbPath -Force
        Write-Success "Old database removed."
    }

    # 1. Start Server in Background
    Write-Step "Starting Axum Server in background..."
    $ServerProcess = Start-Process cargo -ArgumentList "run", "-p", "premix-axum-tutorial" -PassThru -NoNewWindow -WorkingDirectory (Get-Location)
    Write-Host "   Server Process ID: $($ServerProcess.Id)" -ForegroundColor Gray

    # 2. Wait for Server to be ready
    Write-Step "Waiting for Server to be ready on Port 3000..."
    $RetryCount = 0
    $MaxRetries = 60
    $Ready = $false

    while (-not $Ready -and $RetryCount -lt $MaxRetries) {
        try {
            $null = Invoke-WebRequest -Uri "$BaseUrl/users" -Method Get -ErrorAction SilentlyContinue -UseBasicParsing
            $Ready = $true
        }
        catch {
            Start-Sleep -Seconds 1
            $RetryCount++
            Write-Host "." -NoNewline -ForegroundColor Gray
        }
    }

    if (-not $Ready) { throw "Server failed to start in time." }
    Write-Host ""
    Write-Success "Server is LIVE!"

    # ========================================
    # TEST SUITE
    # ========================================
    
    Write-Step "Running Test Suite..."

    # --- TEST 1: List Users (Empty) ---
    try {
        $Users = Invoke-RestMethod -Uri "$BaseUrl/users" -Method Get
        if ($Users.Count -eq 0) {
            Write-TestResult "GET /users (empty list)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "GET /users (empty list)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "GET /users (empty list)" $false
        $TestsFailed++
    }

    # --- TEST 2: Create User ---
    try {
        $UserPayload = @{ name = "Alice"; email = "alice@premix.io" } | ConvertTo-Json
        $CreatedUser = Invoke-RestMethod -Uri "$BaseUrl/users" -Method Post -ContentType "application/json" -Body $UserPayload
        if ($CreatedUser.name -eq "Alice" -and $CreatedUser.email -eq "alice@premix.io") {
            Write-TestResult "POST /users (create Alice)" $true
            $TestsPassed++
            $AliceId = $CreatedUser.id
        }
        else {
            Write-TestResult "POST /users (create Alice)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "POST /users (create Alice)" $false
        $TestsFailed++
    }

    # --- TEST 3: Create Second User ---
    try {
        $UserPayload = @{ name = "Bob"; email = "bob@premix.io" } | ConvertTo-Json
        $CreatedUser = Invoke-RestMethod -Uri "$BaseUrl/users" -Method Post -ContentType "application/json" -Body $UserPayload
        if ($CreatedUser.name -eq "Bob") {
            Write-TestResult "POST /users (create Bob)" $true
            $TestsPassed++
            $BobId = $CreatedUser.id
        }
        else {
            Write-TestResult "POST /users (create Bob)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "POST /users (create Bob)" $false
        $TestsFailed++
    }

    # --- TEST 4: List Users (2 users) ---
    try {
        $Users = Invoke-RestMethod -Uri "$BaseUrl/users" -Method Get
        if ($Users.Count -eq 2) {
            Write-TestResult "GET /users (list 2 users)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "GET /users (list 2 users)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "GET /users (list 2 users)" $false
        $TestsFailed++
    }

    # --- TEST 5: Get User by ID ---
    try {
        $User = Invoke-RestMethod -Uri "$BaseUrl/users/$AliceId" -Method Get
        if ($User.name -eq "Alice") {
            Write-TestResult "GET /users/{id} (fetch Alice)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "GET /users/{id} (fetch Alice)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "GET /users/{id} (fetch Alice)" $false
        $TestsFailed++
    }

    # --- TEST 6: Get Non-existent User (404) ---
    try {
        $null = Invoke-RestMethod -Uri "$BaseUrl/users/9999" -Method Get -ErrorAction Stop
        Write-TestResult "GET /users/9999 (expect 404)" $false
        $TestsFailed++
    }
    catch {
        if ($_.Exception.Response.StatusCode -eq 404 -or $_.Exception.Message -match "404") {
            Write-TestResult "GET /users/9999 (expect 404)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "GET /users/9999 (expect 404)" $false
            $TestsFailed++
        }
    }

    # --- TEST 7: Update User ---
    try {
        $UpdatePayload = @{ name = "Alice Updated" } | ConvertTo-Json
        $UpdatedUser = Invoke-RestMethod -Uri "$BaseUrl/users/$AliceId" -Method Put -ContentType "application/json" -Body $UpdatePayload
        if ($UpdatedUser.name -eq "Alice Updated" -and $UpdatedUser.email -eq "alice@premix.io") {
            Write-TestResult "PUT /users/{id} (update Alice name)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "PUT /users/{id} (update Alice name)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "PUT /users/{id} (update Alice name)" $false
        $TestsFailed++
    }

    # --- TEST 8: Verify Update ---
    try {
        $User = Invoke-RestMethod -Uri "$BaseUrl/users/$AliceId" -Method Get
        if ($User.name -eq "Alice Updated") {
            Write-TestResult "GET /users/{id} (verify update)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "GET /users/{id} (verify update)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "GET /users/{id} (verify update)" $false
        $TestsFailed++
    }

    # --- TEST 9: Delete User ---
    try {
        $Response = Invoke-WebRequest -Uri "$BaseUrl/users/$BobId" -Method Delete -UseBasicParsing
        if ($Response.StatusCode -eq 204) {
            Write-TestResult "DELETE /users/{id} (delete Bob)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "DELETE /users/{id} (delete Bob)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "DELETE /users/{id} (delete Bob)" $false
        $TestsFailed++
    }

    # --- TEST 10: Verify Deletion ---
    try {
        $null = Invoke-RestMethod -Uri "$BaseUrl/users/$BobId" -Method Get -ErrorAction Stop
        Write-TestResult "GET /users/{id} (verify Bob deleted - expect 404)" $false
        $TestsFailed++
    }
    catch {
        if ($_.Exception.Response.StatusCode -eq 404 -or $_.Exception.Message -match "404") {
            Write-TestResult "GET /users/{id} (verify Bob deleted - expect 404)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "GET /users/{id} (verify Bob deleted - expect 404)" $false
            $TestsFailed++
        }
    }

    # --- TEST 11: List Users After Delete ---
    try {
        $Users = Invoke-RestMethod -Uri "$BaseUrl/users" -Method Get
        if ($Users.Count -eq 1) {
            Write-TestResult "GET /users (list after delete, expect 1)" $true
            $TestsPassed++
        }
        else {
            Write-TestResult "GET /users (list after delete, expect 1)" $false
            $TestsFailed++
        }
    }
    catch {
        Write-TestResult "GET /users (list after delete, expect 1)" $false
        $TestsFailed++
    }

    # ========================================
    # SUMMARY
    # ========================================
    Write-Host "`n========================================" -ForegroundColor Cyan
    if ($TestsFailed -eq 0) {
        Write-Host "   ALL $TestsPassed TESTS PASSED!" -ForegroundColor Green
    }
    else {
        Write-Host "   $TestsPassed PASSED, $TestsFailed FAILED" -ForegroundColor Red
    }
    Write-Host "========================================" -ForegroundColor Cyan
}
catch {
    Write-Host "`n[FAIL] TEST FAILED: $_" -ForegroundColor Red
}
finally {
    # Cleanup Server
    if ($ServerProcess -and -not $ServerProcess.HasExited) {
        Write-Step "Shutting down Server (PID: $($ServerProcess.Id))..."
        Stop-Process -Id $ServerProcess.Id -Force -ErrorAction SilentlyContinue
        Write-Success "Server stopped."
    }
}
