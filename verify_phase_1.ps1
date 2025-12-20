$baseUrl = "http://localhost:3000/api/v1"
# Using X-Tenant-Slug header for testing without subdomain
$headers = @{ "Content-Type" = "application/json"; "X-Tenant-Slug" = "demo" }

Write-Host "Waiting for server to be ready..."
$i = 0
do {
    try {
        $resp = Invoke-RestMethod -Uri "http://localhost:3000/health" -Method Get -ErrorAction Stop
        if ($resp.status -eq "ok") {
            Write-Host "Server is UP!"
            break
        }
    }
    catch {
        Write-Host "Waiting for server... ($i)"
        Start-Sleep -Seconds 2
        $i++
    }
} while ($i -lt 30)

if ($i -ge 30) {
    Write-Host "Server failed to start."
    exit 1
}

Write-Host "1. Create Entity Type 'project'..."
$projectBody = @{
    app_id      = "crm"
    name        = "project"
    label       = "Project"
    icon        = "briefcase"
    description = "Client projects"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/metadata/entities" -Method Post -Headers $headers -Body $projectBody
    Write-Host "Success! Entity ID: $($response)"
}
catch {
    Write-Host "Error creating entity: $($_.Exception.Message)"
    # Continue anyway, maybe it exists
}

Write-Host "`n2. Add 'name' field (Required)..."
$nameField = @{
    name        = "name"
    label       = "Project Name"
    field_type  = "Text"
    is_required = $true
    sort_order  = 10
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "$baseUrl/metadata/entities/project/fields" -Method Post -Headers $headers -Body $nameField | Out-Null
    Write-Host "Success adding name field"
}
catch {
    Write-Host "Error adding name field: $($_.Exception.Message)"
}

Write-Host "`n3. Add 'budget' field (Money)..."
$budgetField = @{
    name       = "budget"
    label      = "Budget"
    field_type = @{ type = "Money"; config = @{ currency = "USD" } }
    sort_order = 20
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "$baseUrl/metadata/entities/project/fields" -Method Post -Headers $headers -Body $budgetField | Out-Null
    Write-Host "Success adding budget field"
}
catch {
    Write-Host "Error adding budget field: $($_.Exception.Message)"
}

Write-Host "`n4. Create Valid Record..."
$validRecord = @{
    name   = "Website Redesign"
    budget = 5000
} | ConvertTo-Json

try {
    $rec = Invoke-RestMethod -Uri "$baseUrl/project/records" -Method Post -Headers $headers -Body $validRecord
    Write-Host "Success! Record ID: $($rec.id)"
}
catch {

    Write-Host "Error creating valid record: $($_.Exception.Message)"
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader $_.Exception.Response.GetResponseStream()
        Write-Host "Details: $($reader.ReadToEnd())"
    }
}

Write-Host "`n5. Create Invalid Record (Missing Name)..."
$invalidRecord = @{
    budget = 1000
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "$baseUrl/project/records" -Method Post -Headers $headers -Body $invalidRecord
    Write-Host "FAILED: Should have rejected invalid record!"
}
catch {

    Write-Host "Success! Rejected invalid record as expected."
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader $_.Exception.Response.GetResponseStream()
        Write-Host "Error Details: $($reader.ReadToEnd())"
    }
}
