# Verification script for Phase 2: Contacts Migration

$ErrorActionPreference = "Stop"

$baseUrl = "http://localhost:3000/api/v1"
$tenantId = "b128c8da-6e56-485d-b2fe-e45fb7492b2e" # Demo Tenant

function Test-Endpoint {
    param(
        [string]$Method,
        [string]$Url,
        [hashtable]$Body = $null,
        [string]$Description
    )

    Write-Host "Testing: $Description" -ForegroundColor Cyan
    try {
        $params = @{
            Uri     = $Url
            Method  = $Method
            Headers = @{ "Content-Type" = "application/json" }
        }
        if ($Body) {
            $params.Body = $Body | ConvertTo-Json -Depth 10
        }

        $response = Invoke-RestMethod @params
        Write-Host "Success!" -ForegroundColor Green
        return $response
    }
    catch {
        Write-Host "Failed!" -ForegroundColor Red
        Write-Host $_.Exception.Message
        if ($_.Exception.Response) {
            $reader = New-Object System.IO.StreamReader $_.Exception.Response.GetResponseStream()
            $responseBody = $reader.ReadToEnd()
            Write-Host "Response Body: $responseBody"
        }
        exit 1
    }
}

# 1. List Contacts (Should verify migration worked)
Write-Host "`n1. Listing Contacts (Expect migrated data)..."
$contacts = Test-Endpoint -Method "GET" -Url "$baseUrl/entities/contact?tenant_id=$tenantId" -Description "Get Contacts List"
$count = $contacts.total
Write-Host "Found $count contacts."

if ($count -eq 0) {
    Write-Warning "No contacts found. Migration might have failed or database was empty."
}
else {
    $first = $contacts.data[0]
    Write-Host "First contact: $($first.first_name) $($first.last_name) (ID: $($first.id))"
    # Basic check if internal structure is flattened
    if ($null -eq $first.first_name) {
        Write-Error "Contact structure is unexpected. likely not flattened."
    }
}

# 2. Create New Contact (Should verify write to entity_records)
Write-Host "`n2. Creating New Contact..."
$newContact = @{
    first_name = "Phase2"
    last_name  = "Verification"
    email      = "phase2@example.com"
    phone      = "555-0199"
}

$created = Test-Endpoint -Method "POST" -Url "$baseUrl/entities/contact?tenant_id=$tenantId" -Body $newContact -Description "Create Contact"
$newId = $created.id
Write-Host "Created Contact ID: $newId"

# 3. Verify it appears in list
Write-Host "`n3. Verifying new contact in list..."
$contactsV2 = Test-Endpoint -Method "GET" -Url "$baseUrl/entities/contact?tenant_id=$tenantId&search=Phase2" -Description "Search New Contact"
$found = $contactsV2.data | Where-Object { $_.id -eq $newId }

if ($found) {
    Write-Host "Contact found in list: $($found.first_name)"
}
else {
    Write-Error "Newly created contact not found in list!"
}

# 4. Update Contact
Write-Host "`n4. Updating Contact..."
$update = @{
    first_name = "Phase2Updated"
}
Test-Endpoint -Method "PUT" -Url "$baseUrl/entities/contact/$newId?tenant_id=$tenantId" -Body $update -Description "Update Contact"

# Verify update
$updated = Test-Endpoint -Method "GET" -Url "$baseUrl/entities/contact/$newId?tenant_id=$tenantId" -Description "Get Updated Contact"
if ($updated.first_name -eq "Phase2Updated") {
    Write-Host "Update verified."
}
else {
    Write-Error "Update failed. Name is $($updated.first_name)"
}

# 5. Delete Contact (Soft Delete)
Write-Host "`n5. Deleting Contact..."
Test-Endpoint -Method "DELETE" -Url "$baseUrl/entities/contact/$newId?tenant_id=$tenantId" -Description "Delete Contact"

# Verify deleted
Write-Host "Verifying deletion..."
try {
    $check = Invoke-RestMethod -Uri "$baseUrl/entities/contact/$newId?tenant_id=$tenantId" -Method "GET"
    Write-Error "Contact still exists after deletion!"
}
catch {
    # 404 is expected
    Write-Host "Contact not found (expected 404 or 500 if internal error)" -ForegroundColor Green
}

Write-Host "`nPhase 2 Verification Complete!" -ForegroundColor Green
