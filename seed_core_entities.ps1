$ErrorActionPreference = "Stop"
$baseUrl = "http://localhost:3000/api/v1"
$headers = @{
    "Content-Type"  = "application/json"
    "X-Tenant-Slug" = "demo"
}

# ---------------------------------------------------------
# Helper Functions
# ---------------------------------------------------------

function Wait-For-Server {
    Write-Host "Waiting for server to be ready..." -NoNewline
    $retries = 0
    do {
        try {
            $response = Invoke-RestMethod -Uri "http://localhost:3000/health" -Method Get -ErrorAction SilentlyContinue
            if ($response.status -eq "ok") {
                Write-Host "`nServer is UP!" -ForegroundColor Green
                return
            }
        }
        catch {
            Start-Sleep -Seconds 1
            Write-Host "." -NoNewline
            $retries++
        }
    } while ($retries -lt 30)
    Write-Error "`nServer failed to start."
}

function Create-Entity {
    param (
        [string]$AppId,
        [string]$Name,
        [string]$Label,
        [string]$Icon,
        [string]$Description
    )

    $body = @{
        app_id      = $AppId
        name        = $Name
        label       = $Label
        icon        = $Icon
        description = $Description
    } | ConvertTo-Json

    try {
        Write-Host "Creating Entity Type '$Name'..." -NoNewline
        $response = Invoke-RestMethod -Uri "$baseUrl/metadata/entities" -Method Post -Headers $headers -Body $body
        Write-Host " Success! ID: $response" -ForegroundColor Green
        return $response
    }
    catch {
        $err = $_.Exception.Response
        if ($err.StatusCode -eq 400 -or $err.StatusCode -eq 409) {
            # If it already exists (assuming backend enforces it or returns specific error), we ideally explicitly check first.
            # But generic API might create duplicates if no constraint. 
            # For now, let's assume we are running clean or accept failure.
            Write-Host " Failed (or exists): $($err.StatusCode)" -ForegroundColor Yellow
        }
        else {
            Write-Host " Failed: $($err.StatusCode) - $($_.Exception.Message)" -ForegroundColor Red
            Write-Host $_.ErrorDetails.Message
        }
        return $null
    }
}

function Add-Field {
    param (
        [string]$EntityName,
        [string]$Name,
        [string]$Label,
        [string]$Type,
        [bool]$Required = $false,
        [bool]$List = $true,
        [hashtable]$Options = $null
    )

    # Basic type mapping to JSON structure expected by backend
    $fieldTypeJson = @{}
    
    switch ($Type) {
        "text" { $fieldTypeJson = "Text" }
        "number" { $fieldTypeJson = @{ type = "Number"; config = @{ decimals = 0 } } }
        "money" { $fieldTypeJson = @{ type = "Money"; config = @{ currency = "USD" } } }
        "email" { $fieldTypeJson = "Email" }
        "phone" { $fieldTypeJson = "Phone" }
        "url" { $fieldTypeJson = "Url" }
        "date" { $fieldTypeJson = "Date" }
        "select" { 
            $optsList = @()
            if ($Options) {
                foreach ($key in $Options.Keys) {
                    $optsList += @{ value = $key; label = $Options[$key] }
                }
            }
            $fieldTypeJson = @{ type = "Select"; config = @{ options = $optsList } } 
        }
        Default { $fieldTypeJson = "Text" }
    }

    $body = @{
        name         = $Name
        label        = $Label
        field_type   = $fieldTypeJson
        is_required  = $Required
        show_in_list = $List
    } | ConvertTo-Json -Depth 10

    try {
        Write-Host "  Adding Field '$Name'..." -NoNewline
        $url = "$baseUrl/metadata/entities/$EntityName/fields"
        $response = Invoke-RestMethod -Uri $url -Method Post -Headers $headers -Body $body
        Write-Host " Success!" -ForegroundColor Green
    }
    catch {
        Write-Host " Failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# ---------------------------------------------------------
# Execution
# ---------------------------------------------------------

Wait-For-Server

Write-Host "`n=== Seeding Core Entities ===`n" -ForegroundColor Cyan

# 1. Contact
Create-Entity -AppId "crm" -Name "contact" -Label "Contact" -Icon "user" -Description "People you do business with"
Add-Field -EntityName "contact" -Name "first_name" -Label "First Name" -Type "text" -Required $true
Add-Field -EntityName "contact" -Name "last_name" -Label "Last Name" -Type "text" -Required $true
Add-Field -EntityName "contact" -Name "email" -Label "Email" -Type "email"
Add-Field -EntityName "contact" -Name "phone" -Label "Phone" -Type "phone"
Add-Field -EntityName "contact" -Name "lifecycle_stage" -Label "Lifecycle Stage" -Type "select" -Options @{ "lead" = "Lead"; "customer" = "Customer"; "churned" = "Churned" }

# 2. Company
Create-Entity -AppId "crm" -Name "company" -Label "Company" -Icon "building" -Description "Organizations and businesses"
Add-Field -EntityName "company" -Name "name" -Label "Company Name" -Type "text" -Required $true
Add-Field -EntityName "company" -Name "domain" -Label "Domain" -Type "url"
Add-Field -EntityName "company" -Name "industry" -Label "Industry" -Type "text"

# 3. Deal
Create-Entity -AppId "crm" -Name "deal" -Label "Deal" -Icon "dollar-sign" -Description "Potential revenue opportunities"
Add-Field -EntityName "deal" -Name "name" -Label "Deal Name" -Type "text" -Required $true
Add-Field -EntityName "deal" -Name "amount" -Label "Amount" -Type "money"
Add-Field -EntityName "deal" -Name "stage" -Label "Stage" -Type "select" -Options @{ "new" = "New"; "qualified" = "Qualified"; "won" = "Won"; "lost" = "Lost" }
Add-Field -EntityName "deal" -Name "expected_close_date" -Label "Expected Close" -Type "date"

Write-Host "`nSeeding Complete!" -ForegroundColor Cyan
