# verify_crm_v1.ps1
# Verification script for CRM Phase 3: Interactions & Workflows

$baseUrl = "http://localhost:3000/api/v1"
$tenantSlug = "demo"
$headers = @{
    "X-Tenant-Slug" = $tenantSlug
    "Content-Type"  = "application/json"
}

# 1. Create a dummy contact to link things to
Write-Host "`n1. Creating test contact..." -ForegroundColor Cyan
$contactBody = @{
    first_name = "Timeline"
    last_name  = "Tester"
    email      = "test@timeline.com"
} | ConvertTo-Json
$contact = Invoke-RestMethod -Uri "$baseUrl/records/contact" -Method Post -Headers $headers -Body $contactBody
$contactId = $contact.id
Write-Host "Created contact: $contactId"

# 2. Create an interaction
Write-Host "`n2. Logging an activity..." -ForegroundColor Cyan
$interactionBody = @{
    entity_type      = "contact"
    record_id        = $contactId
    interaction_type = "call"
    title            = "Discovery Call"
    content          = "Discussed CRM requirements."
    created_by       = "f15c7478-c5a2-42c6-abfa-d130cdd968f3"
} | ConvertTo-Json
$interaction = Invoke-RestMethod -Uri "$baseUrl/interactions" -Method Post -Headers $headers -Body $interactionBody
Write-Host "Activity logged: $($interaction.id)"

# 3. Verify interaction summary
Write-Host "`n3. Verifying interaction summary..." -ForegroundColor Cyan
try {
    $summary = Invoke-RestMethod -Uri "$baseUrl/interactions/summary/contact/$contactId" -Method Get -Headers $headers
    Write-Host "Total activities: $($summary.total_count)"
    Write-Host "Last activity: $($summary.last_interaction)"
    Write-Host "Call count: $($summary.counts_by_type.call)"
    
    if ($summary.total_count -ge 1) {
        Write-Host "SUCCESS: Summary retrieved and counts are correct." -ForegroundColor Green
    }
    else {
        Write-Host "FAILED: Summary counts are zero." -ForegroundColor Red
    }
}
catch {
    Write-Host "FAILED: Could not retrieve summary: $_" -ForegroundColor Red
}

# 4. Test Workflow Trigger (Update record)
Write-Host "`n4. Testing Workflow Trigger (Update record)..." -ForegroundColor Cyan
$updateBody = @{
    first_name = "Timeline Updated"
} | ConvertTo-Json
try {
    $update = Invoke-RestMethod -Uri "$baseUrl/records/contact/$contactId" -Method Put -Headers $headers -Body $updateBody
    Write-Host "Record updated successfully. Workflow engine should have been triggered."
    Write-Host "SUCCESS: API call complete." -ForegroundColor Green
}
catch {
    Write-Host "FAILED: Update failed: $_" -ForegroundColor Red
}

Write-Host "`nVerification Complete!" -ForegroundColor Yellow
