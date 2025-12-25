#!/bin/bash
# Quick setup script for Jirsi platform

set -e

echo "🚀 Jirsi Platform - Quick Setup"
echo "================================"

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose not found. Please install Docker Compose first."
    exit 1
fi

echo "✅ Docker and Docker Compose found"

# Create .env file if it doesn't exist
if [ ! -f .env.docker ]; then
    echo "📝 Creating .env.docker from template..."
    cp .env.docker.example .env.docker
    echo "⚠️  Please edit .env.docker with secure passwords!"
    echo "   Press Enter to continue after editing..."
    read
fi

# Start containers
echo "🐳 Starting Docker containers..."
docker-compose up -d

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
sleep 5

until docker exec jirsi-postgres pg_isready -U jirsi -d jirsi_db > /dev/null 2>&1; do
    echo "   PostgreSQL is unavailable - sleeping"
    sleep 2
done

echo "✅ PostgreSQL is ready"

# Wait for Redis to be ready
echo "⏳ Waiting for Redis to be ready..."
until docker exec jirsi-redis redis-cli ping > /dev/null 2>&1; do
    echo "   Redis is unavailable - sleeping"
    sleep 2
done

echo "✅ Redis is ready"

# Run migrations
echo "📦 Running database migrations..."
if command -v sqlx &> /dev/null; then
    sqlx migrate run
else
    echo "⚠️  sqlx-cli not found. Skipping migrations."
    echo "   Install with: cargo install sqlx-cli --no-default-features --features postgres"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "🔗 Access points:"
echo "   PostgreSQL: localhost:5432"
echo "   Redis: localhost:6379"
echo ""
echo "📚 Next steps:"
echo "   1. Run backend: cd crates/backend-api && cargo run"
echo "   2. Run frontend: cd crates/frontend-web && trunk serve"
echo ""
echo "💡 Development tools (optional):"
echo "   pgAdmin: http://localhost:5050"
echo "   Redis Commander: http://localhost:8081"
echo "   Start with: docker-compose --profile dev up -d"
