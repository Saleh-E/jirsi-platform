# Quick setup script for Jirsi platform (PowerShell)

Write-Host "🚀 Jirsi Platform - Quick Setup" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan

# Check prerequisites
Write-Host "📋 Checking prerequisites..." -ForegroundColor Yellow

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Docker not found. Please install Docker Desktop first." -ForegroundColor Red
    exit 1
}

if (-not (Get-Command docker-compose -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Docker Compose not found. Please install Docker Compose first." -ForegroundColor Red
    exit 1
}

Write-Host "✅ Docker and Docker Compose found" -ForegroundColor Green

# Create .env file if it doesn't exist
if (-not (Test-Path .env.docker)) {
    Write-Host "📝 Creating .env.docker from template..." -ForegroundColor Yellow
    Copy-Item .env.docker.example .env.docker
    Write-Host "⚠️  Please edit .env.docker with secure passwords!" -ForegroundColor Yellow
    Write-Host "   Press Enter to continue after editing..." -ForegroundColor Yellow
    Read-Host
}

# Start containers
Write-Host "🐳 Starting Docker containers..." -ForegroundColor Yellow
docker-compose up -d

# Wait for PostgreSQL to be ready
Write-Host "⏳ Waiting for PostgreSQL to be ready..." -ForegroundColor Yellow
Start-Sleep -Seconds 5

$retries = 0
$maxRetries = 30
while ($retries -lt $maxRetries) {
    $result = docker exec jirsi-postgres pg_isready -U jirsi -d jirsi_db 2>&1
    if ($LASTEXITCODE -eq 0) {
        break
    }
    Write-Host "   PostgreSQL is unavailable - sleeping" -ForegroundColor Gray
    Start-Sleep -Seconds 2
    $retries++
}

if ($retries -eq $maxRetries) {
    Write-Host "❌ PostgreSQL failed to start" -ForegroundColor Red
    exit 1
}

Write-Host "✅ PostgreSQL is ready" -ForegroundColor Green

# Wait for Redis to be ready
Write-Host "⏳ Waiting for Redis to be ready..." -ForegroundColor Yellow
$retries = 0
while ($retries -lt $maxRetries) {
    $result = docker exec jirsi-redis redis-cli ping 2>&1
    if ($result -match "PONG") {
        break
    }
    Write-Host "   Redis is unavailable - sleeping" -ForegroundColor Gray
    Start-Sleep -Seconds 2
    $retries++
}

if ($retries -eq $maxRetries) {
    Write-Host "❌ Redis failed to start" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Redis is ready" -ForegroundColor Green

# Run migrations
Write-Host "📦 Running database migrations..." -ForegroundColor Yellow
if (Get-Command sqlx -ErrorAction SilentlyContinue) {
    sqlx migrate run
} else {
    Write-Host "⚠️  sqlx-cli not found. Skipping migrations." -ForegroundColor Yellow
    Write-Host "   Install with: cargo install sqlx-cli --no-default-features --features postgres" -ForegroundColor Gray
}

Write-Host ""
Write-Host "🎉 Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "🔗 Access points:" -ForegroundColor Cyan
Write-Host "   PostgreSQL: localhost:5432"
Write-Host "   Redis: localhost:6379"
Write-Host ""
Write-Host "📚 Next steps:" -ForegroundColor Cyan
Write-Host "   1. Run backend: cd crates\backend-api; cargo run"
Write-Host "   2. Run frontend: cd crates\frontend-web; trunk serve"
Write-Host ""
Write-Host "💡 Development tools (optional):" -ForegroundColor Cyan
Write-Host "   pgAdmin: http://localhost:5050"
Write-Host "   Redis Commander: http://localhost:8081"
Write-Host "   Start with: docker-compose --profile dev up -d"
