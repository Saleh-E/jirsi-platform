$baseUrl = "http://localhost:3000/api/v1"
$tenantSlug = "demo" # Using 'demo' tenant from seed

function Test-Validation {
    Write-Host "`n--- Testing Field Validation ---" -ForegroundColor Cyan

    # 1. Invalid Email
    Write-Host "Testing invalid email..."
    $payload = @{
        first_name = "Test"
        last_name  = "User"
        email      = "invalid-email"
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri "$baseUrl/records/contact?tenant_slug=$tenantSlug" -Method Post -Body $payload -ContentType "application/json"
    }
    catch {
        $errorJson = $_.ErrorDetails.Message | ConvertFrom-Json
        if ($errorJson.error -like "*valid email*") {
            Write-Host "SUCCESS: Caught invalid email" -ForegroundColor Green
        }
        else {
            Write-Host "FAILURE: Unexpected error: $($errorJson.error)" -ForegroundColor Red
        }
    }

    # 2. Required Field Missing
    Write-Host "Testing missing required field (last_name)..."
    $payload = @{
        first_name = "Test"
        email      = "test@example.com"
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri "$baseUrl/records/contact?tenant_slug=$tenantSlug" -Method Post -Body $payload -ContentType "application/json"
    }
    catch {
        $errorJson = $_.ErrorDetails.Message | ConvertFrom-Json
        if ($errorJson.error -like "*required*") {
            Write-Host "SUCCESS: Caught missing required field" -ForegroundColor Green
        }
        else {
            Write-Host "FAILURE: Unexpected error: $($errorJson.error)" -ForegroundColor Red
        }
    }

    # 3. Invalid Property Price (Number)
    Write-Host "Testing invalid property price (string instead of number)..."
    $payload = @{
        title     = "Test Property"
        reference = "REF-123"
        city      = "London"
        price     = "expensive"
    } | ConvertTo-Json
    
    try {
        Invoke-RestMethod -Uri "$baseUrl/records/property?tenant_slug=$tenantSlug" -Method Post -Body $payload -ContentType "application/json"
    }
    catch {
        $errorJson = $_.ErrorDetails.Message | ConvertFrom-Json
        if ($errorJson.error -like "*number*") {
            Write-Host "SUCCESS: Caught invalid number type" -ForegroundColor Green
        }
        else {
            Write-Host "FAILURE: Unexpected error: $($errorJson.error)" -ForegroundColor Red
        }
    }
}

function Test-RLS {
    Write-Host "`n--- Testing RLS Isolation ---" -ForegroundColor Cyan
    
    # This is harder to test without a second tenant, but we can verify RlsConn is setting context.
    # We will try to fetch a record with NO tenant slug and expect failure from middleware.
    Write-Host "Testing request with NO tenant..."
    try {
        Invoke-RestMethod -Uri "$baseUrl/records/contact" -Method Get
    }
    catch {
        if ($_.Exception.Response.StatusCode -eq "BadRequest") {
            Write-Host "SUCCESS: Middleware rejected request without tenant" -ForegroundColor Green
        }
        else {
            Write-Host "FAILURE: Unexpected status: $($_.Exception.Response.StatusCode)" -ForegroundColor Red
        }
    }
}

# Run tests
Test-Validation
Test-RLS
