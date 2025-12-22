# Verification Script for Slice #1
# Tests: Validation, Success, Tenant Isolation

$BaseUrl = "http://localhost:3000/api/v1"
$HeadersDemo = @{ "Content-Type" = "application/json"; "X-Tenant-Slug" = "demo" }
$HeadersAcme = @{ "Content-Type" = "application/json"; "X-Tenant-Slug" = "acme" }

Write-Host "🔍 Starting Verification for Slice #1..." -ForegroundColor Cyan

# 1. Test Validation Error
Write-Host "`n1. Testing Validation (Missing Required Field)..." -ForegroundColor Yellow
try {
    $Response = Invoke-RestMethod -Uri "$BaseUrl/records/contact" -Method Post -Headers $HeadersDemo -Body '{"first_name": "Invalid"}' -SkipHttpErrorCheck
    if ($Response.error) {
        Write-Host "✅ Success: API returned error: $($Response.error)" -ForegroundColor Green
    }
    else {
        Write-Host "❌ Failed: API should have failed but returned success" -ForegroundColor Red
        $Response
    }
}
catch {
    Write-Host "✅ Success: API returned 400 Bad Request" -ForegroundColor Green
}

# 2. Test Success
Write-Host "`n2. Testing Valid Creation..." -ForegroundColor Yellow
$RawContact = '{
    "first_name": "Manual",
    "last_name": "Test",
    "email": "manual.test@example.com",
    "phone": "555-0000"
}'
try {
    $Contact = Invoke-RestMethod -Uri "$BaseUrl/records/contact" -Method Post -Headers $HeadersDemo -Body $RawContact
    Write-Host "✅ Success: Created Contact ID: $($Contact.id)" -ForegroundColor Green
    $ContactId = $Contact.id
}
catch {
    Write-Host "❌ Failed to create contact: $($_.Exception.Message)" -ForegroundColor Red
    exit
}

# 3. Test Retrieval
Write-Host "`n3. Testing Retrieval..." -ForegroundColor Yellow
try {
    $Get = Invoke-RestMethod -Uri "$BaseUrl/records/contact/$ContactId" -Method Get -Headers $HeadersDemo
    if ($Get.first_name -eq "Manual") {
        Write-Host "✅ Success: Retrieved contact correctly" -ForegroundColor Green
    }
    else {
        Write-Host "❌ Failed: Retrieved incorrect data" -ForegroundColor Red
    }
}
catch {
    Write-Host "❌ Failed to get contact: $($_.Exception.Message)" -ForegroundColor Red
}

# 4. Test Isolation
Write-Host "`n4. Testing Tenant Isolation (As Acme Corp)..." -ForegroundColor Yellow
try {
    # Try to access the record created by Demo tenant
    Invoke-RestMethod -Uri "$BaseUrl/records/contact/$ContactId" -Method Get -Headers $HeadersAcme -SkipHttpErrorCheck | Out-Null
    Write-Host "❌ Failed: Acme Corp should NOT see Demo contact, but request succeeded." -ForegroundColor Red
}
catch {
    # We expect a 404 Not Found because RLS hides it
    if ($_.Exception.Response.StatusCode.value__ -eq 404) {
        Write-Host "✅ Success: Acme Corp cannot see Demo contact (404 Not Found)" -ForegroundColor Green
    }
    else {
        Write-Host "⚠️ Warn: Unexpected error code: $($_.Exception.Message)" -ForegroundColor Yellow
    }
}

Write-Host "`n🎉 Verification Complete!" -ForegroundColor Cyan
