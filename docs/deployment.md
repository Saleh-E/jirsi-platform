# Production Deployment Guide

Complete guide for deploying the Jirsi SmartField platform to production.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Environment Setup](#environment-setup)
3. [Docker Deployment](#docker-deployment)
4. [Database Setup](#database-setup)
5. [Build & Deploy](#build--deploy)
6. [Monitoring](#monitoring)
7. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software

- **Rust**: 1.75+ (for backend)
- **Node.js**: 18+ (for optional tooling)
- **PostgreSQL**: 15+
- **Docker**: 24+ (recommended)
- **Docker Compose**: 2.20+

### System Requirements

**Minimum**:
- 2 CPU cores
- 4GB RAM
- 20GB disk space

**Recommended**:
- 4 CPU cores
- 8GB RAM
- 50GB SSD storage

---

## Environment Setup

### 1. Environment Variables

Create `.env` file in project root:

```bash
# Database
DATABASE_URL=postgresql://jirsi:password@localhost:5432/jirsi_prod
DATABASE_POOL_SIZE=20

# Server
HOST=0.0.0.0
PORT=8080
RUST_LOG=info
ENVIRONMENT=production

# Security
SECRET_KEY=your-secret-key-min-32-chars
JWT_SECRET=your-jwt-secret-min-32-chars
CORS_ORIGINS=https://yourdomain.com

# Frontend
TRUNK_SERVE_ADDRESS=0.0.0.0:8081
TRUNK_PUBLIC_URL=/

# Email (optional)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password

# Storage (optional)
S3_BUCKET=jirsi-uploads
S3_REGION=us-east-1
AWS_ACCESS_KEY_ID=your-key
AWS_SECRET_ACCESS_KEY=your-secret
```

### 2. Generate Secrets

```bash
# Generate SECRET_KEY
openssl rand -base64 32

# Generate JWT_SECRET
openssl rand -base64 32
```

---

## Docker Deployment

### Method 1: Docker Compose (Recommended)

**docker-compose.yml**:

```yaml
version: '3.8'

services:
  # PostgreSQL Database
  postgres:
    image: postgres:15-alpine
    container_name: jirsi-postgres
    environment:
      POSTGRES_DB: jirsi_prod
      POSTGRES_USER: jirsi
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      PGDATA: /var/lib/postgresql/data/pgdata
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U jirsi"]
      interval: 10s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # Backend API
  backend:
    build:
      context: .
      dockerfile: Dockerfile.backend
    container_name: jirsi-backend
    environment:
      DATABASE_URL: postgresql://jirsi:${DB_PASSWORD}@postgres:5432/jirsi_prod
      HOST: 0.0.0.0
      PORT: 8080
      SECRET_KEY: ${SECRET_KEY}
      JWT_SECRET: ${JWT_SECRET}
      RUST_LOG: info
      ENVIRONMENT: production
    ports:
      - "8080:8080"
    depends_on:
      postgres:
        condition: service_healthy
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # Frontend (Trunk build)
  frontend:
    build:
      context: .
      dockerfile: Dockerfile.frontend
    container_name: jirsi-frontend
    ports:
      - "8081:80"
    restart: unless-stopped

  # NGINX Reverse Proxy
  nginx:
    image: nginx:alpine
    container_name: jirsi-nginx
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    depends_on:
      - backend
      - frontend
    restart: unless-stopped

volumes:
  postgres_data:
```

### Backend Dockerfile

**Dockerfile.backend**:

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates crates/

# Build release
RUN cargo build --release --bin jirsi-backend

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/jirsi-backend /app/jirsi-backend

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run
CMD ["/app/jirsi-backend"]
```

### Frontend Dockerfile

**Dockerfile.frontend**:

```dockerfile
# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install trunk and wasm target
RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

# Copy source
COPY crates/frontend-web crates/frontend-web/
COPY crates/core-models crates/core-models/
COPY Cargo.toml Cargo.lock ./

# Build frontend
WORKDIR /app/crates/frontend-web
RUN trunk build --release

# Runtime stage  
FROM nginx:alpine

# Copy built files
COPY --from=builder /app/crates/frontend-web/dist /usr/share/nginx/html

# Copy nginx config
COPY nginx-frontend.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### Deploy with Docker Compose

```bash
# Set environment variables
export DB_PASSWORD=$(openssl rand -base64 32)
export SECRET_KEY=$(openssl rand -base64 32)
export JWT_SECRET=$(openssl rand -base64 32)

# Save to .env file
cat > .env << EOF
DB_PASSWORD=$DB_PASSWORD
SECRET_KEY=$SECRET_KEY
JWT_SECRET=$JWT_SECRET
EOF

# Build and start
docker-compose up -d

# Check logs
docker-compose logs -f

# Check health
docker-compose ps
```

---

## Database Setup

### Initialize Database

```bash
# Run migrations
docker-compose exec backend /app/jirsi-backend migrate

# Or manually with sqlx
sqlx database create --database-url "$DATABASE_URL"
sqlx migrate run --database-url "$DATABASE_URL"
```

### Backup Script

**backup.sh**:

```bash
#!/bin/bash

BACKUP_DIR="/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/jirsi_backup_$TIMESTAMP.sql"

# Create backup
docker-compose exec -T postgres pg_dump -U jirsi jirsi_prod > "$BACKUP_FILE"

# Compress
gzip "$BACKUP_FILE"

# Keep only last 30 days
find "$BACKUP_DIR" -name "jirsi_backup_*.sql.gz" -mtime +30 -delete

echo "Backup created: $BACKUP_FILE.gz"
```

### Restore from Backup

```bash
# Uncompress
gunzip jirsi_backup_TIMESTAMP.sql.gz

# Restore
docker-compose exec -T postgres psql -U jirsi jirsi_prod < jirsi_backup_TIMESTAMP.sql
```

---

## Build & Deploy

### Manual Deployment (Without Docker)

#### Backend

```bash
# Build release
cargo build --release --bin jirsi-backend

# Run migrations
DATABASE_URL="postgresql://..." ./target/release/jirsi-backend migrate

# Start server
DATABASE_URL="postgresql://..." \
SECRET_KEY="..." \
./target/release/jirsi-backend
```

#### Frontend

```bash
# Install trunk
cargo install trunk

# Add WASM target
rustup target add wasm32-unknown-unknown

# Build
cd crates/frontend-web
trunk build --release

# Output in: dist/
```

### NGINX Configuration

**nginx.conf**:

```nginx
events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Logging
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript 
               application/x-javascript application/xml+rss 
               application/json application/wasm;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;

    # Upstream backend
    upstream backend {
        server backend:8080;
    }

    # Upstream frontend
    upstream frontend {
        server frontend:80;
    }

    # HTTP to HTTPS redirect
    server {
        listen 80;
        server_name yourdomain.com;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS server
    server {
        listen 443 ssl http2;
        server_name yourdomain.com;

        # SSL certificates
        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;
        
        # SSL settings
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        ssl_prefer_server_ciphers on;

        # Security headers
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;
        add_header Referrer-Policy "no-referrer-when-downgrade" always;
        add_header Content-Security-Policy "default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; img-src 'self' data: https:; font-src 'self' data:;" always;

        # API routes
        location /api/ {
            limit_req zone=api burst=20 nodelay;
            
            proxy_pass http://backend;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection 'upgrade';
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_cache_bypass $http_upgrade;
            
            # Timeouts
            proxy_connect_timeout 60s;
            proxy_send_timeout 60s;
            proxy_read_timeout 60s;
        }

        # Login endpoint (stricter rate limit)
        location /api/v1/auth/login {
            limit_req zone=login burst=3 nodelay;
            
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # Frontend static files
        location / {
            proxy_pass http://frontend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            
            # Cache static assets
            location ~* \.(js|css|wasm|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
                proxy_pass http://frontend;
                expires 1y;
                add_header Cache-Control "public, immutable";
            }
        }

        # Health check
        location /health {
            proxy_pass http://backend/health;
            access_log off;
        }
    }
}
```

---

## Monitoring

### Health Check Endpoints

Add to backend:

```rust
// src/routes/health.rs
use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
```

### Logging

**Production logging setup**:

```rust
// src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

### Monitoring Stack (Optional)

Add to `docker-compose.yml`:

```yaml
  # Prometheus
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    ports:
      - "9090:9090"
    restart: unless-stopped

  # Grafana
  grafana:
    image: grafana/grafana:latest
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
    ports:
      - "3000:3000"
    depends_on:
      - prometheus
    restart: unless-stopped

volumes:
  prometheus_data:
  grafana_data:
```

---

## SSL/TLS Setup

### Let's Encrypt with Certbot

```bash
# Install certbot
docker run -it --rm \
  -v "/etc/letsencrypt:/etc/letsencrypt" \
  -v "/var/lib/letsencrypt:/var/lib/letsencrypt" \
  certbot/certbot certonly \
  --standalone \
  -d yourdomain.com \
  -d www.yourdomain.com \
  --email your@email.com \
  --agree-tos \
  --non-interactive

# Copy certs to nginx volume
cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem ./ssl/
cp /etc/letsencrypt/live/yourdomain.com/privkey.pem ./ssl/

# Restart nginx
docker-compose restart nginx
```

### Auto-renewal Cron

```bash
# Add to crontab
0 0 * * 0 docker run --rm \
  -v "/etc/letsencrypt:/etc/letsencrypt" \
  -v "/var/lib/letsencrypt:/var/lib/letsencrypt" \
  certbot/certbot renew && \
  docker-compose restart nginx
```

---

## Troubleshooting

### Database Connection Issues

```bash
# Check postgres is running
docker-compose ps postgres

# Check connectivity
docker-compose exec backend psql $DATABASE_URL -c "SELECT 1"

# Check logs
docker-compose logs postgres
```

### Frontend Not Loading

```bash
# Check WASM files are served correctly
curl -I https://yourdomain.com/app.wasm

# Should return: Content-Type: application/wasm

# If not, check NGINX mime types
docker-compose exec nginx cat /etc/nginx/mime.types | grep wasm
```

### High Memory Usage

```bash
# Check container stats
docker stats

# Limit backend memory
# Add to docker-compose.yml:
services:
  backend:
    mem_limit: 1g
    memswap_limit: 1g
```

### Slow API Responses

```bash
# Check database query performance
docker-compose exec postgres psql -U jirsi jirsi_prod -c "
  SELECT query, calls, total_exec_time, mean_exec_time 
  FROM pg_stat_statements 
  ORDER BY mean_exec_time DESC 
  LIMIT 10;
"

# Enable query logging
# Add to postgresql.conf:
log_min_duration_statement = 1000  # Log queries > 1s
```

---

## Security Checklist

- [ ] Change all default passwords
- [ ] Use strong SECRET_KEY and JWT_SECRET
- [ ] Enable HTTPS/TLS
- [ ] Set up firewall rules
- [ ] Enable rate limiting
- [ ] Regular security updates
- [ ] Database backups automated
- [ ] Monitoring and alerts configured
- [ ] CORS configured correctly
- [ ] CSP headers set

---

## Performance Optimization

### Database Indexing

```sql
-- Add indexes for common queries
CREATE INDEX idx_fields_entity_type ON fields(entity_type_id);
CREATE INDEX idx_records_entity_type ON records(entity_type_id);
CREATE INDEX idx_records_created_at ON records(created_at DESC);
```

### Connection Pooling

```rust
// Adjust pool size based on load
let pool = PgPoolOptions::new()
    .max_connections(20)  // Increase for high traffic
    .connect(&database_url)
    .await?;
```

### Frontend Optimization

```bash
# Build with optimizations
trunk build --release

# Results in:
# - Minified JS/WASM
# - Gzipped assets
# - Tree-shaken dependencies
```

---

## Maintenance

### Regular Tasks

**Daily**:
- Check logs for errors
- Monitor disk space
- Verify backups

**Weekly**:
- Review security logs
- Check performance metrics
- Update dependencies

**Monthly**:
- Security patches
- Database optimization
- Test backup restore

---

## Rollback Procedure

```bash
1. Stop current deployment
docker-compose down

# Restore database backup
docker-compose exec -T postgres psql -U jirsi jirsi_prod < backup.sql

# Checkout previous version
git checkout v1.0.0

# Rebuild and deploy
docker-compose up -d --build
```

---

## Support

For production issues:
- Check logs: `docker-compose logs -f`
- GitHub Issues: [link]
- Email: support@yourdomain.com

---

**Version**: 1.0.0  
**Last Updated**: 2024-12-24
