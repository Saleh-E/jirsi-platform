# Start Backend
Start-Process pwsh -ArgumentList "-NoExit", "-Command", "cd 'e:\s_programmer\Saas System'; cargo run --bin server"

# Wait 5 seconds for backend to start
Start-Sleep -Seconds 5

# Start Frontend
Start-Process pwsh -ArgumentList "-NoExit", "-Command", "cd 'e:\s_programmer\Saas System\crates\frontend-web'; trunk serve"

# Wait 8 seconds for frontend to start
Start-Sleep -Seconds 8

# Open browser
Start-Process "http://127.0.0.1:8080"

Write-Host "✅ Jirsi Platform is starting..." -ForegroundColor Green
Write-Host "Backend: http://localhost:3000" -ForegroundColor Cyan
Write-Host "Frontend: http://127.0.0.1:8080" -ForegroundColor Cyan
Write-Host "" 
Write-Host "Press Ctrl+C in each terminal to stop the servers." -ForegroundColor Yellow
