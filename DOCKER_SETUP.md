# Jirsi Platform - Quick Start Guide

## 🚀 Quick Start (Docker)

### Prerequisites
- Docker & Docker Compose installed
- Rust (latest stable)
- Node.js 18+

### 1. Start Infrastructure
```bash
# Create .env file
cp .env.docker.example .env.docker

# Edit .env.docker with your passwords

# Start PostgreSQL + Redis
docker-compose up -d

# Start with dev tools (pgAdmin + Redis Commander)
docker-compose --profile dev up -d
```

### 2. Run Migrations
```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run --database-url "postgres://jirsi:your_password@localhost:5432/jirsi_db"
```

### 3. Start Backend
```bash
cd crates/backend-api
cargo run
```

### 4. Start Frontend
```bash
cd crates/frontend-web
trunk serve
```

---

## 🔧 Development Tools

### pgAdmin (PostgreSQL UI)
- URL: http://localhost:5050
- Email: admin@jirsi.local
- Password: (see .env.docker)

### Redis Commander (Redis UI)
- URL: http://localhost:8081

---

## 📊 Verify Setup

### Check PostgreSQL
```bash
docker exec -it jirsi-postgres psql -U jirsi -d jirsi_db -c "\dt"
```

### Check Redis
```bash
docker exec -it jirsi-redis redis-cli -a your_redis_password ping
```

### Check Event Store
```bash
docker exec -it jirsi-postgres psql -U jirsi -d jirsi_db -c "SELECT COUNT(*) FROM events;"
```

---

## 🧪 Run Tests

```bash
# Unit tests
cargo test

# Integration tests (requires running containers)
cargo test --features integration-tests

# E2E tests
cd crates/frontend-web
npm run test:e2e
```

---

## 📝 Common Commands

### Stop all services
```bash
docker-compose down
```

### Stop and remove volumes (⚠️ DATA LOSS)
```bash
docker-compose down -v
```

### View logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f postgres
```

### Rebuild after changes
```bash
docker-compose up -d --build
```

---

## 🔥 Production Deployment

See `docs/deployment.md` for production setup with:
- SSL/TLS configuration
- Backup strategies
- Monitoring setup
- Scaling guidelines
