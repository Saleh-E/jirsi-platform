# Backend API Verification Script - Phase 3 (CRM & Real Estate)
# Tests Deals, Tasks, Properties, Viewings against generic entity_records backend

$ErrorActionPreference = "Stop"

$BASE_URL = "http://localhost:3000/api/v1"
$TENANT_ID = "11111111-1111-1111-1111-111111111111"
$HEADERS = @{
    "Content-Type" = "application/json"
    "X-Tenant-Id"  = $TENANT_ID
}

function Test-Entity-CRUD {
    param (
        [string]$EntityName,
        [hashtable]$CreatePayload,
        [hashtable]$UpdatePayload
    )

    Write-Host "`n--- Testing Entity: $EntityName ---" -ForegroundColor Cyan

    # 1. LIST (Initial)
    try {
        $listUrl = "$BASE_URL/entities/$EntityName?tenant_id=$TENANT_ID"
        $listResponse = Invoke-RestMethod -Uri $listUrl -Method Get -Headers $HEADERS
        Write-Host "Please verify list count before create: $($listResponse.total)"
    }
    catch {
        Write-Error "Failed to list $EntityName : $_"
    }

    # 2. CREATE
    try {
        $createUrl = "$BASE_URL/entities/$EntityName?tenant_id=$TENANT_ID"
        $jsonPayload = $CreatePayload | ConvertTo-Json
        $createResponse = Invoke-RestMethod -Uri $createUrl -Method Post -Headers $HEADERS -Body $jsonPayload
        $entityId = $createResponse.id
        Write-Host "Created $EntityName with ID: $entityId"
    }
    catch {
        Write-Error "Failed to create $EntityName : $_"
        return
    }

    # 3. GET (Verify Create)
    try {
        $getUrl = "$BASE_URL/entities/$EntityName/$entityId?tenant_id=$TENANT_ID"
        $getResponse = Invoke-RestMethod -Uri $getUrl -Method Get -Headers $HEADERS
        Write-Host "Retrieved created $EntityName : $($getResponse | ConvertTo-Json -Depth 2)"
    }
    catch {
        Write-Error "Failed to get $EntityName : $_"
    }

    # 4. UPDATE
    try {
        $updateUrl = "$BASE_URL/entities/$EntityName/$entityId?tenant_id=$TENANT_ID"
        $updateJson = $UpdatePayload | ConvertTo-Json
        $updateResponse = Invoke-RestMethod -Uri $updateUrl -Method Put -Headers $HEADERS -Body $updateJson
        Write-Host "Updated $EntityName : $($updateResponse | ConvertTo-Json -Depth 2)"
    }
    catch {
        Write-Error "Failed to update $EntityName : $_"
    }

    # 5. GET (Verify Update)
    try {
        $getUrl = "$BASE_URL/entities/$EntityName/$entityId?tenant_id=$TENANT_ID"
        $getResponse = Invoke-RestMethod -Uri $getUrl -Method Get -Headers $HEADERS
        Write-Host "Retrieved updated $EntityName : $($getResponse | ConvertTo-Json -Depth 2)"
    }
    catch {
        Write-Error "Failed to get updated $EntityName : $_"
    }

    # 6. DELETE (Soft Delete)
    try {
        $deleteUrl = "$BASE_URL/entities/$EntityName/$entityId?tenant_id=$TENANT_ID"
        $deleteResponse = Invoke-RestMethod -Uri $deleteUrl -Method Delete -Headers $HEADERS
        Write-Host "Deleted $EntityName : $($deleteResponse | ConvertTo-Json -Depth 2)"
    }
    catch {
        Write-Error "Failed to delete $EntityName : $_"
    }

    # 7. LIST (Verify Delete)
    try {
        $listUrl = "$BASE_URL/entities/$EntityName?tenant_id=$TENANT_ID"
        $listResponse = Invoke-RestMethod -Uri $listUrl -Method Get -Headers $HEADERS
        # Verify ID is not in list
        $found = $listResponse.data | Where-Object { $_.id -eq $entityId }
        if ($found) {
            Write-Error "Entity $EntityName with ID $entityId still found in list after delete!"
        }
        else {
            Write-Host "Verified $EntityName $entityId is no longer in list." -ForegroundColor Green
        }
    }
    catch {
        Write-Error "Failed to list after delete: $_"
    }
}

# --- TEST CASES ---

# DEAL
$dealCreate = @{
    name                = "Phase 3 Deal"
    amount              = 50000.00
    stage               = "qualification"
    expected_close_date = "2025-06-30"
}
$dealUpdate = @{
    amount = 55000.00
    stage  = "proposal"
}
Test-Entity-CRUD -EntityName "deal" -CreatePayload $dealCreate -UpdatePayload $dealUpdate

# TASK (Create pipeline_id not needed for tasks?)
# Wait, for deal creation, backend requires pipeline_id?
# Backend code: `let pipeline_id ... fetch default`. So it should work if seed worked.
# Task requires `created_by`. Backend tries to fetch default user.
$taskCreate = @{
    title       = "Migrate CRM Data"
    description = "Move legacy tables to entity_records"
    priority    = "high"
    status      = "in_progress"
    due_date    = "2025-01-15"
}
$taskUpdate = @{
    status   = "completed"
    priority = "critical"
}
Test-Entity-CRUD -EntityName "task" -CreatePayload $taskCreate -UpdatePayload $taskUpdate

# PROPERTY
$propCreate = @{
    title         = "Luxury Apartment"
    reference     = "APT-101"
    property_type = "apartment"
    status        = "active"
    price         = 1200000.00
    bedrooms      = 3
    bathrooms     = 2
}
$propUpdate = @{
    price  = 1250000.00
    status = "sold"
}
Test-Entity-CRUD -EntityName "property" -CreatePayload $propCreate -UpdatePayload $propUpdate

# VIEWING
# Viewings require property_id and contact_id usually.
# Backend validation? `create_record` logic for viewing is generic insert.
# But application logic might require valid IDs?
# We'll just pass some UUIDs or valid ones if possible.
# Ideally we should create a property and contact first, but for generic CRUD test, 
# if there's no FK constraint in `entity_records` JSONB (there isn't), it should accept any UUID string.
$viewingCreate = @{
    property_id  = "00000000-0000-0000-0000-000000000001" # Mock ID
    contact_id   = "00000000-0000-0000-0000-000000000002"  # Mock ID
    scheduled_at = "2025-02-01T10:00:00Z"
    status       = "scheduled"
}
$viewingUpdate = @{
    status   = "completed"
    feedback = "Client liked the kitchen"
}
Test-Entity-CRUD -EntityName "viewing" -CreatePayload $viewingCreate -UpdatePayload $viewingUpdate
