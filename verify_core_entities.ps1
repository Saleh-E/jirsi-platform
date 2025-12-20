$ErrorActionPreference = "Stop"
$baseUrl = "http://localhost:3000/api/v1"
$headers = @{
    "Content-Type"  = "application/json"
    "X-Tenant-Slug" = "demo"
}

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

function Verify-Entity {
    param ($Name)
    try {
        Write-Host "Verifying Entity '$Name'..." -NoNewline
        $entity = Invoke-RestMethod -Uri "$baseUrl/metadata/entities/$Name" -Method Get -Headers $headers
        Write-Host " Found! (ID: $($entity.id))" -ForegroundColor Green
        
        Write-Host "  Fetching fields..." -NoNewline
        $fields = Invoke-RestMethod -Uri "$baseUrl/metadata/entities/$Name/fields" -Method Get -Headers $headers
        Write-Host " Found $($fields.Count) fields." -ForegroundColor Green
        foreach ($f in $fields) {
            Write-Host "    - $($f.name) ($($f.field_type))" -ForegroundColor Gray
        }
        return $true
    }
    catch {
        Write-Host " Failed: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

Wait-For-Server
Write-Host "`n=== Verifying Phase 1 Core Entities ===`n" -ForegroundColor Cyan

$success = $true
$success = $success -and (Verify-Entity -Name "contact")
$success = $success -and (Verify-Entity -Name "company")
$success = $success -and (Verify-Entity -Name "deal")

if ($success) {
    Write-Host "`nAll Core Entities Verified!" -ForegroundColor Green
    exit 0
}
else {
    Write-Host "`nVerification Failed." -ForegroundColor Red
    exit 1
}
