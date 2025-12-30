# Jirsi Platform Edge Deployment Guide

## Fly.io Setup

### Prerequisites
```bash
# Install Fly CLI
curl -L https://fly.io/install.sh | sh

# Login
fly auth login
```

### Initial Deployment
```bash
# Create app
fly apps create jirsi-api

# Create volume for data
fly volumes create jirsi_data --region ams --size 10

# Set secrets
fly secrets set \
    DATABASE_URL="postgres://..." \
    OPENAI_API_KEY="sk-..." \
    TWILIO_ACCOUNT_SID="..." \
    TWILIO_AUTH_TOKEN="..." \
    STRIPE_SECRET_KEY="sk_live_..." \
    JWT_SECRET="..."

# Deploy
fly deploy
```

### PostgreSQL Read Replicas

```bash
# Create primary in Amsterdam
fly postgres create --name jirsi-db --region ams --vm-size shared-cpu-2x

# Add read replica in US
fly postgres readreplica create --name jirsi-db-us --region iad

# Add read replica in Singapore
fly postgres readreplica create --name jirsi-db-sg --region sin
```

### Geo-Routing Configuration

The application uses Fly's anycast routing to automatically route requests to the nearest region.

#### Read Replica Connection

```rust
// In your app, detect region and use appropriate connection
let region = std::env::var("FLY_REGION").unwrap_or("ams".to_string());

let read_db_url = match region.as_str() {
    "iad" | "ord" | "mia" => std::env::var("DATABASE_URL_US"),
    "sin" | "hkg" | "nrt" => std::env::var("DATABASE_URL_ASIA"),
    _ => std::env::var("DATABASE_URL"), // Primary
};
```

### Scaling

```bash
# Scale regions
fly scale count 3 --region ams
fly scale count 2 --region iad
fly scale count 2 --region sin

# Scale VM size
fly scale vm shared-cpu-4x

# Auto-scaling is configured in fly.toml
```

### Monitoring

```bash
# View logs
fly logs

# Check status
fly status

# SSH into instance
fly ssh console
```

### Custom Domains

```bash
# Add custom domain
fly certs create jirsi.com
fly certs create api.jirsi.com

# Configure DNS
# A record: @ -> <fly-ipv4>
# AAAA record: @ -> <fly-ipv6>
# CNAME record: api -> jirsi-api.fly.dev
```

## Performance Optimizations

1. **Static Asset CDN**: Serve static assets from Fly's CDN
2. **Database Connection Pooling**: PgBouncer for connection reuse
3. **Redis Cache**: For session and frequently accessed data
4. **Brotli Compression**: Enabled in reverse proxy

## Rollback

```bash
# List releases
fly releases

# Rollback to previous
fly releases rollback 42
```
