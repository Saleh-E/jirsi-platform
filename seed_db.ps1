$ErrorActionPreference = "Stop"
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/seed" -Method Post
    Write-Host "Seed Result: $($response | ConvertTo-Json -Depth 5)"
}
catch {
    Write-Error "Failed to seed database: $_"
}
