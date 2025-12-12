# SaaS Platform

A multi-tenant SaaS platform built with Rust (Axum + Leptos + PostgreSQL).

## Architecture

- **Metadata-driven**: Apps, Entities, Fields, Views are all configurable
- **Node Engine**: Houdini-style visual programming for automations
- **Multi-tenant**: Subdomain-based tenant isolation

## Quick Start

### Prerequisites

- Rust nightly (for Leptos)
- Docker & Docker Compose
- PostgreSQL 16+

### Development

1. Start the database:
   ```bash
   docker compose -f infra/docker-compose.yml up -d
   ```

2. Set up environment:
   ```bash
   export DATABASE_URL="postgres://postgres:postgres@localhost:5432/saas_platform"
   ```

3. Run migrations:
   ```bash
   cargo sqlx migrate run
   ```

4. Start the backend:
   ```bash
   cargo run -p backend-api
   ```

5. Start the frontend (in another terminal):
   ```bash
   cd crates/frontend-web
   trunk serve
   ```

## Project Structure

```
crates/
├── core-models/        # Shared domain types
├── core-metadata/      # Metadata engine
├── core-auth/          # Authentication & authorization
├── core-node-engine/   # Node graph execution
├── core-engagement/    # Interactions, calendar
├── core-analytics/     # Search, metrics, AI
├── app-crm/            # CRM application
├── app-properties/     # Properties application
├── backend-api/        # Axum HTTP server
├── frontend-web/       # Leptos WASM frontend
└── jobs-runner/        # Background worker
```

## Phase 1 Checklist

- [x] Monorepo structure
- [ ] Database migrations
- [ ] Tenant/User/Auth
- [ ] EntityType/FieldDef
- [ ] Generic UI Shell
- [ ] CRM entities

## License

MIT
